use crate::account_set::{AccountSet, SingleAccountSet};
use crate::anchor_replacement::{AnchorCleanupArgs, ANCHOR_CLOSED_ACCOUNT_DISCRIMINATOR};
use crate::program_account::ProgramAccount;
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use bytemuck::{from_bytes, from_bytes_mut, Pod};
use derivative::Derivative;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use star_frame::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use star_frame::anchor_replacement::AnchorValidateArgs;
use star_frame::program::StarFrameProgram;
use star_frame::sys_calls::SysCallCore;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct AccountLoader<'info, T>
where
    T: Pod + ProgramAccount,
    T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
{
    info: AccountInfo<'info>,
    data: PhantomData<&'info T>,
}

impl<'info, T> AccountLoader<'info, T>
where
    T: Pod + ProgramAccount,
    T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
{
    pub fn new(acc_info: AccountInfo<'info>) -> Self {
        Self {
            info: acc_info,
            data: PhantomData,
        }
    }

    pub fn try_from(acc_info: &AccountInfo<'info>, runtime: impl SysCallCore) -> Result<Self> {
        if acc_info.owner != T::owner_program_id().find_network(runtime.current_network())? {
            return Err(ProgramError::IncorrectProgramId);
        }
        Self::check_data(&acc_info.info_data_bytes()?)?;

        Ok(Self::new(acc_info.clone()))
    }

    fn check_data(data: &impl Deref<Target = [u8]>) -> Result<()> {
        let data = data.as_ref();
        if data.len() < size_of::<T>() + 8 {
            Err(ProgramError::InvalidAccountData)
        } else {
            let discriminant: [u8; 8] = data[..8].try_into().unwrap();
            if discriminant != T::discriminant() {
                Err(ProgramError::InvalidAccountData)
            } else {
                Ok(())
            }
        }
    }

    pub fn load(&self) -> Result<Ref<T>> {
        let r = self.info.info_data_bytes()?;
        Self::check_data(&r)?;
        Ok(Ref::map(r, |r| from_bytes(&r[8..][..size_of::<T>()])))
    }

    pub fn load_mut(&mut self) -> Result<RefMut<T>> {
        let r = self.info.info_data_bytes_mut()?;
        Self::check_data(&r)?;
        Ok(RefMut::map(r, |r| {
            from_bytes_mut(&mut r[8..][..size_of::<T>()])
        }))
    }

    pub fn load_init(&self) -> Result<RefMut<T>> {
        let mut r = self.info.info_data_bytes_mut()?;
        if r.len() < size_of::<T>() + 8 {
            return Err(ProgramError::InvalidAccountData);
        }
        let discriminant: [u8; 8] = r[..8].try_into().unwrap();
        if discriminant != [0; 8] && discriminant != ANCHOR_CLOSED_ACCOUNT_DISCRIMINATOR {
            return Err(ProgramError::InvalidAccountData);
        }

        r[..8].copy_from_slice(&T::discriminant());

        Ok(RefMut::map(r, |r| {
            from_bytes_mut(&mut r[8..][..size_of::<T>()])
        }))
    }
}
impl<'info, T> AccountSet<'info> for AccountLoader<'info, T>
where
    T: Pod + ProgramAccount,
    T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
{
    fn try_to_accounts<'a, E>(
        &'a self,
        mut add_account: impl FnMut(&'a AccountInfo<'info>) -> Result<(), E>,
    ) -> Result<(), E>
    where
        'info: 'a,
    {
        add_account(&self.info)
    }

    fn to_account_metas(&self, mut add_account_meta: impl FnMut(AccountMeta)) {
        add_account_meta(self.info.account_meta())
    }
}
impl<'info, T> SingleAccountSet<'info> for AccountLoader<'info, T>
where
    T: Pod + ProgramAccount,
    T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        &self.info
    }
}

impl<'a, 'info, T> AccountSetDecode<'a, 'info, ()> for AccountLoader<'info, T>
where
    T: Pod + ProgramAccount,
    T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            info: AccountInfo::decode_accounts(accounts, decode_input, sys_calls)?,
            data: PhantomData,
        })
    }
}
impl<'a, 'info, T> AccountSetValidate<'info, AnchorValidateArgs<'a, 'info>>
    for AccountLoader<'info, T>
where
    T: Pod + ProgramAccount,
    T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
{
    fn validate_accounts(
        &mut self,
        validate_input: AnchorValidateArgs<'a, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<(), ProgramError> {
        validate_input.validate(self, sys_calls, T::discriminant())?;
        self.info.validate_accounts((), sys_calls)
    }
}
impl<'a, 'info, T> AccountSetCleanup<'info, AnchorCleanupArgs<'a, 'info>>
    for AccountLoader<'info, T>
where
    T: Pod + ProgramAccount,
    T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: AnchorCleanupArgs<'a, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<(), ProgramError> {
        if let Some(close_to) = cleanup_input.close {
            let mut self_lamports = self.info.lamports.borrow_mut();
            let mut close_to_lamports = close_to.lamports.borrow_mut();
            **close_to_lamports += **self_lamports;
            **self_lamports = 0;
            self.info.info_data_bytes_mut()?[..8]
                .copy_from_slice(&ANCHOR_CLOSED_ACCOUNT_DISCRIMINATOR);
        }
        self.info.cleanup_accounts((), sys_calls)
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::anchor_replacement::account_loader::AccountLoader;
    use crate::anchor_replacement::AnchorValidateArgs;
    use crate::idl::{AccountSetToIdl, AccountToIdl};
    use crate::program_account::ProgramAccount;
    use bytemuck::Pod;
    use star_frame_idl::account_set::{IdlAccountSetDef, IdlRawInputAccount};
    use star_frame_idl::IdlDefinition;

    impl<'a, 'info1, 'info2, T> AccountSetToIdl<'info1, AnchorValidateArgs<'a, 'info2>>
        for AccountLoader<'info1, T>
    where
        T: Pod + ProgramAccount + AccountToIdl,
        T::OwnerProgram: StarFrameProgram<AccountDiscriminant = [u8; 8]>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: AnchorValidateArgs,
        ) -> Result<IdlAccountSetDef> {
            let account = T::account_to_idl(idl_definition)?;
            Ok(IdlAccountSetDef::RawAccount(IdlRawInputAccount {
                possible_account_types: vec![account],
                allow_zeroed: arg.init.map(|i| i.is_zeroed()).unwrap_or(false),
                allow_uninitialized: arg.init.map(|i| i.is_init()).unwrap_or(false),
                signer: arg.check_signer,
                writable: arg.check_writable || arg.init.is_some(),
                extension_fields: Default::default(),
            }))
        }
    }
}
