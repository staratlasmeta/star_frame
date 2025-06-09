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
use anyhow::bail;
pub use impls::*;
pub use modifiers::*;
pub use program::*;
pub use rest::*;
pub use single_set::*;
pub use star_frame_proc::AccountSet;
pub use system_account::*;
pub use sysvar::*;
pub use validated_account::*;

use crate::prelude::{Context, PackedValue, StarFrameProgram};
use crate::Result;
use bytemuck::{bytes_of, from_bytes};
use pinocchio::account_info::AccountInfo;
use std::slice;

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
        ctx: &mut impl Context,
    ) -> Result<Self> {
        // SAFETY: We are calling .validate_accounts() immediately after decoding
        let mut set = unsafe { Self::decode_accounts(accounts, decode, ctx)? };
        set.validate_accounts(validate, ctx)?;
        Ok(set)
    }

    fn try_from_account_with_args(
        account: &'a AccountInfo,
        decode: D,
        validate: V,
        ctx: &mut impl Context,
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
    fn try_from_accounts(accounts: &mut &'a [AccountInfo], ctx: &mut impl Context) -> Result<Self> {
        Self::try_from_accounts_with_args(accounts, (), (), ctx)
    }

    fn try_from_account(account: &'a AccountInfo, ctx: &mut impl Context) -> Result<Self>
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
    ///
    /// # Safety
    /// The output has not been validated. Calls to this function should be followed by a call to [`AccountSetValidate::validate_accounts`], if applicable.
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: A,
        ctx: &mut impl Context,
    ) -> Result<Self>;
}

/// An [`AccountSet`] that can be validated using arg `A`.
/// Evaluate wrapping as inner before outer.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetValidate<A> {
    /// Validate the accounts using `validate_input`.
    fn validate_accounts(&mut self, validate_input: A, ctx: &mut impl Context) -> Result<()>;
}

/// An [`AccountSet`] that can be cleaned up using arg `A`.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetCleanup<A> {
    /// Clean up the accounts using `cleanup_input`.
    fn cleanup_accounts(&mut self, cleanup_input: A, ctx: &mut impl Context) -> Result<()>;
}

#[doc(hidden)]
pub(crate) mod internal_reverse {
    use super::*;

    #[inline]
    pub fn _account_set_validate_reverse<T, A>(
        validate_input: A,
        this: &mut T,
        ctx: &mut impl Context,
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
        ctx: &mut impl Context,
    ) -> Result<()>
    where
        T: AccountSetCleanup<A>,
    {
        this.cleanup_accounts(cleanup_input, ctx)
    }
}

#[cfg(test)]
mod test {
    use crate::account_set::AccountSetValidate;
    use crate::context::ContextCore;
    use crate::Result;
    use pinocchio::sysvars::clock::Clock;
    use pinocchio::sysvars::rent::Rent;
    use solana_pubkey::Pubkey;
    use star_frame::context::ContextAccountCache;
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

    struct DummyRuntime;
    impl ContextCore for DummyRuntime {
        fn current_program_id(&self) -> &Pubkey {
            unimplemented!()
        }

        fn get_rent(&self) -> Result<Rent> {
            unimplemented!()
        }

        fn get_clock(&self) -> Result<Clock> {
            unimplemented!()
        }
    }

    impl ContextAccountCache for DummyRuntime {}

    #[test]
    fn test_validate() {
        let mut vec = Vec::new();
        let mut set = AccountSet123 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut DummyRuntime).unwrap();
        assert_eq!(vec, vec![1, 2, 3]);

        vec.clear();
        let mut set = AccountSet213 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut DummyRuntime).unwrap();
        assert_eq!(vec, vec![2, 1, 3]);

        vec.clear();
        let mut set = AccountSet312 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut DummyRuntime).unwrap();
        assert_eq!(vec, vec![3, 1, 2]);

        vec.clear();
        let mut set = AccountSet231 {
            a: InnerAccount::<1>,
            b: InnerAccount::<2>,
            c: InnerAccount::<3>,
        };
        set.validate_accounts(&mut vec, &mut DummyRuntime).unwrap();
        assert_eq!(vec, vec![2, 3, 1]);
    }
}
