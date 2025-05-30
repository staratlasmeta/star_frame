use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::prelude::{ClientAccountSet, CpiAccountSet};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use anyhow::bail;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

impl<'info, T> AccountSet<'info> for Vec<T> where T: AccountSet<'info> {}

impl<'info, T> CpiAccountSet<'info> for Vec<T>
where
    T: CpiAccountSet<'info>,
{
    type CpiAccounts = Vec<T::CpiAccounts>;
    const MIN_LEN: usize = 0;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.iter().map(T::to_cpi_accounts).collect()
    }
    #[inline]
    fn extend_account_infos(accounts: Self::CpiAccounts, infos: &mut Vec<AccountInfo<'info>>) {
        for a in accounts {
            T::extend_account_infos(a, infos);
        }
    }
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        for a in accounts {
            T::extend_account_metas(program_id, a, metas);
        }
    }
}

impl<T> ClientAccountSet for Vec<T>
where
    T: ClientAccountSet,
{
    type ClientAccounts = Vec<T::ClientAccounts>;
    const MIN_LEN: usize = 0;
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        for a in accounts {
            T::extend_account_metas(program_id, a, metas);
        }
    }
}

impl<'a, 'info, T> AccountSetDecode<'a, 'info, usize> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, ()>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        len: usize,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        // SAFETY: This function is unsafe too
        unsafe {
            <Self as AccountSetDecode<'a, 'info, (usize, ())>>::decode_accounts(
                accounts,
                (len, ()),
                syscalls,
            )
        }
    }
}
impl<'a, 'info, T, TA> AccountSetDecode<'a, 'info, (usize, TA)> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, TA>,
    TA: Clone,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        (len, decode_input): (usize, TA),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        let mut output = Self::with_capacity(len);
        for _ in 0..len {
            // SAFETY: This function is unsafe too
            output.push(unsafe { T::decode_accounts(accounts, decode_input.clone(), syscalls) }?);
        }
        Ok(output)
    }
}
impl<'a, 'info, T, TA, const N: usize> AccountSetDecode<'a, 'info, [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, TA>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: [TA; N],
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        decode_input
            .into_iter()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, syscalls) }
            })
            .collect()
    }
}
impl<'a, 'b, 'info, T, TA, const N: usize> AccountSetDecode<'a, 'info, &'b [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, &'b TA>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: &'b [TA; N],
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        decode_input
            .iter()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, syscalls) }
            })
            .collect()
    }
}
impl<'a, 'b, 'info, T, TA, const N: usize> AccountSetDecode<'a, 'info, &'b mut [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, 'info, &'b mut TA>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: &'b mut [TA; N],
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        decode_input
            .iter_mut()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, syscalls) }
            })
            .collect()
    }
}
impl<'a, 'info, T, I> AccountSetDecode<'a, 'info, (I,)> for Vec<T>
where
    I: IntoIterator,
    T: AccountSetDecode<'a, 'info, I::Item>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: (I,),
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        decode_input
            .0
            .into_iter()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, syscalls) }
            })
            .collect()
    }
}

impl<'info, T> AccountSetValidate<'info, ()> for Vec<T>
where
    T: AccountSetValidate<'info, ()>,
{
    fn validate_accounts(
        &mut self,
        validate_input: (),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for account in self {
            account.validate_accounts(validate_input, sys_calls)?;
        }
        Ok(())
    }
}
// TODO: This arg is annoying
impl<'info, T, TA> AccountSetValidate<'info, (TA,)> for Vec<T>
where
    T: AccountSetValidate<'info, TA>,
    TA: Clone,
{
    fn validate_accounts(
        &mut self,
        validate_input: (TA,),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for account in self {
            account.validate_accounts(validate_input.0.clone(), sys_calls)?;
        }
        Ok(())
    }
}
impl<'info, T, TA> AccountSetValidate<'info, Vec<TA>> for Vec<T>
where
    T: AccountSetValidate<'info, TA>,
{
    fn validate_accounts(
        &mut self,
        validate_input: Vec<TA>,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if validate_input.len() < self.len() {
            bail!(
                "Invalid account data: validate input length {} is less than required length {}",
                validate_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'info, T, TA, const N: usize> AccountSetValidate<'info, [TA; N]> for Vec<T>
where
    T: AccountSetValidate<'info, TA>,
{
    fn validate_accounts(
        &mut self,
        validate_input: [TA; N],
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if validate_input.len() != self.len() {
            bail!(
                "Invalid account data: validate input length {} does not match required length {}",
                validate_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'a, 'info, T, TA, const N: usize> AccountSetValidate<'info, &'a mut [TA; N]> for Vec<T>
where
    T: AccountSetValidate<'info, &'a mut TA>,
{
    fn validate_accounts(
        &mut self,
        validate_input: &'a mut [TA; N],
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if validate_input.len() != self.len() {
            bail!("Invalid account data");
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}

impl<'info, T> AccountSetCleanup<'info, ()> for Vec<T>
where
    T: AccountSetCleanup<'info, ()>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for account in self {
            account.cleanup_accounts(cleanup_input, sys_calls)?;
        }
        Ok(())
    }
}
impl<'info, T, TA> AccountSetCleanup<'info, (TA,)> for Vec<T>
where
    T: AccountSetCleanup<'info, TA>,
    TA: Clone,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (TA,),
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        for account in self {
            account.cleanup_accounts(cleanup_input.0.clone(), sys_calls)?;
        }
        Ok(())
    }
}
impl<'info, T, TA> AccountSetCleanup<'info, Vec<TA>> for Vec<T>
where
    T: AccountSetCleanup<'info, TA>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: Vec<TA>,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if cleanup_input.len() < self.len() {
            bail!(
                "Invalid account data: cleanup input length {} is less than required length {}",
                cleanup_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'info, T, TA, const N: usize> AccountSetCleanup<'info, [TA; N]> for Vec<T>
where
    T: AccountSetCleanup<'info, TA>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: [TA; N],
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if cleanup_input.len() != self.len() {
            bail!(
                "Invalid account data: cleanup input length {} does not match required length {}",
                cleanup_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}
impl<'a, 'info, T, TA, const N: usize> AccountSetCleanup<'info, &'a mut [TA; N]> for Vec<T>
where
    T: AccountSetCleanup<'info, &'a mut TA>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: &'a mut [TA; N],
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if cleanup_input.len() != self.len() {
            bail!("Invalid account data");
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, sys_calls)?;
        }

        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
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
                account_set: account,
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
                account_set: account,
                min: 0,
                max: None,
            })
        }
    }
}
