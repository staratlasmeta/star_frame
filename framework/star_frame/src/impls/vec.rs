use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use anyhow::bail;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;

impl<'info, T> AccountSet<'info> for Vec<T>
where
    T: AccountSet<'info>,
{
    fn try_to_accounts<'a, E>(
        &'a self,
        mut add_account: impl FnMut(&'a AccountInfo<'info>) -> crate::Result<(), E>,
    ) -> crate::Result<(), E>
    where
        'info: 'a,
    {
        for acc in self.iter() {
            acc.try_to_accounts(&mut add_account)?;
        }
        Ok(())
    }

    fn to_account_metas(&self, mut add_account_meta: impl FnMut(AccountMeta)) {
        for acc in self.iter() {
            acc.to_account_metas(&mut add_account_meta);
        }
    }
}
impl<'a, 'info, T> AccountSetDecode<'a, 'info, usize> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, ()>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        len: usize,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        <Self as AccountSetDecode<'a, 'info, (usize, ())>>::decode_accounts(
            accounts,
            (len, ()),
            sys_calls,
        )
    }
}
impl<'a, 'info, T, TA> AccountSetDecode<'a, 'info, (usize, TA)> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, TA>,
    TA: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        (len, decode_input): (usize, TA),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        let mut output = Self::with_capacity(len);
        for _ in 0..len {
            output.push(T::decode_accounts(
                accounts,
                decode_input.clone(),
                sys_calls,
            )?);
        }
        Ok(output)
    }
}
impl<'a, 'info, T, TA, const N: usize> AccountSetDecode<'a, 'info, [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, TA>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: [TA; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        decode_input
            .into_iter()
            .map(|input| T::decode_accounts(accounts, input, sys_calls))
            .collect()
    }
}
impl<'a, 'b, 'info, T, TA, const N: usize> AccountSetDecode<'a, 'info, &'b [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, &'b TA>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: &'b [TA; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        decode_input
            .iter()
            .map(|input| T::decode_accounts(accounts, input, sys_calls))
            .collect()
    }
}
impl<'a, 'b, 'info, T, TA, const N: usize> AccountSetDecode<'a, 'info, &'b mut [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, &'b mut TA>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: &'b mut [TA; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        decode_input
            .iter_mut()
            .map(|input| T::decode_accounts(accounts, input, sys_calls))
            .collect()
    }
}
impl<'a, 'info, T, I> AccountSetDecode<'a, 'info, (I,)> for Vec<T>
where
    I: IntoIterator,
    T: AccountSetDecode<'a, 'info, I::Item>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (I,),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self> {
        decode_input
            .0
            .into_iter()
            .map(|input| T::decode_accounts(accounts, input, sys_calls))
            .collect()
    }
}

impl<'a, 'info, T> AccountSetValidate<'a, 'info, ()> for Vec<T>
where
    T: AccountSetValidate<'a, 'info, ()>,
{
    fn validate_accounts(
        &'a mut self,
        validate_input: (),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for account in self {
            account.validate_accounts(validate_input, sys_calls)?;
        }
        Ok(())
    }
}
// TODO: This arg is annoying
impl<'a, 'info, T, TA> AccountSetValidate<'a, 'info, (TA,)> for Vec<T>
where
    T: AccountSetValidate<'a, 'info, TA>,
    TA: Clone,
{
    fn validate_accounts(
        &'a mut self,
        validate_input: (TA,),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for account in self {
            account.validate_accounts(validate_input.0.clone(), sys_calls)?;
        }
        Ok(())
    }
}
impl<'a, 'info, T, TA> AccountSetValidate<'a, 'info, Vec<TA>> for Vec<T>
where
    T: AccountSetValidate<'a, 'info, TA>,
{
    fn validate_accounts(
        &'a mut self,
        validate_input: Vec<TA>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        if validate_input.len() < self.len() {
            bail!(ProgramError::InvalidAccountData);
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'a, 'info, T, TA, const N: usize> AccountSetValidate<'a, 'info, [TA; N]> for Vec<T>
where
    T: AccountSetValidate<'a, 'info, TA>,
{
    fn validate_accounts(
        &'a mut self,
        validate_input: [TA; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        if validate_input.len() != self.len() {
            bail!(ProgramError::InvalidAccountData);
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'a, 'b, 'info, T, TA, const N: usize> AccountSetValidate<'b, 'info, &'a mut [TA; N]> for Vec<T>
where
    T: AccountSetValidate<'b, 'info, &'a mut TA>,
{
    fn validate_accounts(
        &'b mut self,
        validate_input: &'a mut [TA; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        if validate_input.len() != self.len() {
            bail!(ProgramError::InvalidAccountData);
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}

impl<'a, 'info, T> AccountSetCleanup<'a, 'info, ()> for Vec<T>
where
    T: AccountSetCleanup<'a, 'info, ()>,
{
    fn cleanup_accounts(
        &'a mut self,
        cleanup_input: (),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for account in self {
            account.cleanup_accounts(cleanup_input, sys_calls)?;
        }
        Ok(())
    }
}
impl<'a, 'info, T, TA> AccountSetCleanup<'a, 'info, (TA,)> for Vec<T>
where
    T: AccountSetCleanup<'a, 'info, TA>,
    TA: Clone,
{
    fn cleanup_accounts(
        &'a mut self,
        cleanup_input: (TA,),
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        for account in self {
            account.cleanup_accounts(cleanup_input.0.clone(), sys_calls)?;
        }
        Ok(())
    }
}
impl<'a, 'info, T, TA> AccountSetCleanup<'a, 'info, Vec<TA>> for Vec<T>
where
    T: AccountSetCleanup<'a, 'info, TA>,
{
    fn cleanup_accounts(
        &'a mut self,
        cleanup_input: Vec<TA>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        if cleanup_input.len() < self.len() {
            bail!(ProgramError::InvalidAccountData);
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'a, 'info, T, TA, const N: usize> AccountSetCleanup<'a, 'info, [TA; N]> for Vec<T>
where
    T: AccountSetCleanup<'a, 'info, TA>,
{
    fn cleanup_accounts(
        &'a mut self,
        cleanup_input: [TA; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        if cleanup_input.len() != self.len() {
            bail!(ProgramError::InvalidAccountData);
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'a, 'b, 'info, T, TA, const N: usize> AccountSetCleanup<'b, 'info, &'a mut [TA; N]> for Vec<T>
where
    T: AccountSetCleanup<'b, 'info, &'a mut TA>,
{
    fn cleanup_accounts(
        &'b mut self,
        cleanup_input: &'a mut [TA; N],
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()> {
        if cleanup_input.len() != self.len() {
            bail!(ProgramError::InvalidAccountData);
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}

#[cfg(feature = "idl")]
pub mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    #[derive(Debug, Copy, Clone)]
    pub struct VecSize {
        pub min: usize,
        pub max: Option<usize>,
    }

    impl<'info, T, A> AccountSetToIdl<'info, (VecSize, A)> for Vec<T>
    where
        T: AccountSetToIdl<'info, A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (VecSize, A),
        ) -> crate::Result<IdlAccountSetDef> {
            let account = Box::new(T::account_set_to_idl(idl_definition, arg.1)?);
            Ok(IdlAccountSetDef::Many {
                account,
                min: arg.0.min,
                max: arg.0.max,
            })
        }
    }

    impl<'info, T> AccountSetToIdl<'info, ()> for Vec<T>
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
                min: 0,
                max: None,
            })
        }
    }
}
