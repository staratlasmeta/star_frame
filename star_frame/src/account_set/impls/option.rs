use crate::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::client::{ClientAccountSet, CpiAccountSet};
use crate::prelude::{CheckKey, Context, SingleAccountSet};
use crate::Result;
use advancer::Advance;
use anyhow::Context as _;
use pinocchio::account_info::AccountInfo;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

impl<T> CpiAccountSet for Option<T>
where
    T: CpiAccountSet,
{
    type CpiAccounts = Option<T::CpiAccounts>;
    const MIN_LEN: usize = 1;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.as_ref().map(T::to_cpi_accounts)
    }
    #[inline]
    fn extend_account_infos(
        program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        ctx: &Context,
    ) -> Result<()> {
        if let Some(accounts) = accounts {
            T::extend_account_infos(program_id, accounts, infos, ctx)
        } else {
            infos.push(
                *ctx.program_for_key(program_id)
                    .context(format!("Program {program_id} not found"))?,
            );
            Ok(())
        }
    }
    #[inline]
    fn extend_account_metas(
        program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
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
    #[inline]
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

impl<'a, A, DArg> AccountSetDecode<'a, DArg> for Option<A>
where
    A: AccountSetDecode<'a, DArg>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: DArg,
        ctx: &mut Context,
    ) -> Result<Self> {
        if accounts.is_empty() {
            Ok(None)
        } else if accounts[0].pubkey() == ctx.current_program_id() {
            let _program = accounts
                .try_advance(1)
                .expect("There is at least one account skip Option<None>");
            Ok(None)
        } else {
            // SAFETY: This function is unsafe too
            Ok(Some(unsafe {
                A::decode_accounts(accounts, decode_input, ctx)?
            }))
        }
    }
}

impl<A, VArg> AccountSetValidate<VArg> for Option<A>
where
    A: AccountSetValidate<VArg>,
{
    fn validate_accounts(&mut self, validate_input: VArg, ctx: &mut Context) -> Result<()> {
        if let Some(inner) = self {
            inner.validate_accounts(validate_input, ctx)
        } else {
            Ok(())
        }
    }
}

impl<A, CArg> AccountSetCleanup<CArg> for Option<A>
where
    A: AccountSetCleanup<CArg>,
{
    fn cleanup_accounts(&mut self, cleanup_input: CArg, ctx: &mut Context) -> Result<()> {
        if let Some(inner) = self {
            inner.cleanup_accounts(cleanup_input, ctx)
        } else {
            Ok(())
        }
    }
}

impl<T> CheckKey for Option<T>
where
    T: CheckKey,
{
    fn check_key(&self, key: &Pubkey) -> Result<()> {
        if let Some(inner) = self {
            inner.check_key(key)
        } else {
            Ok(())
        }
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use crate::Result;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    // todo: figure out our optionals for IDLs. Thinking we should remove our separate decode
    //  strategies and just use the program id method. This would make using option much simpler on
    //  arg side and be more in line with how the rest of the ecosystem handles optionals.
    impl<A, Arg> AccountSetToIdl<Arg> for Option<A>
    where
        A: AccountSetToIdl<Arg>,
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
