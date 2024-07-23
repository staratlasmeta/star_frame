use crate::instruction::InstructionSet;
use crate::prelude::SysCalls;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

impl InstructionSet for () {
    type Discriminant = ();

    fn handle_ix(
        _ix_bytes: &[u8],
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _sys_calls: &mut impl SysCalls,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
