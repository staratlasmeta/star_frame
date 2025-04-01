use crate::account_set::{AccountSetDecode, SingleAccountSet, SingleSetMeta};
use crate::client::ClientAccountSet;
use crate::prelude::{CpiAccountSet, SyscallAccountCache};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use advancer::AdvanceArray;
use anyhow::Context;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::{AccountSet, AccountSetCleanup, AccountSetValidate};
use std::cell::{Ref, RefMut};

impl<'info> AccountSet<'info> for AccountInfo<'info> {
    #[inline]
    fn set_account_cache(&mut self, _syscalls: &mut impl SyscallAccountCache<'info>) {}
}
impl<'__a, 'info> AccountSet<'info> for &'__a AccountInfo<'info> {
    #[inline]
    fn set_account_cache(&mut self, _syscalls: &mut impl SyscallAccountCache<'info>) {}
}
impl<'info> SingleAccountSet<'info> for AccountInfo<'info> {
    const META: SingleSetMeta = SingleSetMeta::default();
    #[inline]
    fn account_info(&self) -> &AccountInfo<'info> {
        self
    }
    #[inline]
    fn account_meta(&self) -> AccountMeta {
        AccountMeta {
            pubkey: *self.key,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }
    #[inline]
    fn is_signer(&self) -> bool {
        self.is_signer
    }
    #[inline]
    fn is_writable(&self) -> bool {
        self.is_writable
    }
    #[inline]
    fn key(&self) -> &'info Pubkey {
        self.key
    }
    #[inline]
    fn owner(&self) -> &'info Pubkey {
        self.owner
    }
    #[inline]
    fn info_data_bytes<'a>(&'a self) -> Result<Ref<'a, [u8]>>
    where
        'info: 'a,
    {
        self.data
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)
            .map(|d| Ref::map(d, |d| &**d))
            .with_context(|| format!("Error borrowing data on account {}", self.key))
    }
    #[inline]
    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, &'info mut [u8]>>
    where
        'info: 'a,
    {
        self.data
            .try_borrow_mut()
            .map_err(|_| ProgramError::AccountBorrowFailed)
            .with_context(|| format!("Error borrowing mut data on account {}", self.key))
    }
}

impl<'a, 'info> ClientAccountSet for &'a AccountInfo<'info> {
    type ClientAccounts = Pubkey;
    const MIN_LEN: usize = 1;

    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta::new(*accounts, false));
    }
}

impl<'info> ClientAccountSet for AccountInfo<'info> {
    type ClientAccounts = Pubkey;
    const MIN_LEN: usize = 1;

    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta::new(*accounts, false));
    }
}

impl<'a, 'info> CpiAccountSet<'info> for &'a AccountInfo<'info> {
    type CpiAccounts = AccountInfo<'info>;
    const MIN_LEN: usize = 1;

    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.account_info_cloned()
    }

    fn extend_account_infos(accounts: Self::CpiAccounts, infos: &mut Vec<AccountInfo<'info>>) {
        infos.push(accounts);
    }

    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta::new(*accounts.key, false));
    }
}

impl<'info> CpiAccountSet<'info> for AccountInfo<'info> {
    type CpiAccounts = AccountInfo<'info>;
    const MIN_LEN: usize = 1;

    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.account_info_cloned()
    }

    fn extend_account_infos(accounts: Self::CpiAccounts, infos: &mut Vec<AccountInfo<'info>>) {
        infos.push(accounts);
    }

    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta::new(*accounts.key, false));
    }
}

impl<'__a, 'info> SingleAccountSet<'info> for &'__a AccountInfo<'info> {
    const META: SingleSetMeta = SingleSetMeta::default();
    #[inline]
    fn account_info(&self) -> &AccountInfo<'info> {
        self
    }
}
impl<'a, 'info> AccountSetDecode<'a, 'info, ()> for AccountInfo<'info> {
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        _decode_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts.try_advance_array()?;
        Ok(account[0].clone())
    }
}
impl<'a, 'info> AccountSetDecode<'a, 'info, ()> for &'a AccountInfo<'info> {
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        _decode_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts.try_advance_array()?;
        Ok(&account[0])
    }
}
impl<'info> AccountSetValidate<'info, ()> for AccountInfo<'info> {
    fn validate_accounts(
        &mut self,
        _validate_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        Ok(())
    }
}

impl<'a, 'info> AccountSetValidate<'info, ()> for &'a AccountInfo<'info> {
    fn validate_accounts(
        &mut self,
        validate_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        Ok(validate_input)
    }
}

impl<'info> AccountSetCleanup<'info, ()> for AccountInfo<'info> {
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        Ok(cleanup_input)
    }
}
impl<'a, 'info> AccountSetCleanup<'info, ()> for &'a AccountInfo<'info> {
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        Ok(cleanup_input)
    }
}
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl_impl {
    use super::*;
    use crate::idl::{AccountSetToIdl, FindIdlSeeds};
    use crate::prelude::Seeds;
    use star_frame_idl::account_set::{IdlAccountSetDef, IdlSingleAccountSet};
    use star_frame_idl::seeds::IdlFindSeeds;
    use star_frame_idl::IdlDefinition;

    impl<'info> AccountSetToIdl<'info, ()> for AccountInfo<'info> {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            _arg: (),
        ) -> Result<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet::default()))
        }
    }

    impl<'info, T> AccountSetToIdl<'info, Seeds<(T, Pubkey)>> for AccountInfo<'info>
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: Seeds<(T, Pubkey)>,
        ) -> Result<IdlAccountSetDef> {
            let (seeds, program) = arg.0;
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                seeds: Some(IdlFindSeeds {
                    seeds: T::find_seeds(&seeds)?,
                    program: Some(program),
                }),
                ..Default::default()
            }))
        }
    }

    impl<'info, T> AccountSetToIdl<'info, Seeds<T>> for AccountInfo<'info>
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: Seeds<T>,
        ) -> Result<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                seeds: Some(IdlFindSeeds {
                    seeds: T::find_seeds(&arg.0)?,
                    program: None,
                }),
                ..Default::default()
            }))
        }
    }

    impl<'a, 'info, A> AccountSetToIdl<'info, A> for &'a AccountInfo<'info>
    where
        AccountInfo<'info>: AccountSetToIdl<'info, A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            AccountInfo::account_set_to_idl(idl_definition, arg)
        }
    }
}
