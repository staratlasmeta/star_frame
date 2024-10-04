use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;

impl<'info> AccountSet<'info> for () {
    fn try_to_accounts<'a, E>(
        &'a self,
        _add_account: impl FnMut(&'a AccountInfo<'info>) -> Result<(), E>,
    ) -> Result<(), E>
    where
        'info: 'a,
    {
        Ok(())
    }

    fn to_account_metas(&self, _add_account_meta: impl FnMut(AccountMeta)) {}
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
