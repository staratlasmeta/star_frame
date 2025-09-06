//! `AccountSet` implementation for the unit type. Enables instructions that require no accounts using `()` syntax as a zero-cost abstraction.

use std::mem::MaybeUninit;

use crate::{
    account_set::{
        AccountSetCleanup, AccountSetDecode, AccountSetValidate, ClientAccountSet, CpiAccountSet,
    },
    prelude::*,
};

impl ClientAccountSet for () {
    type ClientAccounts = ();
    const MIN_LEN: usize = 0;
    #[inline]
    fn extend_account_metas(
        _program_id: &Pubkey,
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
        _program: Option<&'a AccountInfo>,
        _accounts: &'a Self::CpiAccounts,
        _index: &mut usize,
        _infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn write_account_metas<'a>(
        _program_id: &'a Pubkey,
        _accounts: &'a Self::CpiAccounts,
        _index: &mut usize,
        _metas: &mut [MaybeUninit<PinocchioAccountMeta<'a>>],
    ) {
    }
}

impl<'a> AccountSetDecode<'a, ()> for () {
    fn decode_accounts(
        _accounts: &mut &'a [AccountInfo],
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
