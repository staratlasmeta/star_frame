use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::client::{ClientAccountSet, CpiAccountSet};
use crate::prelude::SyscallAccountCache;
use crate::syscalls::SyscallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

impl<'info, A> AccountSet<'info> for Option<A>
where
    A: AccountSet<'info>,
{
    fn set_account_cache(&mut self, syscalls: &mut impl SyscallAccountCache<'info>) {
        if let Some(inner) = self {
            inner.set_account_cache(syscalls);
        }
    }
}

impl<'info, T> CpiAccountSet<'info> for Option<T>
where
    T: CpiAccountSet<'info>,
{
    type CpiAccounts<'a> = Option<T::CpiAccounts<'info>>;
    const MIN_LEN: usize = 1;
    fn extend_account_infos(
        accounts: Self::CpiAccounts<'info>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) {
        if let Some(accounts) = accounts {
            T::extend_account_infos(accounts, infos);
        }
    }
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts<'info>,
        metas: &mut Vec<AccountMeta>,
    ) {
        if let Some(accounts) = accounts {
            T::extend_account_metas(program_id, accounts, metas);
        } else {
            metas.push(AccountMeta::new_readonly(*program_id, false));
        }
    }
}

impl<T> ClientAccountSet for Option<T>
where
    T: ClientAccountSet,
{
    type ClientAccounts = Option<T::ClientAccounts>;
    const MIN_LEN: usize = 1;
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        if let Some(accounts) = accounts {
            T::extend_account_metas(program_id, accounts, metas);
        } else {
            metas.push(AccountMeta::new_readonly(*program_id, false));
        }
    }
}

impl<'a, 'info, A, DArg> AccountSetDecode<'a, 'info, DArg> for Option<A>
where
    A: AccountSetDecode<'a, 'info, DArg>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: DArg,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        if accounts.is_empty() || accounts[0].key == syscalls.current_program_id() {
            Ok(None)
        } else {
            Ok(Some(A::decode_accounts(accounts, decode_input, syscalls)?))
        }
    }
}

impl<'info, A, VArg> AccountSetValidate<'info, VArg> for Option<A>
where
    A: AccountSetValidate<'info, VArg>,
{
    fn validate_accounts(
        &mut self,
        validate_input: VArg,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if let Some(inner) = self {
            inner.validate_accounts(validate_input, sys_calls)
        } else {
            Ok(())
        }
    }
}

impl<'info, A, CArg> AccountSetCleanup<'info, CArg> for Option<A>
where
    A: AccountSetCleanup<'info, CArg>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: CArg,
        sys_calls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if let Some(inner) = self {
            inner.cleanup_accounts(cleanup_input, sys_calls)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use crate::Result;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    // todo: figure out our optionals for IDLs. Thinking we should remove our separate decode
    //  strategies and just use the program id method. This would make using option much simpler on
    //  arg side and be more in line with how the rest of the ecosystem handles optionals.
    impl<'info, A, Arg> AccountSetToIdl<'info, Arg> for Option<A>
    where
        A: AccountSetToIdl<'info, Arg>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: Arg,
        ) -> Result<IdlAccountSetDef> {
            let mut set = A::account_set_to_idl(idl_definition, arg)?;
            if let Ok(inner) = set.single() {
                inner.optional = true;
                return Ok(set);
            }
            Ok(IdlAccountSetDef::Or(vec![
                set,
                IdlAccountSetDef::empty_struct(),
            ]))
        }
    }
}
