use crate::{instruction::InstructionSet, prelude::Context};
use pinocchio::account_info::AccountInfo;
use solana_pubkey::Pubkey;

impl InstructionSet for () {
    type Discriminant = ();

    fn handle_ix(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _ix_bytes: &[u8],
        _ctx: &mut Context,
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
