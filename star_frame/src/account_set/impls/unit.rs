use crate::prelude::*;

impl CpiAccountSet for () {
    type CpiAccounts = ();
    const MIN_LEN: usize = 0;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {}
    #[inline]
    fn extend_account_infos(
        _program_id: &Pubkey,
        _accounts: Self::CpiAccounts,
        _infos: &mut Vec<AccountInfo>,
        _ctx: &Context,
    ) -> Result<()> {
        Ok(())
    }
    #[inline]
    fn extend_account_metas(
        _program_id: &Pubkey,
        _accounts: &Self::CpiAccounts,
        _metas: &mut Vec<AccountMeta>,
    ) {
    }
}

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
