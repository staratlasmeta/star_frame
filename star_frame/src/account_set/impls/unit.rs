//! `AccountSet` implementation for the unit type. Enables instructions that require no accounts using `()` syntax as a zero-cost abstraction.

use core::mem::MaybeUninit;

use crate::{
    account_set::{AccountSetCleanup, AccountSetDecode, AccountSetValidate, CpiAccountSet},
    prelude::*,
};

#[cfg(not(target_os = "solana"))]
impl crate::account_set::ClientAccountSet for () {
    type ClientAccounts = ();
    const MIN_LEN: usize = 0;
    #[inline]
    fn extend_account_metas(
        _program_id: &Address,
        _accounts: &Self::ClientAccounts,
        _metas: &mut Vec<AccountMeta>,
    ) {
    }
}

unsafe impl CpiAccountSet for () {
    type ContainsOption = typenum::False;
    type CpiAccounts = ();
    type AccountLen = typenum::U0;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {}

    #[inline]
    fn write_account_infos<'a>(
        _program: Option<&'a AccountView>,
        _accounts: &'a Self::CpiAccounts,
        _index: &mut usize,
        _infos: &mut [MaybeUninit<&'a AccountView>],
    ) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn write_account_metas<'a>(
        _program_id: &'a Address,
        _accounts: &'a Self::CpiAccounts,
        _index: &mut usize,
        _metas: &mut [MaybeUninit<InstructionAccount<'a>>],
    ) {
    }
}

impl<'a> AccountSetDecode<'a, ()> for () {
    fn decode_accounts(
        _accounts: &mut &'a [AccountView],
        decode_input: (),
        _ctx: &mut Context,
    ) -> Result<Self> {
        Ok(decode_input)
    }
}
impl AccountSetValidate<()> for () {
    fn validate_accounts(&mut self, validate_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(validate_input)
    }
}

impl AccountSetCleanup<()> for () {
    fn cleanup_accounts(&mut self, cleanup_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(cleanup_input)
    }
}
