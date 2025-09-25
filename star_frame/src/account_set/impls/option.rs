//! `AccountSet` implementations for optional types. Enables conditional account presence using `Option<T>` syntax with automatic handling of None cases.

use std::mem::MaybeUninit;

use crate::{
    account_set::{
        AccountSetCleanup, AccountSetDecode, AccountSetValidate, CheckKey, ClientAccountSet,
        CpiAccountSet, DynamicCpiAccountSetLen,
    },
    prelude::*,
    ErrorCode,
};
use advancer::Advance;
use typenum::{Eq, IsEqual};

#[doc(hidden)]
pub trait OptionAccountLenHelper {
    type Result: typenum::Unsigned;
}

impl OptionAccountLenHelper for typenum::True {
    type Result = typenum::U1;
}

impl OptionAccountLenHelper for typenum::False {
    type Result = DynamicCpiAccountSetLen;
}

unsafe impl<T> CpiAccountSet for Option<T>
where
    T: CpiAccountSet,
    <T as CpiAccountSet>::AccountLen: IsEqual<typenum::U1>,
    <<T as CpiAccountSet>::AccountLen as IsEqual<typenum::U1>>::Output: OptionAccountLenHelper,
{
    type ContainsOption = typenum::True;
    type CpiAccounts = Option<T::CpiAccounts>;
    type AccountLen =
        <Eq<<T as CpiAccountSet>::AccountLen, typenum::U1> as OptionAccountLenHelper>::Result;

    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.as_ref().map(T::to_cpi_accounts)
    }

    fn write_account_infos<'a>(
        program: Option<&'a AccountInfo>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        if let Some(accounts) = accounts {
            T::write_account_infos(program, accounts, index, infos)
        } else {
            infos[*index] = MaybeUninit::new(program.ok_or_else(|| {
                error!(
                    ErrorCode::MissingOptionalProgram,
                    "Program not passed in to write_account_infos. This should be prevented by the MakeCpi trait",
                )
            })?);
            *index += 1;
            Ok(())
        }
    }

    fn write_account_metas<'a>(
        program_id: &'a Pubkey,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<pinocchio::instruction::AccountMeta<'a>>],
    ) {
        if let Some(accounts) = accounts {
            T::write_account_metas(program_id, accounts, index, metas);
        } else {
            metas[*index] = MaybeUninit::new(pinocchio::instruction::AccountMeta::readonly(
                program_id.as_array(),
            ));
            *index += 1;
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
    #[inline]
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: DArg,
        ctx: &mut Context,
    ) -> Result<Self> {
        if accounts.is_empty() {
            Ok(None)
        } else if accounts[0].pubkey().fast_eq(ctx.current_program_id()) {
            let _program = accounts
                .try_advance(1)
                .expect("There is at least one account skip Option<None>");
            Ok(None)
        } else {
            Ok(Some(A::decode_accounts(accounts, decode_input, ctx)?))
        }
    }
}

impl<A, VArg> AccountSetValidate<VArg> for Option<A>
where
    A: AccountSetValidate<VArg>,
{
    #[inline]
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
    #[inline]
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
    #[inline]
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
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

    impl<A, Arg> AccountSetToIdl<Arg> for Option<A>
    where
        A: AccountSetToIdl<Arg>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: Arg,
        ) -> crate::IdlResult<IdlAccountSetDef> {
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
