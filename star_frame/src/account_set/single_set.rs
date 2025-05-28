use crate::account_set::{AccountSet, HasOwnerProgram, SignedAccount, WritableAccount};
use crate::anyhow::Result;
use crate::client::MakeCpi;
use crate::prelude::{StarFrameProgram, SyscallInvoke, System};
use crate::program::system_program;
use crate::syscalls::SyscallCore;
use anyhow::{anyhow, bail};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::cell::{Ref, RefMut};
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
pub trait SingleAccountSet<'info>: AccountSet<'info> {
    /// Associated metadata for the account set
    fn meta() -> SingleSetMeta
    where
        Self: Sized;

    /// Gets the contained account.
    fn account_info(&self) -> &AccountInfo<'info>;
    /// Gets the contained account cloned.
    #[inline]
    fn account_info_cloned(&self) -> AccountInfo<'info> {
        self.account_info().clone()
    }
    /// Gets the account meta of the contained account.
    #[inline]
    fn account_meta(&self) -> AccountMeta {
        let info = self.account_info();
        AccountMeta {
            pubkey: *info.key(),
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
            bail!("Account {} is not signed", self.key())
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
            bail!("Account {} is not writable", self.key())
        }
    }

    /// Gets the key of the contained account.
    #[inline]
    fn key(&self) -> &'info Pubkey {
        self.account_info().key()
    }

    /// Checks if the key matches the expected key.
    #[inline]
    fn check_key(&self, expected: &Pubkey) -> Result<()> {
        if self.key() == expected {
            Ok(())
        } else {
            bail!(
                "Account key {} does not match expected public key {}",
                self.key(),
                expected
            )
        }
    }

    /// Gets the owner of the contained account.
    #[inline]
    fn owner(&self) -> &'info Pubkey {
        self.account_info().owner()
    }

    /// Gets the data of the contained account immutably.
    #[inline]
    fn info_data_bytes<'a>(&'a self) -> Result<Ref<'a, [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes()
    }
    /// Gets the data of the contained account mutably.
    #[inline]
    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, &'info mut [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes_mut()
    }
}

pub trait CanCloseAccount<'info>: SingleAccountSet<'info> {
    /// Closes the account by zeroing the lamports and replacing the discriminant with all `u8::MAX`,
    /// reallocating down to size.
    fn close(&self, recipient: &dyn CanReceiveRent<'info>) -> Result<()>
    where
        Self: HasOwnerProgram,
        Self: Sized,
    {
        let info = self.account_info();
        info.realloc(
            size_of::<<Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        self.info_data_bytes_mut()?.fill(u8::MAX);
        recipient.receive_rent(**info.try_borrow_lamports()?)?;
        **info.try_borrow_mut_lamports()? = 0;
        Ok(())
    }

    /// Closes the account by reallocating to zero and assigning to the System program.
    /// This is the same as calling `close` but not abusable and harder for indexer detection.
    ///
    /// It also happens to be unsound because [`AccountInfo::assign`] is unsound.
    fn close_full(&self, recipient: &dyn CanReceiveRent<'info>) -> Result<()> {
        let info = self.account_info();
        recipient.receive_rent(**info.try_borrow_lamports()?)?;
        **info.try_borrow_mut_lamports()? = 0;
        info.realloc(0, false)?;
        info.assign(&System::ID);
        Ok(())
    }
}

impl<'info, T> CanCloseAccount<'info> for T where T: SingleAccountSet<'info> {}

pub trait CanReceiveRent<'info> {
    fn account_to_modify(&self) -> &AccountInfo<'info>;
    fn receive_rent(&self, lamports: u64) -> Result<()> {
        **self.account_to_modify().try_borrow_mut_lamports()? += lamports;
        Ok(())
    }
}

impl<'info, T> CanReceiveRent<'info> for T
where
    T: WritableAccount<'info>,
{
    fn account_to_modify(&self) -> &AccountInfo<'info> {
        self.account_info()
    }
}

/// Indicates that this account can fund rent on another account, and potentially be used to create an account.
pub trait CanFundRent<'info>: CanReceiveRent<'info> {
    /// Whether [`Self::account_to_modify`](`CanReceiveRent::account_to_modify`) can be used as the funder for a [`system_program::CreateAccount`] CPI.
    fn can_create_account(&self) -> bool;
    /// Increases the rent of the recipient by `lamports`.
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet<'info>,
        lamports: u64,
        syscalls: &dyn SyscallInvoke<'info>,
    ) -> Result<()>;

    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

impl<'info, T> CanFundRent<'info> for T
where
    T: CanReceiveRent<'info> + SignedAccount<'info>,
{
    fn can_create_account(&self) -> bool {
        true
    }
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet<'info>,
        lamports: u64,
        syscalls: &dyn SyscallInvoke<'info>,
    ) -> Result<()> {
        let cpi = System::cpi(
            &system_program::Transfer { lamports },
            system_program::TransferCpiAccounts {
                funder: self.account_info_cloned(),
                recipient: recipient.account_info_cloned(),
            },
        )?;
        match self.signer_seeds() {
            None => cpi.invoke(syscalls)?,
            Some(seeds) => cpi.invoke_signed(&[&seeds], syscalls)?,
        };
        Ok(())
    }

    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        SignedAccount::signer_seeds(self)
    }
}

pub trait CanModifyRent<'info> {
    fn account_to_modify(&self) -> &AccountInfo<'info>;

    /// Normalizes the rent of an account if data size is changed.
    /// Assumes `Self` is mutable and owned by this program.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn normalize_rent(
        &self,
        funder: &dyn CanFundRent<'info>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let account = self.account_to_modify();
        let rent = syscalls.get_rent()?;
        let lamports = **account.try_borrow_lamports()?;
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                if lamports == 0 {
                    return Ok(());
                }
                let transfer_amount = rent_lamports - lamports;
                CanFundRent::fund_rent(funder, account, transfer_amount, syscalls)?;
                Ok(())
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                **account.try_borrow_mut_lamports()? -= transfer_amount;
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
        recipient: &dyn CanReceiveRent<'info>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let account = self.account_to_modify();
        let rent = syscalls.get_rent()?;
        let lamports = **account.try_borrow_lamports()?;
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                if lamports > 0 {
                    Err(anyhow!(
                        "Tried to refund rent from {} but does not have enough lamports to cover rent",
                        account.key()
                    ))
                } else {
                    Ok(())
                }
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                **account.try_borrow_mut_lamports()? -= transfer_amount;
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
        funder: &dyn CanFundRent<'info>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let account = self.account_to_modify();
        let rent = syscalls.get_rent()?;
        let lamports = **account.try_borrow_lamports()?;
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        if rent_lamports > lamports {
            if lamports == 0 {
                return Ok(());
            }
            let transfer_amount = rent_lamports - lamports;
            CanFundRent::fund_rent(funder, account, transfer_amount, syscalls)?;
        }
        Ok(())
    }

    /// Emits a warning message if the account has more lamports than required by rent.
    #[cfg_attr(not(feature = "cleanup_rent_warning"), allow(unused_variables))]
    fn check_cleanup(&self, sys_calls: &impl SyscallCore) -> Result<()> {
        #[cfg(feature = "cleanup_rent_warning")]
        {
            use std::cmp::Ordering;
            let account = self.account_to_modify();
            if account.is_writable() {
                let rent = sys_calls.get_rent()?;
                let lamports = **account.try_borrow_lamports()?;
                let data_len = account.data_len();
                let rent_lamports = rent.minimum_balance(data_len);
                if rent_lamports.cmp(&lamports) == Ordering::Less {
                    solana_program::msg!(
                        "{} was left with more lamports than required by rent",
                        account.key()
                    );
                }
            }
        }
        Ok(())
    }
}

impl<'info, T> CanModifyRent<'info> for T
where
    T: SingleAccountSet<'info>,
{
    fn account_to_modify(&self) -> &AccountInfo<'info> {
        self.account_info()
    }
}

pub trait CanSystemCreateAccount<'info> {
    fn account_to_create(&self) -> &AccountInfo<'info>;
    /// Creates an account using the system program
    /// Assumes `Self` is owned by the System program and funder is a System account
    fn system_create_account(
        &self,
        funder: &dyn CanFundRent<'info>,
        owner: Pubkey,
        space: usize,
        account_seeds: &Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let account = self.account_to_create();
        if account.owner() != &System::ID {
            bail!(ProgramError::InvalidAccountOwner);
        }
        let current_lamports = **account.try_borrow_lamports()?;
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
                &system_program::CreateAccount {
                    lamports: exempt_lamports,
                    space: space as u64,
                    owner,
                },
                system_program::CreateAccountCpiAccounts {
                    funder: funder.account_to_modify().clone(),
                    new_account: account.clone(),
                },
            )?
            .invoke_signed(seeds, syscalls)?;
        } else {
            let required_lamports = exempt_lamports.saturating_sub(current_lamports).max(1);
            if required_lamports > 0 {
                CanFundRent::fund_rent(funder, account, required_lamports, syscalls)?;
            }
            let account_seeds: &[&[&[u8]]] = match &account_seeds {
                Some(seeds) => &[seeds],
                None => &[],
            };
            System::cpi(
                &system_program::Allocate {
                    space: space as u64,
                },
                system_program::AllocateCpiAccounts {
                    account: account.clone(),
                },
            )?
            .invoke_signed(account_seeds, syscalls)?;
            System::cpi(
                &system_program::Assign { owner },
                system_program::AssignCpiAccounts {
                    account: account.clone(),
                },
            )?
            .invoke_signed(account_seeds, syscalls)?;
        }
        Ok(())
    }
}

impl<'info, T> CanSystemCreateAccount<'info> for T
where
    T: SingleAccountSet<'info>,
{
    fn account_to_create(&self) -> &AccountInfo<'info> {
        self.account_info()
    }
}
