use crate::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::client::{ClientAccountSet, CpiAccountSet};
use crate::context::Context;
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
    fn extend_account_infos(
        program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        ctx: &Context,
    ) -> Result<()> {
        for a in accounts {
            A::extend_account_infos(program_id, a, infos, ctx)?;
        }
        Ok(())
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
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: [DArg; N],
        ctx: &mut Context,
    ) -> Result<Self> {
        let mut decode_input = decode_input.into_iter();
        try_array_init(|_| A::decode_accounts(accounts, decode_input.next().unwrap(), ctx))
    }
}
impl<'a, A, const N: usize, DArg> AccountSetDecode<'a, (DArg,)> for [A; N]
where
    A: AccountSetDecode<'a, DArg>,
    DArg: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: (DArg,),
        ctx: &mut Context,
    ) -> Result<Self> {
        try_array_init(|_| A::decode_accounts(accounts, decode_input.0.clone(), ctx))
    }
}
impl<'a, A, const N: usize> AccountSetDecode<'a, ()> for [A; N]
where
    A: AccountSetDecode<'a, ()>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: (),
        ctx: &mut Context,
    ) -> Result<Self> {
        Self::decode_accounts(accounts, (decode_input,), ctx)
    }
}

impl<A, const N: usize, VArg> AccountSetValidate<[VArg; N]> for [A; N]
where
    A: AccountSetValidate<VArg>,
{
    fn validate_accounts(&mut self, validate_input: [VArg; N], ctx: &mut Context) -> Result<()> {
        for (a, v) in self.iter_mut().zip(validate_input) {
            a.validate_accounts(v, ctx)?;
        }
        Ok(())
    }
}
impl<A, const N: usize, VArg> AccountSetValidate<(VArg,)> for [A; N]
where
    A: AccountSetValidate<VArg>,
    VArg: Clone,
{
    fn validate_accounts(&mut self, validate_input: (VArg,), ctx: &mut Context) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input.0.clone(), ctx)?;
        }
        Ok(())
    }
}
impl<A, const N: usize> AccountSetValidate<()> for [A; N]
where
    A: AccountSetValidate<()>,
{
    fn validate_accounts(&mut self, validate_input: (), ctx: &mut Context) -> Result<()> {
        for a in self {
            a.validate_accounts(validate_input, ctx)?;
        }
        Ok(())
    }
}

impl<A, const N: usize, VArg> AccountSetCleanup<[VArg; N]> for [A; N]
where
    A: AccountSetCleanup<VArg>,
{
    fn cleanup_accounts(&mut self, cleanup_input: [VArg; N], ctx: &mut Context) -> Result<()> {
        for (a, v) in self.iter_mut().zip(cleanup_input) {
            a.cleanup_accounts(v, ctx)?;
        }
        Ok(())
    }
}
impl<A, const N: usize, VArg> AccountSetCleanup<(VArg,)> for [A; N]
where
    A: AccountSetCleanup<VArg>,
    VArg: Clone,
{
    fn cleanup_accounts(&mut self, cleanup_input: (VArg,), ctx: &mut Context) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input.0.clone(), ctx)?;
        }
        Ok(())
    }
}
impl<A, const N: usize> AccountSetCleanup<()> for [A; N]
where
    A: AccountSetCleanup<()>,
{
    fn cleanup_accounts(&mut self, cleanup_input: (), ctx: &mut Context) -> Result<()> {
        for a in self {
            a.cleanup_accounts(cleanup_input, ctx)?;
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
