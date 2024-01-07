use crate::account_set::{AccountSet, AccountSetCleanup, AccountSetDecode, AccountSetValidate};
use crate::instruction::FrameworkSerialize;
use crate::sys_calls::{SysCallInvoke, SysCalls};
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use star_frame::instruction::InstructionSet;

impl<'info> AccountSet<'info> for () {
    fn try_to_accounts<'a, E>(
        &'a self,
        _add_account: impl FnMut(&'a AccountInfo<'info>) -> crate::Result<(), E>,
    ) -> crate::Result<(), E>
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
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self, ProgramError> {
        Ok(decode_input)
    }
}
impl<'info> AccountSetValidate<'info, ()> for () {
    fn validate_accounts(
        &mut self,
        validate_input: (),
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<(), ProgramError> {
        Ok(validate_input)
    }
}

impl<'info> AccountSetCleanup<'info, ()> for () {
    fn cleanup_accounts(
        &mut self,
        cleanup_input: (),
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<(), ProgramError> {
        Ok(cleanup_input)
    }
}

impl FrameworkSerialize for () {
    fn to_bytes(self, _output: &mut &mut [u8]) -> Result<()> {
        Ok(())
    }
    fn from_bytes(_bytes: &[u8]) -> Result<Self> {
        Ok(())
    }
}
impl<'a> InstructionSet<'a> for () {
    type Discriminant = ();

    fn handle_ix(
        self,
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _sys_calls: &mut impl SysCalls,
    ) -> ProgramResult {
        Ok(())
    }
}
