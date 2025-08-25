use crate::prelude::*;

/// An [`InstructionSet`] that errors when called.
#[derive(Align1, Debug, Copy, Clone)]
pub struct UnCallable;

impl InstructionSet for UnCallable {
    type Discriminant = ();

    fn dispatch(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _ix_bytes: &[u8],
        _ctx: &mut Context,
    ) -> Result<()> {
        panic!("Cannot call dispatch on Uncallable")
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::IdlDefinition;

    impl InstructionSetToIdl for UnCallable {
        fn instruction_set_to_idl(_idl_definition: &mut IdlDefinition) -> Result<()> {
            Ok(())
        }
    }
}
