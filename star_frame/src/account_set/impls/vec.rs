//! `AccountSet` implementations for dynamic vectors. Enables variable-length collections of accounts using `Vec<T>` syntax with runtime-determined sizes.

use crate::{
    account_set::{
        AccountSetCleanup, AccountSetDecode, AccountSetValidate, ClientAccountSet, CpiAccountSet,
        DynamicCpiAccountSetLen,
    },
    prelude::*,
};

unsafe impl<T> CpiAccountSet for Vec<T>
where
    T: CpiAccountSet,
{
    type ContainsOption = T::ContainsOption;
    type CpiAccounts = Vec<T::CpiAccounts>;
    type AccountLen = DynamicCpiAccountSetLen;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.iter().map(T::to_cpi_accounts).collect()
    }

    #[inline]
    fn write_account_infos<'a>(
        program: Option<&'a AccountInfo>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [core::mem::MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        for a in accounts {
            T::write_account_infos(program, a, index, infos)?;
        }
        Ok(())
    }

    fn write_account_metas<'a>(
        program_id: &'a Pubkey,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [core::mem::MaybeUninit<pinocchio::instruction::AccountMeta<'a>>],
    ) {
        for a in accounts {
            T::write_account_metas(program_id, a, index, metas);
        }
    }
}

impl<T> ClientAccountSet for Vec<T>
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
        for a in accounts {
            T::extend_account_metas(program_id, a, metas);
        }
    }
}

impl<'a, T> AccountSetDecode<'a, usize> for Vec<T>
where
    T: AccountSetDecode<'a, ()>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        len: usize,
        ctx: &mut Context,
    ) -> Result<Self> {
        <Self as AccountSetDecode<'a, (usize, ())>>::decode_accounts(accounts, (len, ()), ctx)
    }
}
impl<'a, T, TA> AccountSetDecode<'a, (usize, TA)> for Vec<T>
where
    T: AccountSetDecode<'a, TA>,
    TA: Clone,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        (len, decode_input): (usize, TA),
        ctx: &mut Context,
    ) -> Result<Self> {
        let mut output = Self::with_capacity(len);
        for _ in 0..len {
            output.push(T::decode_accounts(accounts, decode_input.clone(), ctx)?);
        }
        Ok(output)
    }
}
impl<'a, T, TA, const N: usize> AccountSetDecode<'a, [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, TA>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: [TA; N],
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .into_iter()
            .map(|input| T::decode_accounts(accounts, input, ctx))
            .collect()
    }
}
impl<'a, 'b, T, TA, const N: usize> AccountSetDecode<'a, &'b [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, &'b TA>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: &'b [TA; N],
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .iter()
            .map(|input| T::decode_accounts(accounts, input, ctx))
            .collect()
    }
}
impl<'a, 'b, T, TA, const N: usize> AccountSetDecode<'a, &'b mut [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, &'b mut TA>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: &'b mut [TA; N],
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .iter_mut()
            .map(|input| T::decode_accounts(accounts, input, ctx))
            .collect()
    }
}
impl<'a, T, I> AccountSetDecode<'a, (I,)> for Vec<T>
where
    I: IntoIterator,
    T: AccountSetDecode<'a, I::Item>,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: (I,),
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .0
            .into_iter()
            .map(|input| T::decode_accounts(accounts, input, ctx))
            .collect()
    }
}

impl<T> AccountSetValidate<()> for Vec<T>
where
    T: AccountSetValidate<()>,
{
    fn validate_accounts(&mut self, validate_input: (), ctx: &mut Context) -> Result<()> {
        for account in self {
            account.validate_accounts(validate_input, ctx)?;
        }
        Ok(())
    }
}
// TODO: This arg is annoying
impl<T, TA> AccountSetValidate<(TA,)> for Vec<T>
where
    T: AccountSetValidate<TA>,
    TA: Clone,
{
    fn validate_accounts(&mut self, validate_input: (TA,), ctx: &mut Context) -> Result<()> {
        for account in self {
            account.validate_accounts(validate_input.0.clone(), ctx)?;
        }
        Ok(())
    }
}

impl<T, TA> AccountSetValidate<Vec<TA>> for Vec<T>
where
    T: AccountSetValidate<TA>,
{
    fn validate_accounts(&mut self, validate_input: Vec<TA>, ctx: &mut Context) -> Result<()> {
        if validate_input.len() < self.len() {
            bail!(
                ProgramError::InvalidArgument,
                "Validate input length {} is less than required length {}",
                validate_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, ctx)?;
        }

        Ok(())
    }
}
impl<T, TA, const N: usize> AccountSetValidate<[TA; N]> for Vec<T>
where
    T: AccountSetValidate<TA>,
{
    fn validate_accounts(&mut self, validate_input: [TA; N], ctx: &mut Context) -> Result<()> {
        if validate_input.len() != self.len() {
            bail!(
                ProgramError::InvalidArgument,
                "Validate input length {} does not match required length {}",
                validate_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, ctx)?;
        }

        Ok(())
    }
}
impl<'a, T, TA, const N: usize> AccountSetValidate<&'a mut [TA; N]> for Vec<T>
where
    T: AccountSetValidate<&'a mut TA>,
{
    fn validate_accounts(
        &mut self,
        validate_input: &'a mut [TA; N],
        ctx: &mut Context,
    ) -> Result<()> {
        if validate_input.len() != self.len() {
            bail!(
                ProgramError::InvalidArgument,
                "Validate input length {} does not match required length {}",
                validate_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(validate_input) {
            account.validate_accounts(input, ctx)?;
        }

        Ok(())
    }
}

impl<T> AccountSetCleanup<()> for Vec<T>
where
    T: AccountSetCleanup<()>,
{
    fn cleanup_accounts(&mut self, cleanup_input: (), ctx: &mut Context) -> Result<()> {
        for account in self {
            account.cleanup_accounts(cleanup_input, ctx)?;
        }
        Ok(())
    }
}
impl<T, TA> AccountSetCleanup<(TA,)> for Vec<T>
where
    T: AccountSetCleanup<TA>,
    TA: Clone,
{
    fn cleanup_accounts(&mut self, cleanup_input: (TA,), ctx: &mut Context) -> Result<()> {
        for account in self {
            account.cleanup_accounts(cleanup_input.0.clone(), ctx)?;
        }
        Ok(())
    }
}
impl<T, TA> AccountSetCleanup<Vec<TA>> for Vec<T>
where
    T: AccountSetCleanup<TA>,
{
    fn cleanup_accounts(&mut self, cleanup_input: Vec<TA>, ctx: &mut Context) -> Result<()> {
        if cleanup_input.len() < self.len() {
            bail!(
                ProgramError::InvalidArgument,
                "Cleanup input length {} is less than required length {}",
                cleanup_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, ctx)?;
        }

        Ok(())
    }
}
impl<T, TA, const N: usize> AccountSetCleanup<[TA; N]> for Vec<T>
where
    T: AccountSetCleanup<TA>,
{
    fn cleanup_accounts(&mut self, cleanup_input: [TA; N], ctx: &mut Context) -> Result<()> {
        if cleanup_input.len() != self.len() {
            bail!(
                ProgramError::InvalidArgument,
                "Cleanup input length {} does not match required length {}",
                cleanup_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, ctx)?;
        }

        Ok(())
    }
}
impl<'a, T, TA, const N: usize> AccountSetCleanup<&'a mut [TA; N]> for Vec<T>
where
    T: AccountSetCleanup<&'a mut TA>,
{
    fn cleanup_accounts(
        &mut self,
        cleanup_input: &'a mut [TA; N],
        ctx: &mut Context,
    ) -> Result<()> {
        if cleanup_input.len() != self.len() {
            bail!(
                ProgramError::InvalidArgument,
                "Cleanup input length {} does not match required length {}",
                cleanup_input.len(),
                self.len()
            );
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, ctx)?;
        }

        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl_impl {
    use super::*;
    use core::ops::{Bound, RangeBounds};

    use crate::idl::AccountSetToIdl;
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

    impl<B, T, A> AccountSetToIdl<(B, A)> for Vec<T>
    where
        B: RangeBounds<usize>,
        T: AccountSetToIdl<A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (B, A),
        ) -> crate::IdlResult<IdlAccountSetDef> {
            let account = Box::new(T::account_set_to_idl(idl_definition, arg.1)?);
            let min = match arg.0.start_bound() {
                Bound::Included(x) => *x,
                Bound::Excluded(x) => *x + 1,
                Bound::Unbounded => 0,
            };
            let max = match arg.0.end_bound() {
                Bound::Included(x) => Some(*x),
                Bound::Excluded(x) => Some(x.saturating_sub(1)),
                Bound::Unbounded => None,
            };
            Ok(IdlAccountSetDef::Many {
                account_set: account,
                min,
                max,
            })
        }
    }

    impl<T> AccountSetToIdl<()> for Vec<T>
    where
        T: AccountSetToIdl<()>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (),
        ) -> crate::IdlResult<IdlAccountSetDef> {
            let account = Box::new(T::account_set_to_idl(idl_definition, arg)?);
            Ok(IdlAccountSetDef::Many {
                account_set: account,
                min: 0,
                max: None,
            })
        }
    }
}
