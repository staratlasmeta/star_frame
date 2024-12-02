use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::prelude::{ClientAccountSet, CpiAccountSet, SyscallAccountCache};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

impl<'info> AccountSet<'info> for () {
    #[inline]
    fn set_account_cache(&mut self, _syscalls: &mut impl SyscallAccountCache<'info>) {}
}

impl<'info> CpiAccountSet<'info> for () {
    type CpiAccounts<'a> = ();
    const MIN_LEN: usize = 0;
    #[inline]
    fn extend_account_infos(
        _accounts: Self::CpiAccounts<'info>,
        _infos: &mut Vec<AccountInfo<'info>>,
    ) {
    }
    #[inline]
    fn extend_account_metas(
        _program_id: &Pubkey,
        _accounts: &Self::CpiAccounts<'info>,
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

impl<'a, 'info> AccountSetDecode<'a, 'info, ()> for () {
    fn decode_accounts(
        _accounts: &mut &'a [AccountInfo],
        decode_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self> {
        Ok(decode_input)
    }
}
impl<'info> AccountSetValidate<'info, ()> for () {
    fn validate_accounts(
        &mut self,
        validate_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        Ok(validate_input)
    }
}

impl<'info> AccountSetCleanup<'info, ()> for () {
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<()> {
        Ok(cleanup_input)
    }
}
