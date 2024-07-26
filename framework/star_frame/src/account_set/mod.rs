mod data_account;
mod impls;
mod init_account;
mod mutable;
mod program;
mod rest;
mod seeded_account;
mod seeded_data_account;
mod seeded_init_account;
mod signer;
mod system_account;

pub use data_account::*;
pub use impls::*;
pub use init_account::*;
pub use mutable::*;
pub use program::*;
pub use rest::*;
pub use seeded_account::*;
pub use seeded_data_account::*;
pub use seeded_init_account::*;
pub use signer::*;
pub use star_frame_proc::AccountSet;
pub use star_frame_proc::AccountToIdl;
pub use system_account::*;


use crate::syscalls::SyscallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::cell::{Ref, RefMut};
use std::convert::Infallible;

/// A set of accounts that can be used as input to an instruction.
pub trait AccountSet<'info> {
    fn try_to_accounts<'a, E>(
        &'a self,
        add_account: impl FnMut(&'a AccountInfo<'info>) -> Result<(), E>,
    ) -> Result<(), E>
    where
        'info: 'a;

    /// Add all the accounts in this set using `add_account`.
    fn to_accounts<'a>(&'a self, mut add_account: impl FnMut(&'a AccountInfo<'info>))
    where
        'info: 'a,
    {
        self.try_to_accounts::<Infallible>(|a| {
            add_account(a);
            Ok(())
        })
        .unwrap();
    }

    /// Gets a vector of all the accounts in this set.
    fn to_accounts_vec<'a>(&'a self) -> Vec<&'a AccountInfo<'info>> {
        let mut out = Vec::new();
        self.to_accounts(|acc| out.push(acc));
        out
    }

    /// Add all accounts in this set using `add_account_meta`.
    fn to_account_metas(&self, add_account_meta: impl FnMut(AccountMeta));

    /// Gets a vector of all the account metas in this set.
    fn to_account_metas_vec(&self) -> Vec<AccountMeta> {
        let mut out = Vec::new();
        self.to_account_metas(|acc| out.push(acc));
        out
    }
}

/// An [`AccountSet`] that contains exactly 1 account.
pub trait SingleAccountSet<'info>: AccountSet<'info> {
    /// Gets the contained account.
    fn account_info(&self) -> &AccountInfo<'info>;
    /// Gets the contained account cloned.
    fn account_info_cloned(&self) -> AccountInfo<'info> {
        self.account_info().clone()
    }
    /// Gets the account meta of the contained account.
    fn account_meta(&self) -> AccountMeta {
        let info = self.account_info();
        AccountMeta {
            pubkey: *info.key(),
            is_signer: info.is_signer(),
            is_writable: info.is_writable(),
        }
    }

    /// Gets whether this account signed.
    fn is_signer(&self) -> bool {
        self.account_info().is_signer()
    }
    /// Gets whether this account is writable.
    fn is_writable(&self) -> bool {
        self.account_info().is_writable()
    }

    /// Gets the key of the contained account.
    fn key(&self) -> &'info Pubkey {
        self.account_info().key()
    }
    /// Gets the owner of the contained account.
    fn owner(&self) -> &'info Pubkey {
        self.account_info().owner()
    }

    /// Gets the data of the contained account immutably.
    fn info_data_bytes<'a>(&'a self) -> Result<Ref<'a, [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes()
    }
    /// Gets the data of the contained account mutably.
    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, &'info mut [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes_mut()
    }
}

/// Indicates the underlying account is a signer.
pub trait SignedAccount<'info>: SingleAccountSet<'info> {
    /// Gets the seeds of the account if it is seeded.
    fn signer_seeds(&self) -> Option<Vec<&[u8]>>;
}

/// A marker trait that indicates the underlying account is writable.
pub trait WritableAccount<'info>: SingleAccountSet<'info> {}

/// A marker trait that indicates the underlying type has a [`ProgramAccount`] in it.
pub trait HasProgramAccount<'info>: SingleAccountSet<'info> {
    type ProgramAccount: ProgramAccount + ?Sized;
}

/// An [`AccountSet`] that can be decoded from a list of [`AccountInfo`]s using arg `A`.
pub trait AccountSetDecode<'a, 'info, A>: AccountSet<'info> + Sized {
    /// Decode the accounts from `accounts` using `decode_input`.
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<Self>;
}

/// An [`AccountSet`] that can be validated using arg `A`.
/// Evaluate wrapping as inner before outer.
pub trait AccountSetValidate<'info, A>: AccountSet<'info> + Sized {
    /// Validate the accounts using `validate_input`.
    fn validate_accounts(
        &mut self,
        validate_input: A,
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()>;
}

/// An [`AccountSet`] that can be cleaned up using arg `A`.
pub trait AccountSetCleanup<'info, A>: AccountSet<'info> + Sized {
    /// Clean up the accounts using `cleanup_input`.
    fn cleanup_accounts(
        &mut self,
        cleanup_input: A,
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()>;
}

#[cfg(test)]
mod test {
    use crate::account_set::AccountSetValidate;
    use crate::syscalls::{SyscallCore, SyscallInvoke};
    use crate::SolanaInstruction;
    use solana_program::account_info::AccountInfo;
    use solana_program::clock::Clock;
    use solana_program::entrypoint_deprecated::ProgramResult;
    use solana_program::program_error::ProgramError;
    use solana_program::pubkey::Pubkey;
    use solana_program::rent::Rent;
    use star_frame_proc::AccountSet;

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>, extra_validation = { arg.push(N); Ok(()) })]
    struct InnerAccount<const N: usize>;

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet123 {
        #[validate(arg = arg)]
        a: InnerAccount<1>,
        #[validate(arg = arg)]
        b: InnerAccount<2>,
        #[validate(arg = arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet213 {
        #[validate(arg = arg, requires = [b])]
        a: InnerAccount<1>,
        #[validate(arg = arg)]
        b: InnerAccount<2>,
        #[validate(arg = arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet312 {
        #[validate(arg = arg, requires = [c])]
        a: InnerAccount<1>,
        #[validate(arg = arg, requires = [c])]
        b: InnerAccount<2>,
        #[validate(arg = arg)]
        c: InnerAccount<3>,
    }

    #[derive(AccountSet)]
    #[validate(arg = &mut Vec<usize>)]
    struct AccountSet231 {
        #[validate(arg = arg, requires = [c])]
        a: InnerAccount<1>,
        #[validate(arg = arg)]
        b: InnerAccount<2>,
        #[validate(arg = arg)]
        c: InnerAccount<3>,
    }

    struct DummyRuntime;
    impl SyscallCore for DummyRuntime {
        fn current_program_id(&self) -> &Pubkey {
            unimplemented!()
        }

        fn get_rent(&mut self) -> Result<Rent, ProgramError> {
            unimplemented!()
        }

        fn get_clock(&mut self) -> Result<Clock, ProgramError> {
            unimplemented!()
        }
    }
    impl SyscallInvoke for DummyRuntime {
        fn invoke(
            &mut self,
            _instruction: &SolanaInstruction,
            _accounts: &[AccountInfo],
        ) -> ProgramResult {
            unimplemented!()
        }

        unsafe fn invoke_unchecked(
            &mut self,
            _instruction: &SolanaInstruction,
            _accounts: &[AccountInfo],
        ) -> ProgramResult {
            unimplemented!()
        }

        fn invoke_signed(
            &mut self,
            _instruction: &SolanaInstruction,
            _accounts: &[AccountInfo],
            _signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
            unimplemented!()
        }

        unsafe fn invoke_signed_unchecked(
            &mut self,
            _instruction: &SolanaInstruction,
            _accounts: &[AccountInfo],
            _signers_seeds: &[&[&[u8]]],
        ) -> ProgramResult {
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
