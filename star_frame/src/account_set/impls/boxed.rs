//! `AccountSet` implementations for boxed types. Enables heap allocation of account sets with transparent delegation to the underlying type.

use std::mem::MaybeUninit;

use crate::{
    account_set::{
        modifiers::{
            CanInitAccount, CanInitSeeds, HasInnerType, HasOwnerProgram, HasSeeds, SignedAccount,
            WritableAccount,
        },
        single_set::SingleSetMeta,
        AccountSetCleanup, AccountSetDecode, AccountSetValidate, ClientAccountSet, CpiAccountSet,
    },
    prelude::*,
};

impl<T> SingleAccountSet for Box<T>
where
    T: SingleAccountSet,
{
    #[inline]
    fn meta() -> SingleSetMeta {
        T::meta()
    }

    #[inline]
    fn account_info(&self) -> &AccountInfo {
        T::account_info(self)
    }
}

unsafe impl<T> CpiAccountSet for Box<T>
where
    T: CpiAccountSet,
{
    type ContainsOption = T::ContainsOption;
    type CpiAccounts = T::CpiAccounts;
    type AccountLen = T::AccountLen;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        T::to_cpi_accounts(self)
    }

    #[inline]
    fn write_account_infos<'a>(
        program: Option<&'a AccountInfo>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        T::write_account_infos(program, accounts, index, infos)
    }

    #[inline]
    fn write_account_metas<'a>(
        program_id: &'a Pubkey,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<PinocchioAccountMeta<'a>>],
    ) {
        T::write_account_metas(program_id, accounts, index, metas);
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

impl<'a, T, DArg> AccountSetDecode<'a, DArg> for Box<T>
where
    T: AccountSetDecode<'a, DArg>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: DArg,
        ctx: &mut Context,
    ) -> Result<Self> {
        T::decode_accounts(accounts, decode_input, ctx).map(Box::new)
    }
}

impl<T, VArg> AccountSetValidate<VArg> for Box<T>
where
    T: AccountSetValidate<VArg>,
{
    fn validate_accounts(&mut self, validate_input: VArg, ctx: &mut Context) -> Result<()> {
        T::validate_accounts(self, validate_input, ctx)
    }
}

impl<T, CArg> AccountSetCleanup<CArg> for Box<T>
where
    T: AccountSetCleanup<CArg>,
{
    fn cleanup_accounts(&mut self, cleanup_input: CArg, ctx: &mut Context) -> Result<()> {
        T::cleanup_accounts(self, cleanup_input, ctx)
    }
}

impl<T> SignedAccount for Box<T>
where
    T: SignedAccount,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        T::signer_seeds(self)
    }
}

impl<T> WritableAccount for Box<T> where T: WritableAccount {}

impl<T> HasInnerType for Box<T>
where
    T: HasInnerType,
{
    type Inner = T::Inner;
}

impl<T> HasOwnerProgram for Box<T>
where
    T: HasOwnerProgram,
{
    type OwnerProgram = T::OwnerProgram;
}

impl<T> HasSeeds for Box<T>
where
    T: HasSeeds,
{
    type Seeds = T::Seeds;
}

impl<T, A> CanInitSeeds<A> for Box<T>
where
    T: CanInitSeeds<A>,
{
    fn init_seeds(&mut self, arg: &A, ctx: &Context) -> Result<()> {
        T::init_seeds(self, arg, ctx)
    }
}

impl<T, A> CanInitAccount<A> for Box<T>
where
    T: CanInitAccount<A>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: A,
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<bool> {
        T::init_account::<IF_NEEDED>(self, arg, account_seeds, ctx)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

    impl<T, Arg> AccountSetToIdl<Arg> for Box<T>
    where
        T: AccountSetToIdl<Arg>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: Arg,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
        }
    }
}
