use crate::account_set::{AccountSet, AccountSetDecode, AccountSetValidate, SingleAccountSet};
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use star_frame::account_set::AccountSetCleanup;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct Signer<T>(T);
impl<T> Deref for Signer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Signer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> Signer<T> {
    pub fn new(info: T) -> Self {
        Self(info)
    }

    pub fn try_from<'info>(info: &T) -> Result<Self>
    where
        T: SingleAccountSet<'info> + Clone,
    {
        if !info.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(Signer::new(info.clone()))
    }
}
impl<'info, T> AccountSet<'info> for Signer<T>
where
    T: AccountSet<'info>,
{
    fn try_to_accounts<'a, E>(
        &'a self,
        add_account: impl FnMut(&'a AccountInfo<'info>) -> std::result::Result<(), E>,
    ) -> std::result::Result<(), E>
    where
        'info: 'a,
    {
        self.0.try_to_accounts(add_account)
    }

    fn to_account_metas(&self, add_account_meta: impl FnMut(AccountMeta)) {
        self.0.to_account_metas(add_account_meta)
    }
}
impl<'info, T> SingleAccountSet<'info> for Signer<T>
where
    T: SingleAccountSet<'info>,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.0.account_info()
    }
}
impl<'a, 'info, T, A> AccountSetDecode<'a, 'info, A> for Signer<T>
where
    T: AccountSetDecode<'a, 'info, A>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> std::result::Result<Self, ProgramError> {
        Ok(Self(T::decode_accounts(accounts, decode_input, sys_calls)?))
    }
}
impl<'info, T, A> AccountSetValidate<'info, A> for Signer<T>
where
    T: AccountSetValidate<'info, A>,
{
    fn validate_accounts(
        &mut self,
        validate_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> std::result::Result<(), ProgramError> {
        self.0.validate_accounts(validate_input, sys_calls)?;
        self.try_to_accounts(|a| {
            if a.is_signer {
                Ok(())
            } else {
                Err(ProgramError::MissingRequiredSignature)
            }
        })
    }
}
impl<'info, T, A> AccountSetCleanup<'info, A> for Signer<T>
where
    T: AccountSetCleanup<'info, A>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> std::result::Result<(), ProgramError> {
        self.0.cleanup_accounts(cleanup_input, sys_calls)
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, A> AccountSetToIdl<'info, A> for Signer<T>
    where
        T: AccountSetToIdl<'info, A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
                .map(Box::new)
                .map(IdlAccountSetDef::Signer)
        }
    }
}
