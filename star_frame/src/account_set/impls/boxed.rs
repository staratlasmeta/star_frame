use crate::account_set::{
    AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate, SingleAccountSet,
    SingleSetMeta,
};
use crate::client::{ClientAccountSet, CpiAccountSet};
use crate::prelude::SyscallAccountCache;
use crate::syscalls::SyscallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

impl<'info, T> AccountSet<'info> for Box<T>
where
    T: AccountSet<'info>,
{
    #[inline]
    fn set_account_cache(&mut self, syscalls: &mut impl SyscallAccountCache<'info>) {
        T::set_account_cache(self, syscalls);
    }
}

impl<'info, T> SingleAccountSet<'info> for Box<T>
where
    T: SingleAccountSet<'info>,
{
    const META: SingleSetMeta = T::META;

    fn account_info(&self) -> &AccountInfo<'info> {
        T::account_info(self)
    }
}

impl<'info, T> CpiAccountSet<'info> for Box<T>
where
    T: CpiAccountSet<'info>,
{
    type CpiAccounts = T::CpiAccounts;
    const MIN_LEN: usize = T::MIN_LEN;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        T::to_cpi_accounts(self)
    }
    #[inline]
    fn extend_account_infos(accounts: Self::CpiAccounts, infos: &mut Vec<AccountInfo<'info>>) {
        T::extend_account_infos(accounts, infos);
    }
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        T::extend_account_metas(program_id, accounts, metas);
    }
}

impl<T> ClientAccountSet for Box<T>
where
    T: ClientAccountSet,
{
    type ClientAccounts = T::ClientAccounts;
    const MIN_LEN: usize = T::MIN_LEN;
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        T::extend_account_metas(program_id, accounts, metas);
    }
}

impl<'a, 'info, T, DArg> AccountSetDecode<'a, 'info, DArg> for Box<T>
where
    T: AccountSetDecode<'a, 'info, DArg>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: DArg,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        // SAFETY: This function is unsafe too
        unsafe { T::decode_accounts(accounts, decode_input, syscalls).map(Box::new) }
    }
}

impl<'info, T, VArg> AccountSetValidate<'info, VArg> for Box<T>
where
    T: AccountSetValidate<'info, VArg>,
{
    fn validate_accounts(
        &mut self,
        validate_input: VArg,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        T::validate_accounts(self, validate_input, sys_calls)
    }
}

impl<'info, T, CArg> AccountSetCleanup<'info, CArg> for Box<T>
where
    T: AccountSetCleanup<'info, CArg>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: CArg,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        T::cleanup_accounts(self, cleanup_input, sys_calls)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use crate::Result;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, Arg> AccountSetToIdl<'info, Arg> for Box<T>
    where
        T: AccountSetToIdl<'info, Arg>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: Arg,
        ) -> Result<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
        }
    }
}
