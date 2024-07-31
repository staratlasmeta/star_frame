use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use std::marker::PhantomData;

impl<'info, T> AccountSet<'info> for PhantomData<T>
where
    T: ?Sized,
{
    fn try_to_accounts<'a, E>(
        &'a self,
        _add_account: impl FnMut(&'a AccountInfo<'info>) -> crate::Result<(), E>,
    ) -> Result<(), E>
    where
        'info: 'a,
    {
        Ok(())
    }

    fn to_account_metas(&self, _add_account_meta: impl FnMut(AccountMeta)) {}
}
impl<'a, 'info, T> AccountSetDecode<'a, 'info, ()> for PhantomData<T>
where
    T: ?Sized,
{
    fn decode_accounts(
        _accounts: &mut &'a [AccountInfo<'info>],
        _decode_input: (),
        _syscalls: &mut impl SyscallInvoke,
    ) -> Result<Self> {
        Ok(Self)
    }
}
impl<'info, T> AccountSetValidate<'info, ()> for PhantomData<T>
where
    T: ?Sized,
{
    fn validate_accounts(
        &mut self,
        _validate_input: (),
        _syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        Ok(())
    }
}
impl<'info, T> AccountSetCleanup<'info, ()> for PhantomData<T>
where
    T: ?Sized,
{
    fn cleanup_accounts(
        &mut self,
        _cleanup_input: (),
        _syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame::idl::AccountSetToIdl;
    use star_frame::program::system_program::SystemProgram;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<T> TypeToIdl for PhantomData<T>
    where
        T: TypeToIdl + ?Sized,
    {
        type AssociatedProgram = SystemProgram;

        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::Struct(vec![]))
        }
    }
    impl<'info, T> AccountSetToIdl<'info, ()> for PhantomData<T>
    where
        T: ?Sized,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            _arg: (),
        ) -> Result<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Struct(vec![]))
        }
    }
}
