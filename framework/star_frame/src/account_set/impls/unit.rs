use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::syscalls::SyscallInvoke;
use crate::Result;
use solana_program::account_info::AccountInfo;

impl<'info> AccountSet<'info> for () {}
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
