use crate::account_set::{AccountSetDecode, SingleAccountSet};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use advance::AdvanceArray;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use star_frame::account_set::{AccountSet, AccountSetCleanup, AccountSetValidate};
use std::cell::{Ref, RefMut};

impl<'info> AccountSet<'info> for AccountInfo<'info> {}
impl<'__a, 'info> AccountSet<'info> for &'__a AccountInfo<'info> {}
impl<'info> SingleAccountSet<'info> for AccountInfo<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self
    }

    fn account_meta(&self) -> AccountMeta {
        AccountMeta {
            pubkey: *self.key,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }

    fn is_signer(&self) -> bool {
        self.is_signer
    }

    fn is_writable(&self) -> bool {
        self.is_writable
    }

    fn key(&self) -> &'info Pubkey {
        self.key
    }

    fn owner(&self) -> &'info Pubkey {
        self.owner
    }

    fn info_data_bytes<'a>(&'a self) -> Result<Ref<'a, [u8]>>
    where
        'info: 'a,
    {
        self.try_borrow_data()
            .map(|d| Ref::map(d, |d| &**d))
            .map_err(Into::into)
    }

    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, &'info mut [u8]>>
    where
        'info: 'a,
    {
        self.try_borrow_mut_data().map_err(Into::into)
    }
}
impl<'__a, 'info> SingleAccountSet<'info> for &'__a AccountInfo<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        self
    }

    fn account_meta(&self) -> AccountMeta {
        AccountMeta {
            pubkey: *self.key,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
        }
    }

    fn is_signer(&self) -> bool {
        self.is_signer
    }

    fn is_writable(&self) -> bool {
        self.is_writable
    }

    fn key(&self) -> &'info Pubkey {
        self.key
    }

    fn owner(&self) -> &'info Pubkey {
        self.owner
    }

    fn info_data_bytes<'a>(&'a self) -> Result<Ref<'a, [u8]>>
    where
        'info: 'a,
    {
        self.try_borrow_data()
            .map(|d| Ref::map(d, |d| &**d))
            .map_err(Into::into)
    }

    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, &'info mut [u8]>>
    where
        'info: 'a,
    {
        self.try_borrow_mut_data().map_err(Into::into)
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

impl<'info> AccountSetValidate<'info, &Pubkey> for AccountInfo<'info> {
    fn validate_accounts(
        &mut self,
        validate_input: &Pubkey,
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        anyhow::ensure!(self.key == validate_input);
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

impl<'a, 'info> AccountSetValidate<'info, &Pubkey> for &'a AccountInfo<'info> {
    fn validate_accounts(
        &mut self,
        validate_input: &Pubkey,
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        anyhow::ensure!(self.key == validate_input);
        Ok(())
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
#[cfg(feature = "idl")]
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
    impl<'info> AccountSetToIdl<'info, Pubkey> for AccountInfo<'info> {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: Pubkey,
        ) -> Result<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                address: Some(arg),
                ..Default::default()
            }))
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
