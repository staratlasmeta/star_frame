use crate::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::client::{ClientAccountSet, CpiAccountSet};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use array_init::try_array_init;
use pinocchio::account_info::AccountInfo;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

impl<A, const N: usize> CpiAccountSet for [A; N]
where
    A: CpiAccountSet,
{
    type CpiAccounts = [A::CpiAccounts; N];
    const MIN_LEN: usize = N * A::MIN_LEN;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.each_ref().map(A::to_cpi_accounts)
    }
    #[inline]
    fn extend_account_infos(accounts: Self::CpiAccounts, infos: &mut Vec<AccountInfo>) {
        for a in accounts {
            A::extend_account_infos(a, infos);
        }
    }
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        for a in accounts {
            A::extend_account_metas(program_id, a, metas);
        }
    }
}

impl<A, const N: usize> ClientAccountSet for [A; N]
where
    A: ClientAccountSet,
{
    type ClientAccounts = [A::ClientAccounts; N];
    const MIN_LEN: usize = N * A::MIN_LEN;
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        for a in accounts {
            A::extend_account_metas(program_id, a, metas);
        }
    }
}

impl<'a, A, const N: usize, DArg> AccountSetDecode<'a, [DArg; N]> for [A; N]
where
    A: AccountSetDecode<'a, DArg>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: [DArg; N],
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<Self> {
        let mut decode_input = decode_input.into_iter();
        // SAFETY: This function is unsafe too
        try_array_init(|_| unsafe {
            A::decode_accounts(accounts, decode_input.next().unwrap(), syscalls)
        })
    }
}
impl<'a, A, const N: usize, DArg> AccountSetDecode<'a, (DArg,)> for [A; N]
where
    A: AccountSetDecode<'a, DArg>,
    DArg: Clone,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: (DArg,),
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<Self> {
        // SAFETY: This function is unsafe too
        try_array_init(|_| unsafe {
            A::decode_accounts(accounts, decode_input.0.clone(), syscalls)
        })
    }
}
impl<'a, A, const N: usize> AccountSetDecode<'a, ()> for [A; N]
where
    A: AccountSetDecode<'a, ()>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: (),
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<Self> {
        // SAFETY: This function is unsafe too
        unsafe { Self::decode_accounts(accounts, (decode_input,), syscalls) }
    }
}

impl<A, const N: usize, VArg> AccountSetValidate<[VArg; N]> for [A; N]
where
    A: AccountSetValidate<VArg>,
{
    fn validate_accounts(
        &mut self,
        validate_input: [VArg; N],
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        for (a, v) in self.iter_mut().zip(validate_input) {
            a.validate_accounts(v, syscalls)?;
        }
        Ok(())
    }
}
impl<A, const N: usize, VArg> AccountSetValidate<(VArg,)> for [A; N]
where
    A: AccountSetValidate<VArg>,
    VArg: Clone,
{
    fn validate_accounts(
        &mut self,
        validate_input: (VArg,),
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input.0.clone(), syscalls)?;
        }
        Ok(())
    }
}
impl<A, const N: usize> AccountSetValidate<()> for [A; N]
where
    A: AccountSetValidate<()>,
{
    fn validate_accounts(
        &mut self,
        validate_input: (),
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input, syscalls)?;
        }
        Ok(())
    }
}

impl<A, const N: usize, VArg> AccountSetCleanup<[VArg; N]> for [A; N]
where
    A: AccountSetCleanup<VArg>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: [VArg; N],
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        for (a, v) in self.iter_mut().zip(cleanup_input) {
            a.cleanup_accounts(v, syscalls)?;
        }
        Ok(())
    }
}
impl<A, const N: usize, VArg> AccountSetCleanup<(VArg,)> for [A; N]
where
    A: AccountSetCleanup<VArg>,
    VArg: Clone,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (VArg,),
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input.0.clone(), syscalls)?;
        }
        Ok(())
    }
}
impl<A, const N: usize> AccountSetCleanup<()> for [A; N]
where
    A: AccountSetCleanup<()>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input, syscalls)?;
        }
        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<T, const N: usize> AccountSetToIdl<()> for [T; N]
    where
        T: AccountSetToIdl<()>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (),
        ) -> crate::Result<IdlAccountSetDef> {
            let account_set = Box::new(T::account_set_to_idl(idl_definition, arg)?);
            Ok(IdlAccountSetDef::Many {
                account_set,
                min: N,
                max: Some(N),
            })
        }
    }
}
