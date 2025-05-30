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

use crate::prelude::{PackedValue, StarFrameProgram};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use bytemuck::{bytes_of, from_bytes};
use solana_program::account_info::AccountInfo;
use std::slice;

/// A set of accounts that can be used as input to an instruction.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSet<'info> {}

pub trait ProgramAccount: HasOwnerProgram {
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant;
    #[must_use]
    fn discriminant_bytes() -> Vec<u8> {
        bytes_of(&Self::DISCRIMINANT).into()
    }

    fn validate_account_info<'info>(info: &impl SingleAccountSet<'info>) -> Result<()> {
        if info.owner() != &Self::OwnerProgram::ID {
            bail!(
                "Account {} owner {} does not match expected program ID {}",
                info.key(),
                info.owner(),
                Self::OwnerProgram::ID
            );
        }
        let data = info.info_data_bytes()?;
        if data.len() < size_of::<OwnerProgramDiscriminant<Self>>()
            || from_bytes::<PackedValue<OwnerProgramDiscriminant<Self>>>(
                &data[..size_of::<OwnerProgramDiscriminant<Self>>()],
            ) != &Self::DISCRIMINANT
        {
            bail!(
                "Account {} data does not match expected discriminant for program {}",
                info.key(),
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
pub trait TryFromAccountsWithArgs<'a, 'info, D, V>:
    AccountSetDecode<'a, 'info, D> + AccountSetValidate<'info, V>
{
    fn try_from_accounts_with_args(
        accounts: &mut &'a [AccountInfo<'info>],
        decode: D,
        validate: V,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        // SAFETY: We are calling .validate_accounts() immediately after decoding
        let mut set = unsafe { Self::decode_accounts(accounts, decode, syscalls)? };
        set.validate_accounts(validate, syscalls)?;
        Ok(set)
    }

    fn try_from_account_with_args(
        account: &'a AccountInfo<'info>,
        decode: D,
        validate: V,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self>
    where
        Self: SingleAccountSet<'info>,
    {
        let accounts = &mut slice::from_ref(account);
        Self::try_from_accounts_with_args(accounts, decode, validate, syscalls)
    }
}

/// Additional convenience methods around [`TryFromAccountsWithArgs`] for when the [`AccountSetDecode`] and [`AccountSetValidate`] args are `()`.
pub trait TryFromAccounts<'a, 'info>: TryFromAccountsWithArgs<'a, 'info, (), ()> {
    fn try_from_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        Self::try_from_accounts_with_args(accounts, (), (), syscalls)
    }

    fn try_from_account(
        account: &'a AccountInfo<'info>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self>
    where
        Self: SingleAccountSet<'info>,
    {
        Self::try_from_account_with_args(account, (), (), syscalls)
    }
}

impl<'a, 'info, T, D, V> TryFromAccountsWithArgs<'a, 'info, D, V> for T where
    T: AccountSetDecode<'a, 'info, D> + AccountSetValidate<'info, V>
{
}

impl<'a, 'info, T> TryFromAccounts<'a, 'info> for T where
    T: TryFromAccountsWithArgs<'a, 'info, (), ()>
{
}

/// An [`AccountSet`] that can be decoded from a list of [`AccountInfo`]s using arg `A`.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetDecode<'a, 'info, A>: AccountSet<'info> + Sized {
    /// Decode the accounts from `accounts` using `decode_input`.
    ///
    /// # Safety
    /// The output has not been validated. Calls to this function should be followed by a call to [`AccountSetValidate::validate_accounts`], if applicable.
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self>;
}

/// An [`AccountSet`] that can be validated using arg `A`.
/// Evaluate wrapping as inner before outer.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetValidate<'info, A>: AccountSet<'info> + Sized {
    /// Validate the accounts using `validate_input`.
    fn validate_accounts(
        &mut self,
        validate_input: A,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()>;
}

/// An [`AccountSet`] that can be cleaned up using arg `A`.
///
/// Derivable via [`derive@AccountSet`].
pub trait AccountSetCleanup<'info, A>: AccountSet<'info> + Sized {
    /// Clean up the accounts using `cleanup_input`.
    fn cleanup_accounts(
        &mut self,
        cleanup_input: A,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()>;
}

#[doc(hidden)]
pub(crate) mod internal_reverse {
    use super::*;

    #[inline]
    pub fn _account_set_validate_reverse<'info, T, A>(
        validate_input: A,
        this: &mut T,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()>
    where
        T: AccountSetValidate<'info, A>,
    {
        this.validate_accounts(validate_input, syscalls)
    }

    #[inline]
    pub fn _account_set_cleanup_reverse<'info, T, A>(
        cleanup_input: A,
        this: &mut T,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()>
    where
        T: AccountSetCleanup<'info, A>,
    {
        this.cleanup_accounts(cleanup_input, syscalls)
    }
}

#[cfg(test)]
mod test {
    use crate::account_set::AccountSetValidate;
    use crate::syscalls::{SyscallCore, SyscallInvoke};
    use crate::Result;
    use crate::SolanaInstruction;
    use solana_program::account_info::AccountInfo;
    use solana_program::clock::Clock;
    use solana_program::pubkey::Pubkey;
    use solana_program::rent::Rent;
    use star_frame::syscalls::SyscallAccountCache;
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
    impl SyscallCore for DummyRuntime {
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

    impl SyscallAccountCache<'_> for DummyRuntime {}
    impl SyscallInvoke<'_> for DummyRuntime {
        fn invoke(
            &self,
            _instruction: &SolanaInstruction,
            _accounts: &[AccountInfo],
        ) -> Result<()> {
            unimplemented!()
        }

        fn invoke_signed(
            &self,
            _instruction: &SolanaInstruction,
            _accounts: &[AccountInfo],
            _signers_seeds: &[&[&[u8]]],
        ) -> Result<()> {
            unimplemented!()
        }
    }

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
