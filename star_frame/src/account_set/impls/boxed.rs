use crate::prelude::*;
use pinocchio::account_info::AccountInfo;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

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

impl<T> CpiAccountSet for Box<T>
where
    T: CpiAccountSet,
{
    type CpiAccounts = T::CpiAccounts;
    const MIN_LEN: usize = T::MIN_LEN;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        T::to_cpi_accounts(self)
    }
    #[inline]
    fn extend_account_infos(
        program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        ctx: &Context,
    ) -> Result<()> {
        T::extend_account_infos(program_id, accounts, infos, ctx)
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
        account_seeds: Option<Vec<&[u8]>>,
        ctx: &Context,
    ) -> Result<()> {
        T::init_account::<IF_NEEDED>(self, arg, account_seeds, ctx)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::{idl::AccountSetToIdl, Result};
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

    impl<T, Arg> AccountSetToIdl<Arg> for Box<T>
    where
        T: AccountSetToIdl<Arg>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: Arg,
        ) -> Result<IdlAccountSetDef> {
            T::account_set_to_idl(idl_definition, arg)
        }
    }
}
