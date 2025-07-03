use crate::account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate};

use crate::prelude::{ClientAccountSet, Context, CpiAccountSet};
use crate::Result;
use anyhow::bail;
use pinocchio::account_info::AccountInfo;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

impl<T> CpiAccountSet for Vec<T>
where
    T: CpiAccountSet,
{
    type CpiAccounts = Vec<T::CpiAccounts>;
    const MIN_LEN: usize = 0;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        self.iter().map(T::to_cpi_accounts).collect()
    }
    #[inline]
    fn extend_account_infos(
        program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        ctx: &Context,
    ) -> Result<()> {
        for a in accounts {
            T::extend_account_infos(program_id, a, infos, ctx)?;
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
            T::extend_account_metas(program_id, a, metas);
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
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        len: usize,
        ctx: &mut Context,
    ) -> Result<Self> {
        // SAFETY: This function is unsafe too
        unsafe {
            <Self as AccountSetDecode<'a, (usize, ())>>::decode_accounts(accounts, (len, ()), ctx)
        }
    }
}
impl<'a, T, TA> AccountSetDecode<'a, (usize, TA)> for Vec<T>
where
    T: AccountSetDecode<'a, TA>,
    TA: Clone,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        (len, decode_input): (usize, TA),
        ctx: &mut Context,
    ) -> Result<Self> {
        let mut output = Self::with_capacity(len);
        for _ in 0..len {
            // SAFETY: This function is unsafe too
            output.push(unsafe { T::decode_accounts(accounts, decode_input.clone(), ctx) }?);
        }
        Ok(output)
    }
}
impl<'a, T, TA, const N: usize> AccountSetDecode<'a, [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, TA>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: [TA; N],
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .into_iter()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, ctx) }
            })
            .collect()
    }
}
impl<'a, 'b, T, TA, const N: usize> AccountSetDecode<'a, &'b [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, &'b TA>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: &'b [TA; N],
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .iter()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, ctx) }
            })
            .collect()
    }
}
impl<'a, 'b, T, TA, const N: usize> AccountSetDecode<'a, &'b mut [TA; N]> for Vec<T>
where
    T: AccountSetDecode<'a, &'b mut TA>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: &'b mut [TA; N],
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .iter_mut()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, ctx) }
            })
            .collect()
    }
}
impl<'a, T, I> AccountSetDecode<'a, (I,)> for Vec<T>
where
    I: IntoIterator,
    T: AccountSetDecode<'a, I::Item>,
{
    unsafe fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        decode_input: (I,),
        ctx: &mut Context,
    ) -> Result<Self> {
        decode_input
            .0
            .into_iter()
            .map(|input| {
                // SAFETY: This function is unsafe too
                unsafe { T::decode_accounts(accounts, input, ctx) }
            })
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
                "Invalid account data: validate input length {} is less than required length {}",
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
                "Invalid account data: validate input length {} does not match required length {}",
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
            bail!("Invalid account data");
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
                "Invalid account data: cleanup input length {} is less than required length {}",
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
                "Invalid account data: cleanup input length {} does not match required length {}",
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
            bail!("Invalid account data");
        }

        for (account, input) in self.iter_mut().zip(cleanup_input) {
            account.cleanup_accounts(input, ctx)?;
        }

        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    #[derive(Debug, Copy, Clone)]
    pub struct VecSize {
        pub min: usize,
        pub max: Option<usize>,
    }

    impl<T, A> AccountSetToIdl<(VecSize, A)> for Vec<T>
    where
        T: AccountSetToIdl<A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: (VecSize, A),
        ) -> crate::Result<IdlAccountSetDef> {
            let account = Box::new(T::account_set_to_idl(idl_definition, arg.1)?);
            Ok(IdlAccountSetDef::Many {
                account_set: account,
                min: arg.0.min,
                max: arg.0.max,
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
        ) -> crate::Result<IdlAccountSetDef> {
            let account = Box::new(T::account_set_to_idl(idl_definition, arg)?);
            Ok(IdlAccountSetDef::Many {
                account_set: account,
                min: 0,
                max: None,
            })
        }
    }
}
