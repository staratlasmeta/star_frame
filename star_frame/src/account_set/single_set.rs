//! Base trait for account sets containing exactly one account.
//!
//! The `SingleAccountSet` trait is a foundational building block for Star Frame's account system.
//! It represents account sets that contain exactly one account and provides the base functionality
//! that modifier types like `Signer<T>`, `Mut<T>`, and `Account<T>` build upon.

use crate::{
    account_set::{
        modifiers::{HasOwnerProgram, OwnerProgramDiscriminant, SignedAccount, WritableAccount},
        CanAddLamports, CanCloseAccount, CanFundRent, CanModifyRent, CanSystemCreateAccount,
        CheckKey,
    },
    prelude::*,
    program::system,
    ErrorCode,
};
use core::cmp::Ordering;
use pinocchio::account::{Ref, RefMut};

/// Metadata associated with a single account, describing its mutability and signing requirements.
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

/// Base trait for account sets that contain exactly one account.
///
/// This trait serves as the foundation for all single-account types in Star Frame's account system.
/// Modifier types like `Signer<T>`, `Mut<T>`, `Account<T>`, and others require their inner type `T`
/// to implement `SingleAccountSet`, allowing them to add additional validation and behavior while
/// preserving access to the underlying account.
pub trait SingleAccountSet {
    /// Associated metadata for the account set
    fn meta() -> SingleSetMeta
    where
        Self: Sized;

    /// Gets the contained account by reference
    fn account_info(&self) -> &AccountView;

    /// Returns true if this account is signed.
    #[inline]
    fn is_signer(&self) -> bool {
        self.account_info().is_signer()
    }

    /// Checks that this account is a signer. Returns an error if it is not.
    #[inline]
    fn check_signer(&self) -> Result<()> {
        if self.is_signer() {
            Ok(())
        } else {
            bail!(
                ErrorCode::ExpectedSigner,
                "Account {} is not signed",
                self.address()
            )
        }
    }

    /// Returns true if this account is writable.
    #[inline]
    fn is_writable(&self) -> bool {
        self.account_info().is_writable()
    }

    /// Checks that this account is writable. Returns an error if it is not.
    #[inline]
    fn check_writable(&self) -> Result<()> {
        if self.is_writable() {
            Ok(())
        } else {
            bail!(
                ErrorCode::ExpectedWritable,
                "Account {} is not writable",
                self.address()
            )
        }
    }

    /// Returns a reference to the public key of the contained account.
    #[inline]
    fn address(&self) -> &Address {
        self.account_info().address()
    }

    /// Returns the public key of the owner of the contained account.
    #[inline]
    fn owner_address(&self) -> Address {
        // SAFETY: We are copying out the owner immediately
        unsafe { *self.account_info().owner() }
    }

    /// Returns a reference to the data of the contained account.
    #[inline]
    fn account_data(&self) -> Result<Ref<'_, [u8]>> {
        self.account_info()
            .try_borrow()
            .with_ctx(|| format!("Failed to borrow data for account {}", self.address()))
    }

    /// Returns a mutable reference to the data of the contained account.
    #[inline]
    fn account_data_mut(&self) -> Result<RefMut<'_, [u8]>> {
        self.account_info().try_borrow_mut().with_ctx(|| {
            format!(
                "Failed to borrow mutable data for account {}",
                self.address()
            )
        })
    }
}

impl<T> CheckKey for T
where
    T: SingleAccountSet,
{
    #[inline]
    fn check_key(&self, expected: &Address) -> Result<()> {
        if self.account_info().address().fast_eq(expected) {
            Ok(())
        } else {
            bail!(
                ErrorCode::AddressMismatch,
                "Account key {} does not match expected public key {}",
                self.address(),
                expected
            )
        }
    }
}

impl<T> CanAddLamports for T
where
    T: WritableAccount + Debug + ?Sized,
{
    #[inline]
    fn account_to_modify(&self) -> AccountView {
        *self.account_info()
    }
}

impl<T> CanFundRent for T
where
    T: CanAddLamports + SignedAccount + ?Sized,
{
    #[inline]
    fn can_create_account(&self) -> bool {
        true
    }
    #[inline]
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet,
        lamports: u64,
        _ctx: &Context,
    ) -> Result<()> {
        let cpi = System::cpi(
            system::Transfer { lamports },
            system::TransferCpiAccounts {
                funder: *self.account_info(),
                recipient: *recipient.account_info(),
            },
            None,
        );
        match self.signer_seeds() {
            None => cpi.invoke()?,
            Some(seeds) => cpi.invoke_signed(&[&seeds])?,
        }
        Ok(())
    }

    #[inline]
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        SignedAccount::signer_seeds(self)
    }
}

impl<T> CanCloseAccount for T
where
    T: SingleAccountSet + ?Sized,
{
    #[inline]
    fn close_account(&self, recipient: &(impl CanAddLamports + ?Sized)) -> Result<()>
    where
        Self: HasOwnerProgram,
        Self: Sized,
    {
        let info = self.account_info();
        info.resize(size_of::<OwnerProgramDiscriminant<Self>>())?;
        info.account_data_mut()?.fill(u8::MAX);
        recipient.add_lamports(info.lamports())?;
        info.set_lamports(0);
        Ok(())
    }

    #[inline]
    fn close_account_full(&self, recipient: &dyn CanAddLamports) -> Result<()> {
        let info = self.account_info();
        recipient.add_lamports(info.lamports())?;
        info.close()?;
        Ok(())
    }
}

impl<T> CanModifyRent for T
where
    T: SingleAccountSet + ?Sized,
{
    #[inline]
    fn normalize_rent(&self, funder: &(impl CanFundRent + ?Sized), ctx: &Context) -> Result<()> {
        let account = self.account_info();
        let rent = ctx.get_rent()?;
        let lamports = account.lamports();
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                if lamports == 0 {
                    return Ok(());
                }
                let transfer_amount = rent_lamports - lamports;
                CanFundRent::fund_rent(funder, &account, transfer_amount, ctx)?;
                Ok(())
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                account.set_lamports(lamports - transfer_amount);
                funder.add_lamports(transfer_amount)?;
                Ok(())
            }
        }
    }

    #[inline]
    fn refund_rent(&self, recipient: &(impl CanAddLamports + ?Sized), ctx: &Context) -> Result<()> {
        let account = self.account_info();
        let rent = ctx.get_rent()?;
        let lamports = account.lamports();
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                ensure!(
                    lamports > 0,
                    ProgramError::InsufficientFunds,
                    "Tried to refund rent from {} but does not have enough lamports to cover rent",
                    account.address()
                );
                Ok(())
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                account.set_lamports(lamports - transfer_amount);
                recipient.add_lamports(transfer_amount)?;
                Ok(())
            }
        }
    }

    #[inline]
    fn receive_rent(&self, funder: &(impl CanFundRent + ?Sized), ctx: &Context) -> Result<()> {
        let account = self.account_info();
        let rent = ctx.get_rent()?;
        let lamports = account.lamports();
        let data_len = account.data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        if rent_lamports > lamports {
            if lamports == 0 {
                return Ok(());
            }
            let transfer_amount = rent_lamports - lamports;
            CanFundRent::fund_rent(funder, &account, transfer_amount, ctx)?;
        }
        Ok(())
    }

    #[cfg_attr(not(feature = "cleanup_rent_warning"), allow(unused_variables))]
    #[inline]
    fn check_cleanup(&self, ctx: &Context) -> Result<()> {
        #[cfg(feature = "cleanup_rent_warning")]
        {
            use core::cmp::Ordering;
            let account = self.account_info();
            if account.is_writable() {
                let rent = ctx.get_rent()?;
                let lamports = account.lamports();
                let data_len = account.data_len();
                let rent_lamports = rent.minimum_balance(data_len);
                if rent_lamports.cmp(&lamports) == Ordering::Less {
                    pinocchio_log::log!(
                        "{} was left with more lamports than required by rent",
                        account.address().to_string().as_str()
                    );
                }
            }
        }
        Ok(())
    }
}

impl<T> CanSystemCreateAccount for T
where
    T: SingleAccountSet + ?Sized,
{
    #[inline]
    fn system_create_account(
        &self,
        funder: &(impl CanFundRent + ?Sized),
        owner: Address,
        space: usize,
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<()> {
        let account = *self.account_info();
        let current_lamports = account.lamports();
        let exempt_lamports = ctx.get_rent()?.minimum_balance(space);

        if current_lamports == 0 && funder.can_create_account() {
            let funder_seeds: Option<Vec<&[u8]>> = funder.signer_seeds();
            let seeds: &[&[&[u8]]] = match (&funder_seeds, account_seeds) {
                (Some(signer_seeds), Some(account_seeds)) => &[signer_seeds, account_seeds],
                (Some(signer_seeds), None) => &[signer_seeds],
                (None, Some(account_seeds)) => &[account_seeds],
                (None, None) => &[],
            };
            System::cpi(
                system::CreateAccount {
                    lamports: exempt_lamports,
                    space: space as u64,
                    owner,
                },
                system::CreateAccountCpiAccounts {
                    funder: funder.account_to_modify(),
                    new_account: account,
                },
                None,
            )
            .invoke_signed(seeds)
            .ctx("System::CreateAccount CPI failed")?;
        } else {
            let required_lamports = exempt_lamports.saturating_sub(current_lamports).max(1);
            if required_lamports > 0 {
                CanFundRent::fund_rent(funder, &account, required_lamports, ctx)
                    .ctx("Failed to fund rent")?;
            }
            let account_seeds: &[&[&[u8]]] = match &account_seeds {
                Some(seeds) => &[seeds],
                None => &[],
            };
            System::cpi(
                system::Allocate {
                    space: space as u64,
                },
                system::AllocateCpiAccounts { account },
                None,
            )
            .invoke_signed(account_seeds)
            .ctx("System::Allocate CPI failed")?;
            System::cpi(
                system::Assign { owner },
                system::AssignCpiAccounts { account },
                None,
            )
            .invoke_signed(account_seeds)
            .ctx("System::Assign CPI failed")?;
        }
        Ok(())
    }
}
