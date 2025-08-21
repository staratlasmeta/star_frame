mod account;
mod impls;
mod modifiers;
mod program;
mod rest;
mod single_set;
mod system_account;
mod sysvar;
mod validated_account;

pub use account::*;
use anyhow::{anyhow, bail, Context as _};
pub use impls::*;
pub use modifiers::*;
use pinocchio::program_error::ProgramError;
pub use program::*;
pub use rest::*;
pub use single_set::*;
use solana_pubkey::Pubkey;
pub use star_frame_proc::AccountSet;
use std::fmt::Debug;
pub use system_account::*;
pub use sysvar::*;
pub use validated_account::*;

use crate::client::MakeCpi as _;
use crate::prelude::{Context, PackedValue, StarFrameProgram, System};
use crate::program::system;
use crate::Result;
use bytemuck::{bytes_of, from_bytes};
use pinocchio::account_info::AccountInfo;
use std::{cmp::Ordering, slice};

pub trait ProgramAccount: HasOwnerProgram {
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant;
    #[must_use]
    fn discriminant_bytes() -> Vec<u8> {
        bytes_of(&Self::DISCRIMINANT).into()
    }

    fn validate_account_info(info: &impl SingleAccountSet) -> Result<()> {
        if info.owner_pubkey() != Self::OwnerProgram::ID {
            bail!(
                "Account {} owner {} does not match expected program ID {}",
                info.pubkey(),
                info.owner_pubkey(),
                Self::OwnerProgram::ID
            );
        }
        let data = info.account_data()?;
        if data.len() < size_of::<OwnerProgramDiscriminant<Self>>()
            || from_bytes::<PackedValue<OwnerProgramDiscriminant<Self>>>(
                &data[..size_of::<OwnerProgramDiscriminant<Self>>()],
            ) != &Self::DISCRIMINANT
        {
            bail!(
                "Account {} data does not match expected discriminant for program {}",
                info.pubkey(),
                Self::OwnerProgram::ID
            )
        }
        Ok(())
    }
}

/// Convenience methods for decoding and validating a list of [`AccountInfo`]s to an [`AccountSet`]. Performs
/// [`AccountSetDecode::decode_accounts`] and [`AccountSetValidate::validate_accounts`] on the accounts.
///
/// See [`TryFromAccounts`] for a version of this trait that uses `()` for the decode and validate args.
pub trait TryFromAccountsWithArgs<'a, D, V>:
    AccountSetDecode<'a, D> + AccountSetValidate<V>
{
    fn try_from_accounts_with_args(
        accounts: &mut &'a [AccountInfo],
        decode: D,
        validate: V,
        ctx: &mut Context,
    ) -> Result<Self> {
        let mut set = Self::decode_accounts(accounts, decode, ctx)?;
        set.validate_accounts(validate, ctx)?;
        Ok(set)
    }

    fn try_from_account_with_args(
        account: &'a AccountInfo,
        decode: D,
        validate: V,
        ctx: &mut Context,
    ) -> Result<Self>
    where
        Self: SingleAccountSet,
    {
        let accounts = &mut slice::from_ref(account);
        Self::try_from_accounts_with_args(accounts, decode, validate, ctx)
    }
}

/// Additional convenience methods around [`TryFromAccountsWithArgs`] for when the [`AccountSetDecode`] and [`AccountSetValidate`] args are `()`.
pub trait TryFromAccounts<'a>: TryFromAccountsWithArgs<'a, (), ()> {
    fn try_from_accounts(accounts: &mut &'a [AccountInfo], ctx: &mut Context) -> Result<Self> {
        Self::try_from_accounts_with_args(accounts, (), (), ctx)
    }

    fn try_from_account(account: &'a AccountInfo, ctx: &mut Context) -> Result<Self>
    where
        Self: SingleAccountSet,
    {
        Self::try_from_account_with_args(account, (), (), ctx)
    }
}

impl<'a, T, D, V> TryFromAccountsWithArgs<'a, D, V> for T where
    T: AccountSetDecode<'a, D> + AccountSetValidate<V>
{
}

impl<'a, T> TryFromAccounts<'a> for T where T: TryFromAccountsWithArgs<'a, (), ()> {}

/// An [`AccountSet`] that can be decoded from a list of [`AccountInfo`]s using arg `A`.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetDecode<'a, A>: Sized {
    /// Decode the accounts from `accounts` using `decode_input`.
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: A,
        ctx: &mut Context,
    ) -> Result<Self>;
}

/// An [`AccountSet`] that can be validated using arg `A`.
/// Evaluate wrapping as inner before outer.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetValidate<A> {
    /// Validate the accounts using `validate_input`.
    fn validate_accounts(&mut self, validate_input: A, ctx: &mut Context) -> Result<()>;
}

/// An [`AccountSet`] that can be cleaned up using arg `A`.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetCleanup<A> {
    /// Clean up the accounts using `cleanup_input`.
    fn cleanup_accounts(&mut self, cleanup_input: A, ctx: &mut Context) -> Result<()>;
}

/// Trait for checking if the key matches the expected key.
pub trait CheckKey {
    /// Checks if the key matches the expected key.
    fn check_key(&self, key: &Pubkey) -> Result<()>;
}

static_assertions::assert_obj_safe!(CanCloseAccount, CanAddLamports, CanFundRent);

pub trait CanCloseAccount {
    /// Gets the account info of the account to close.
    fn account_to_close(&self) -> AccountInfo;
    /// Closes the account by zeroing the lamports and replacing the discriminant with all `u8::MAX`,
    /// reallocating down to size.
    fn close(&self, recipient: &(impl CanAddLamports + ?Sized)) -> Result<()>
    where
        Self: HasOwnerProgram,
        Self: Sized,
    {
        let info = self.account_to_close();
        info.realloc(
            size_of::<<Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        info.account_data_mut()?.fill(u8::MAX);
        recipient.add_lamports(info.lamports())?;
        *info.try_borrow_mut_lamports()? = 0;
        Ok(())
    }

    /// Closes the account by reallocating to zero and assigning to the System program.
    /// This is the same as calling `close` but not abusable and harder for indexer detection.
    ///
    /// It also happens to be unsound because [`AccountInfo::assign`] is unsound.
    fn close_full(&self, recipient: &dyn CanAddLamports) -> Result<()> {
        let info = self.account_to_close();
        recipient.add_lamports(info.lamports())?;
        *info.try_borrow_mut_lamports()? = 0;
        info.realloc(0, false)?;
        unsafe { info.assign(System::ID.as_array()) }; // TODO: Fix safety
        Ok(())
    }
}

pub trait CanAddLamports: Debug {
    fn account_to_modify(&self) -> AccountInfo;
    fn add_lamports(&self, lamports: u64) -> Result<()> {
        *self.account_to_modify().try_borrow_mut_lamports()? += lamports;
        Ok(())
    }
}
/// Indicates that this account can fund rent on another account, and potentially be used to create an account.
pub trait CanFundRent: CanAddLamports {
    /// Whether [`Self::account_to_modify`](`CanAddLamports::account_to_modify`) can be used as the funder for a [`crate::program::system::CreateAccount`] CPI.
    fn can_create_account(&self) -> bool;
    /// Increases the rent of the recipient by `lamports`.
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet,
        lamports: u64,
        ctx: &Context,
    ) -> Result<()>;

    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        None
    }
}

pub trait CanModifyRent {
    fn account_to_modify(&self) -> AccountInfo;

    /// Normalizes the rent of an account if data size is changed.
    /// Assumes `Self` is mutable and owned by this program.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn normalize_rent(&self, funder: &(impl CanFundRent + ?Sized), ctx: &Context) -> Result<()> {
        let account = self.account_to_modify();
        let rent = ctx.get_rent()?;
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
                CanFundRent::fund_rent(funder, &account, transfer_amount, ctx)?;
                Ok(())
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                *account.try_borrow_mut_lamports()? -= transfer_amount;
                funder.add_lamports(transfer_amount)?;
                Ok(())
            }
        }
    }

    /// Refunds rent to the funder so long as the account has more than the minimum rent.
    /// Assumes `Self` is owned by this program and is mutable.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn refund_rent(&self, recipient: &(impl CanAddLamports + ?Sized), ctx: &Context) -> Result<()> {
        let account = self.account_to_modify();
        let rent = ctx.get_rent()?;
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
                recipient.add_lamports(transfer_amount)?;
                Ok(())
            }
        }
    }

    /// Receive rent to self to be at least the minimum rent. This will not normalize down excess lamports.
    /// Assumes `Self` is owned by this program and is mutable.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn receive_rent(&self, funder: &(impl CanFundRent + ?Sized), ctx: &Context) -> Result<()> {
        let account = self.account_to_modify();
        let rent = ctx.get_rent()?;
        let lamports = *account.try_borrow_lamports()?;
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

    /// Emits a warning message if the account has more lamports than required by rent.
    #[cfg_attr(not(feature = "cleanup_rent_warning"), allow(unused_variables))]
    fn check_cleanup(&self, ctx: &Context) -> Result<()> {
        #[cfg(feature = "cleanup_rent_warning")]
        {
            use std::cmp::Ordering;
            let account = self.account_to_modify();
            if account.is_writable() {
                let rent = ctx.get_rent()?;
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
        ctx: &Context,
    ) -> Result<()> {
        let account = self.account_to_create();
        if account.owner_pubkey() != System::ID {
            bail!(ProgramError::InvalidAccountOwner);
        }
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
                &system::CreateAccount {
                    lamports: exempt_lamports,
                    space: space as u64,
                    owner,
                },
                system::CreateAccountCpiAccounts {
                    funder: funder.account_to_modify(),
                    new_account: account,
                },
                ctx,
            )?
            .invoke_signed(seeds)
            .context("System::CreateAccount CPI failed")?;
        } else {
            let required_lamports = exempt_lamports.saturating_sub(current_lamports).max(1);
            if required_lamports > 0 {
                CanFundRent::fund_rent(funder, &account, required_lamports, ctx)
                    .context("Failed to fund rent")?;
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
                ctx,
            )?
            .invoke_signed(account_seeds)
            .context("System::Allocate CPI failed")?;
            System::cpi(
                &system::Assign { owner },
                system::AssignCpiAccounts { account },
                ctx,
            )?
            .invoke_signed(account_seeds)
            .context("System::Assign CPI failed")?;
        }
        Ok(())
    }
}

#[doc(hidden)]
pub(crate) mod internal_reverse {
    use super::*;

    #[inline]
    pub fn _account_set_validate_reverse<T, A>(
        validate_input: A,
        this: &mut T,
        ctx: &mut Context,
    ) -> Result<()>
    where
        T: AccountSetValidate<A>,
    {
        this.validate_accounts(validate_input, ctx)
    }

    #[inline]
    pub fn _account_set_cleanup_reverse<T, A>(
        cleanup_input: A,
        this: &mut T,
        ctx: &mut Context,
    ) -> Result<()>
    where
        T: AccountSetCleanup<A>,
    {
        this.cleanup_accounts(cleanup_input, ctx)
    }
}

#[cfg(test)]
mod test {
    use crate::{account_set::AccountSetValidate, prelude::Context};
    use star_frame_proc::AccountSet;

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>, extra_validation = { arg.push(N); Ok(()) })]
    struct InnerAccount<const N: usize>;

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet123 {
        #[validate(arg = &mut *arg)]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg)]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet213 {
        #[validate(arg = &mut *arg, requires = [b])]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg)]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet312 {
        #[validate(arg = &mut *arg, requires = [c])]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg, requires = [c])]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet231 {
        #[validate(arg = &mut *arg, requires = [c])]
        a: InnerAccount<1>,
        #[validate(arg = &mut *arg)]
        b: InnerAccount<2>,
        #[validate(arg = &mut *arg)]
        c: InnerAccount<3>,
    }

    #[test]
    fn test_validate() {
        let mut vec = Vec::new();
        let mut ctx = Context::default();
        let mut set = AccountSet123 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![1, 2, 3]);

        vec.clear();
        let mut set = AccountSet213 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![2, 1, 3]);

        vec.clear();
        let mut set = AccountSet312 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![3, 1, 2]);

        vec.clear();
        let mut set = AccountSet231 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut ctx).unwrap();
        assert_eq!(vec, vec![2, 3, 1]);
    }
}
