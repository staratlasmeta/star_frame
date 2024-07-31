use crate::instruction::InstructionSet;
use crate::prelude::Syscalls;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

impl InstructionSet for () {
    type Discriminant = ();

    fn handle_ix(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _ix_bytes: &[u8],
        _syscalls: &mut impl Syscalls,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
