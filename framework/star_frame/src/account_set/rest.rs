use crate::account_set::{
    AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate,
};
use crate::syscalls::SyscallInvoke;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use crate::client::{ClientAccountSet, CpiAccountSet};

#[derive(AccountSet, Debug, Deref, DerefMut)]
#[account_set(
    skip_cpi_account_set, 
    skip_client_account_set, 
    skip_default_decode, 
    skip_default_idl, 
    generics = [where T: AccountSet<'info>]
)]
#[validate(generics = [<A> where T: AccountSetValidate<'info, A>, A: Clone], arg = A)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<'info, A>, A: Clone], arg = A)]
pub struct Rest<T>(
    #[validate(arg = (arg,))]
    #[cleanup(arg = (arg,))]
    Vec<T>,
);

impl<'info, T> CpiAccountSet<'info> for Rest<T>
where
    T: CpiAccountSet<'info>,
{
    type CpiAccounts<'a> = Vec<T::CpiAccounts<'info>>;
    const MIN_LEN: usize = 0;
    fn extend_account_infos(
        accounts: Self::CpiAccounts<'info>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) {
        <Vec<T>>::extend_account_infos(accounts, infos);
    }
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts<'info>,
        metas: &mut Vec<AccountMeta>,
    ) {
        <Vec<T>>::extend_account_metas(program_id, accounts, metas);
    }
}

impl<T> ClientAccountSet for Rest<T>
where
    T: ClientAccountSet,
{
    type ClientAccounts = Vec<T::ClientAccounts>;
    const MIN_LEN: usize = 0;
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        <Vec<T>>::extend_account_metas(program_id, accounts, metas);
    }
}

impl<'a, 'info, A, T> AccountSetDecode<'a, 'info, A> for Rest<T>
where
    T: AccountSetDecode<'a, 'info, A>,
    A: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo<'info>],
        decode_input: A,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> crate::Result<Self> {
        let mut out = vec![];
        while !accounts.is_empty() {
            out.push(T::decode_accounts(
                accounts,
                decode_input.clone(),
                syscalls,
            )?);
        }
        Ok(Self(out))
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::account_set::vec::idl_impl::VecSize;
    use crate::idl::AccountSetToIdl;

    impl<'info, T, A> AccountSetToIdl<'info, A> for Rest<T>
    where
        T: AccountSetToIdl<'info, A>,
        A: Clone,
    {
        fn account_set_to_idl(
            idl_definition: &mut star_frame::__private::macro_prelude::IdlDefinition,
            arg: A,
        ) -> star_frame::Result<star_frame::__private::macro_prelude::IdlAccountSetDef> {
            <Vec<T> as AccountSetToIdl<'info, _>>::account_set_to_idl(
                idl_definition,
                (VecSize { min: 0, max: None }, arg),
            )
        }
    }
}
