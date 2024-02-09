use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use array_init::try_array_init;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;

impl<'info, A, const N: usize> AccountSet<'info> for [A; N]
where
    A: AccountSet<'info>,
{
    fn try_to_accounts<'a, E>(
        &'a self,
        mut add_account: impl FnMut(&'a AccountInfo<'info>) -> Result<(), E>,
    ) -> Result<(), E>
    where
        'info: 'a,
    {
        for a in self.iter() {
            a.try_to_accounts(&mut add_account)?;
        }
        Ok(())
    }

    fn to_account_metas(&self, mut add_account_meta: impl FnMut(AccountMeta)) {
        for a in self.iter() {
            a.to_account_metas(&mut add_account_meta);
        }
    }
}

impl<'a, 'info, A, const N: usize, DArg> AccountSetDecode<'a, 'info, [DArg; N]> for [A; N]
where
    A: AccountSetDecode<'a, 'info, DArg>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: [DArg; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        let mut decode_input = decode_input.into_iter();
        try_array_init(|_| A::decode_accounts(accounts, decode_input.next().unwrap(), sys_calls))
    }
}
impl<'a, 'info, A, const N: usize, DArg> AccountSetDecode<'a, 'info, (DArg,)> for [A; N]
where
    A: AccountSetDecode<'a, 'info, DArg>,
    DArg: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (DArg,),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        try_array_init(|_| A::decode_accounts(accounts, decode_input.0.clone(), sys_calls))
    }
}
impl<'a, 'info, A, const N: usize> AccountSetDecode<'a, 'info, ()> for [A; N]
where
    A: AccountSetDecode<'a, 'info, ()>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        Self::decode_accounts(accounts, (decode_input,), sys_calls)
    }
}

impl<'info, A, const N: usize, VArg> AccountSetValidate<'info, [VArg; N]> for [A; N]
where
    A: AccountSetValidate<'info, VArg>,
{
    fn validate_accounts(
        &mut self,
        validate_input: [VArg; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for (a, v) in self.iter_mut().zip(validate_input.into_iter()) {
            a.validate_accounts(v, sys_calls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize, VArg> AccountSetValidate<'info, (VArg,)> for [A; N]
where
    A: AccountSetValidate<'info, VArg>,
    VArg: Clone,
{
    fn validate_accounts(
        &mut self,
        validate_input: (VArg,),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input.0.clone(), sys_calls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize> AccountSetValidate<'info, ()> for [A; N]
where
    A: AccountSetValidate<'info, ()>,
{
    fn validate_accounts(
        &mut self,
        validate_input: (),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input, sys_calls)?;
        }
        Ok(())
    }
}

impl<'info, A, const N: usize, VArg> AccountSetCleanup<'info, [VArg; N]> for [A; N]
where
    A: AccountSetCleanup<'info, VArg>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: [VArg; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for (a, v) in self.iter_mut().zip(cleanup_input.into_iter()) {
            a.cleanup_accounts(v, sys_calls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize, VArg> AccountSetCleanup<'info, (VArg,)> for [A; N]
where
    A: AccountSetCleanup<'info, VArg>,
    VArg: Clone,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (VArg,),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input.0.clone(), sys_calls)?;
        }
        Ok(())
    }
}
impl<'info, A, const N: usize> AccountSetCleanup<'info, ()> for [A; N]
where
    A: AccountSetCleanup<'info, ()>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input, sys_calls)?;
        }
        Ok(())
    }
}

#[cfg(feature = "idl")]
pub mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, const N: usize> AccountSetToIdl<'info, ()> for [T; N]
    where
        T: AccountSetToIdl<'info, ()>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (),
        ) -> crate::Result<IdlAccountSetDef> {
            let account = Box::new(T::account_set_to_idl(idl_definition, arg)?);
            Ok(IdlAccountSetDef::Many {
                account,
                min: N,
                max: Some(N),
            })
        }
    }
}
