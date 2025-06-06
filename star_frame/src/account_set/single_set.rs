use crate::account_set::{HasOwnerProgram, SignedAccount, WritableAccount};
use crate::anyhow::Result;
use crate::client::MakeCpi;
use crate::prelude::{StarFrameProgram, SyscallInvoke, System};
use crate::program::system;
use crate::syscalls::SyscallCore;
use anyhow::{anyhow, bail, Context};
use pinocchio::account_info::{AccountInfo, Ref, RefMut};
use pinocchio::program_error::ProgramError;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;
use std::cmp::Ordering;
use std::mem::size_of;

#[derive(Debug, Clone, Copy)]
pub struct SingleSetMeta {
    pub signer: bool,
    pub writable: bool,
}

impl SingleSetMeta {
    #[must_use]
    pub const fn default() -> Self {
        Self {
            signer: false,
            writable: false,
        }
    }
}

/// An [`AccountSet`] that contains exactly 1 account.
pub trait SingleAccountSet {
    /// Associated metadata for the account set
    fn meta() -> SingleSetMeta
    where
        Self: Sized;

    /// Gets the contained account by reference
    fn account_info(&self) -> &AccountInfo;

    /// Gets the account meta of the contained account.
    #[inline]
    fn account_meta(&self) -> AccountMeta {
        let info = self.account_info();
        AccountMeta {
            pubkey: *info.pubkey(),
            is_signer: info.is_signer(),
            is_writable: info.is_writable(),
        }
    }

    /// Gets whether this account signed.
    #[inline]
    fn is_signer(&self) -> bool {
        self.account_info().is_signer()
    }

    /// Checks if this account is signed.
    #[inline]
    fn check_signer(&self) -> Result<()> {
        if self.is_signer() {
            Ok(())
        } else {
            bail!("Account {} is not signed", self.pubkey())
        }
    }

    /// Gets whether this account is writable.
    #[inline]
    fn is_writable(&self) -> bool {
        self.account_info().is_writable()
    }

    /// Checks if this account is writable.
    #[inline]
    fn check_writable(&self) -> Result<()> {
        if self.is_writable() {
            Ok(())
        } else {
            bail!("Account {} is not writable", self.pubkey())
        }
    }

    /// Gets the key of the contained account.
    #[inline]
    fn pubkey(&self) -> &Pubkey {
        self.account_info().pubkey()
    }

    /// Checks if the key matches the expected key.
    #[inline]
    fn check_key(&self, expected: &Pubkey) -> Result<()> {
        if self.pubkey() == expected {
            Ok(())
        } else {
            bail!(
                "Account key {} does not match expected public key {}",
                self.pubkey(),
                expected
            )
        }
    }

    /// Gets the owner of the contained account.
    #[inline]
    fn owner_pubkey(&self) -> Pubkey {
        bytemuck::cast(self.account_info().owner_key())
    }

    /// Gets the data of the contained account immutably.
    #[inline]
    fn account_data(&self) -> Result<Ref<'_, [u8]>> {
        self.account_info()
            .try_borrow_data()
            .with_context(|| format!("Failed to borrow data for account {}", self.pubkey()))
    }
    /// Gets the data of the contained account mutably.
    #[inline]
    fn account_data_mut(&self) -> Result<RefMut<'_, [u8]>> {
        self.account_info().try_borrow_mut_data().with_context(|| {
            format!(
                "Failed to borrow mutable data for account {}",
                self.pubkey()
            )
        })
    }
}

static_assertions::assert_obj_safe!(CanCloseAccount, CanReceiveRent, CanFundRent);

pub trait CanCloseAccount: SingleAccountSet {
    /// Closes the account by zeroing the lamports and replacing the discriminant with all `u8::MAX`,
    /// reallocating down to size.
    fn close(&self, recipient: &(impl CanReceiveRent + ?Sized)) -> Result<()>
    where
        Self: HasOwnerProgram,
        Self: Sized,
    {
        let info = self.account_info();
        info.realloc(
            size_of::<<Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        self.account_info().try_borrow_mut_data()?.fill(u8::MAX);
        recipient.receive_rent(info.lamports())?;
        *info.try_borrow_mut_lamports()? = 0;
        Ok(())
    }

    /// Closes the account by reallocating to zero and assigning to the System program.
    /// This is the same as calling `close` but not abusable and harder for indexer detection.
    ///
    /// It also happens to be unsound because [`AccountInfo::assign`] is unsound.
    fn close_full(&self, recipient: &dyn CanReceiveRent) -> Result<()> {
        let info = self.account_info();
        recipient.receive_rent(info.lamports())?;
        *info.try_borrow_mut_lamports()? = 0;
        info.realloc(0, false)?;
        unsafe { info.assign(System::ID.as_array()) }; // TODO: Fix safety
        Ok(())
    }
}

impl<T> CanCloseAccount for T where T: SingleAccountSet + ?Sized {}

pub trait CanReceiveRent {
    fn account_to_modify(&self) -> AccountInfo;
    fn receive_rent(&self, lamports: u64) -> Result<()> {
        *self.account_to_modify().try_borrow_mut_lamports()? += lamports;
        Ok(())
    }
}

impl<T> CanReceiveRent for T
where
    T: WritableAccount + ?Sized,
{
    fn account_to_modify(&self) -> AccountInfo {
        *self.account_info()
    }
}

/// Indicates that this account can fund rent on another account, and potentially be used to create an account.
pub trait CanFundRent: CanReceiveRent {
    /// Whether [`Self::account_to_modify`](`CanReceiveRent::account_to_modify`) can be used as the funder for a [`system_program::CreateAccount`] CPI.
    fn can_create_account(&self) -> bool;
    /// Increases the rent of the recipient by `lamports`.
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet,
        lamports: u64,
        syscalls: &dyn SyscallInvoke,
    ) -> Result<()>;

    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

impl<T> CanFundRent for T
where
    T: CanReceiveRent + SignedAccount + ?Sized,
{
    fn can_create_account(&self) -> bool {
        true
    }
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet,
        lamports: u64,
        syscalls: &dyn SyscallInvoke,
    ) -> Result<()> {
        let cpi = System::cpi(
            &system::Transfer { lamports },
            system::TransferCpiAccounts {
                funder: *self.account_info(),
                recipient: *recipient.account_info(),
            },
        )?;
        match self.signer_seeds() {
            None => cpi.invoke(syscalls)?,
            Some(seeds) => cpi.invoke_signed(&[&seeds], syscalls)?,
        }
        Ok(())
    }

    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        SignedAccount::signer_seeds(self)
    }
}

pub trait CanModifyRent {
    fn account_to_modify(&self) -> AccountInfo;

    /// Normalizes the rent of an account if data size is changed.
    /// Assumes `Self` is mutable and owned by this program.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn normalize_rent(
        &self,
        funder: &(impl CanFundRent + ?Sized),
        syscalls: &dyn SyscallInvoke,
    ) -> Result<()> {
        let account = self.account_to_modify();
        let rent = syscalls.get_rent()?;
        let lamports = *account.try_borrow_lamports()?;
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                if lamports == 0 {
                    return Ok(());
                }
                let transfer_amount = rent_lamports - lamports;
                CanFundRent::fund_rent(funder, &account, transfer_amount, syscalls)?;
                Ok(())
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                *account.try_borrow_mut_lamports()? -= transfer_amount;
                funder.receive_rent(transfer_amount)?;
                Ok(())
            }
        }
    }

    /// Refunds rent to the funder so long as the account has more than the minimum rent.
    /// Assumes `Self` is owned by this program and is mutable.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn refund_rent(
        &self,
        recipient: &(impl CanReceiveRent + ?Sized),
        syscalls: &(impl SyscallInvoke + ?Sized),
    ) -> Result<()> {
        let account = self.account_to_modify();
        let rent = syscalls.get_rent()?;
        let lamports = *account.try_borrow_lamports()?;
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                if lamports > 0 {
                    Err(anyhow!(
                        "Tried to refund rent from {} but does not have enough lamports to cover rent",
                        account.pubkey()
                    ))
                } else {
                    Ok(())
                }
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                *account.try_borrow_mut_lamports()? -= transfer_amount;
                recipient.receive_rent(transfer_amount)?;
                Ok(())
            }
        }
    }

    /// Receive rent to self to be at least the minimum rent. This will not normalize down excess lamports.
    /// Assumes `Self` is owned by this program and is mutable.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn receive_rent(
        &self,
        funder: &(impl CanFundRent + ?Sized),
        syscalls: &dyn SyscallInvoke,
    ) -> Result<()> {
        let account = self.account_to_modify();
        let rent = syscalls.get_rent()?;
        let lamports = *account.try_borrow_lamports()?;
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        if rent_lamports > lamports {
            if lamports == 0 {
                return Ok(());
            }
            let transfer_amount = rent_lamports - lamports;
            CanFundRent::fund_rent(funder, &account, transfer_amount, syscalls)?;
        }
        Ok(())
    }

    /// Emits a warning message if the account has more lamports than required by rent.
    #[cfg_attr(not(feature = "cleanup_rent_warning"), allow(unused_variables))]
    fn check_cleanup(&self, sys_calls: &(impl SyscallCore + ?Sized)) -> Result<()> {
        #[cfg(feature = "cleanup_rent_warning")]
        {
            use std::cmp::Ordering;
            let account = self.account_to_modify();
            if account.is_writable() {
                let rent = sys_calls.get_rent()?;
                let lamports = account.lamports();
                let data_len = account.data_len();
                let rent_lamports = rent.minimum_balance(data_len);
                if rent_lamports.cmp(&lamports) == Ordering::Less {
                    pinocchio::msg!(
                        "{} was left with more lamports than required by rent",
                        account.pubkey()
                    );
                }
            }
        }
        Ok(())
    }
}

impl<T> CanModifyRent for T
where
    T: SingleAccountSet + ?Sized,
{
    fn account_to_modify(&self) -> AccountInfo {
        *self.account_info()
    }
}

pub trait CanSystemCreateAccount {
    fn account_to_create(&self) -> AccountInfo;
    /// Creates an account using the system program
    /// Assumes `Self` is owned by the System program and funder is a System account
    fn system_create_account(
        &self,
        funder: &(impl CanFundRent + ?Sized),
        owner: Pubkey,
        space: usize,
        account_seeds: &Option<Vec<&[u8]>>,
        syscalls: &dyn SyscallInvoke,
    ) -> Result<()> {
        let account = self.account_to_create();
        if account.owner_pubkey() != System::ID {
            bail!(ProgramError::InvalidAccountOwner);
        }
        let current_lamports = account.lamports();
        let exempt_lamports = syscalls.get_rent()?.minimum_balance(space);

        if current_lamports == 0 && funder.can_create_account() {
            let funder_seeds: Option<Vec<&[u8]>> = funder.signer_seeds();
            let seeds: &[&[&[u8]]] = match (&funder_seeds, account_seeds) {
                (Some(signer_seeds), Some(account_seeds)) => &[signer_seeds, account_seeds],
                (Some(signer_seeds), None) => &[signer_seeds],
                (None, Some(account_seeds)) => &[account_seeds],
                (None, None) => &[],
            };
            System::cpi(
                &system::CreateAccount {
                    lamports: exempt_lamports,
                    space: space as u64,
                    owner,
                },
                system::CreateAccountCpiAccounts {
                    funder: funder.account_to_modify(),
                    new_account: account,
                },
            )?
            .invoke_signed(seeds, syscalls)?;
        } else {
            let required_lamports = exempt_lamports.saturating_sub(current_lamports).max(1);
            if required_lamports > 0 {
                CanFundRent::fund_rent(funder, &account, required_lamports, syscalls)?;
            }
            let account_seeds: &[&[&[u8]]] = match &account_seeds {
                Some(seeds) => &[seeds],
                None => &[],
            };
            System::cpi(
                &system::Allocate {
                    space: space as u64,
                },
                system::AllocateCpiAccounts { account },
            )?
            .invoke_signed(account_seeds, syscalls)?;
            System::cpi(
                &system::Assign { owner },
                system::AssignCpiAccounts { account },
            )?
            .invoke_signed(account_seeds, syscalls)?;
        }
        Ok(())
    }
}

impl<T> CanSystemCreateAccount for T
where
    T: SingleAccountSet + ?Sized,
{
    fn account_to_create(&self) -> AccountInfo {
        *self.account_info()
    }
}
