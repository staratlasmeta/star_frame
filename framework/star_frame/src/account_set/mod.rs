pub mod data_account;
pub mod init_account;
pub mod mutable;
pub mod program;
pub mod rest;
pub mod seeded_account;
pub mod signer;
pub mod system_account;

pub use star_frame_proc::AccountSet;
pub use star_frame_proc::AccountToIdl;

use crate::sys_calls::SysCallInvoke;
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
    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes_mut()
    }
}

/// An [`AccountSet`] that can be decoded from a list of [`AccountInfo`]s using arg `A`.
pub trait AccountSetDecode<'a, 'info, A>: AccountSet<'info> + Sized {
    /// Decode the accounts from `accounts` using `decode_input`.
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self>;
}

/// An [`AccountSet`] that can be validated using arg `A`.
/// Evaluate wrapping as inner before outer.
pub trait AccountSetValidate<'info, A>: AccountSet<'info> + Sized {
    /// Validate the accounts using `validate_input`.
    fn validate_accounts(
        &mut self,
        validate_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()>;
}

/// An [`AccountSet`] that can be cleaned up using arg `A`.
pub trait AccountSetCleanup<'info, A>: AccountSet<'info> + Sized {
    /// Clean up the accounts using `cleanup_input`.
    fn cleanup_accounts(
        &mut self,
        cleanup_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()>;
}
