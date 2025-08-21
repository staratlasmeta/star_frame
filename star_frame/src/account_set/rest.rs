use crate::{
    account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate},
    client::{ClientAccountSet, CpiAccountSet},
    context::Context,
};
use derive_more::{Deref, DerefMut};
use pinocchio::account_info::AccountInfo;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

#[derive(AccountSet, Debug, Deref, DerefMut, Clone)]
#[account_set(
    skip_cpi_account_set,
    skip_client_account_set,
    skip_default_decode,
    skip_default_idl
)]
#[validate(generics = [<A> where T: AccountSetValidate<A>, A: Clone], arg = A)]
#[cleanup(generics = [<A> where T: AccountSetCleanup<A>, A: Clone], arg = A)]
pub struct Rest<T>(
    #[validate(arg = (arg,))]
    #[cleanup(arg = (arg,))]
    Vec<T>,
);

impl<T> CpiAccountSet for Rest<T>
where
    T: CpiAccountSet,
{
    type CpiAccounts = Vec<T::CpiAccounts>;
    const MIN_LEN: usize = 0;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        CpiAccountSet::to_cpi_accounts(&self.0)
    }
    #[inline]
    fn extend_account_infos(
        program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        ctx: &Context,
    ) -> anyhow::Result<()> {
        <Vec<T>>::extend_account_infos(program_id, accounts, infos, ctx)
    }
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
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
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        <Vec<T>>::extend_account_metas(program_id, accounts, metas);
    }
}

impl<'a, A, T> AccountSetDecode<'a, A> for Rest<T>
where
    T: AccountSetDecode<'a, A>,
    A: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: A,
        ctx: &mut Context,
    ) -> crate::Result<Self> {
        let mut out = vec![];
        while !accounts.is_empty() {
            out.push(T::decode_accounts(accounts, decode_input.clone(), ctx)?);
        }
        Ok(Self(out))
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::{account_set::vec::idl_impl::VecSize, idl::AccountSetToIdl};

    impl<T, A> AccountSetToIdl<A> for Rest<T>
    where
        T: AccountSetToIdl<A>,
        A: Clone,
    {
        fn account_set_to_idl(
            idl_definition: &mut star_frame::__private::macro_prelude::IdlDefinition,
            arg: A,
        ) -> star_frame::Result<star_frame::__private::macro_prelude::IdlAccountSetDef> {
            <Vec<T> as AccountSetToIdl<_>>::account_set_to_idl(
                idl_definition,
                (VecSize { min: 0, max: None }, arg),
            )
        }
    }
}
