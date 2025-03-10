use crate::instruction::InstructionSet;
use crate::prelude::Syscalls;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

impl InstructionSet for () {
    type Discriminant = ();

    fn handle_ix<'info>(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo<'info>],
        _ix_bytes: &[u8],
        _syscalls: &mut impl Syscalls<'info>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::InstructionSetToIdl;
    use star_frame_idl::IdlDefinition;

    impl InstructionSetToIdl for () {
        fn instruction_set_to_idl(_idl_definition: &mut IdlDefinition) -> anyhow::Result<()> {
            Ok(())
        }
    }
}
