use crate::instruction::InstructionSet;
use crate::serialize::FrameworkFromBytes;
use crate::sys_calls::SysCalls;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::serialize::FrameworkSerialize;
use star_frame_proc::Align1;

#[derive(Align1, Debug, Copy, Clone)]
pub struct UnCallable;

impl FrameworkSerialize for UnCallable {
    fn to_bytes(&self, _output: &mut &mut [u8]) -> Result<()> {
        panic!("Cannot call `to_bytes` on Uncallable")
    }
}
unsafe impl<'a> FrameworkFromBytes<'a> for UnCallable {
    fn from_bytes(_bytes: &mut &'a [u8]) -> Result<Self> {
        panic!("Cannot call `from_bytes` on Uncallable")
    }
}

impl InstructionSet for UnCallable {
    type Discriminant = ();

    fn handle_ix(
        _ix_bytes: &[u8],
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        panic!("Cannot call handle_ix on Uncallable")
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::idl::InstructionSetToIdl;
    use star_frame_idl::IdlDefinition;

    impl InstructionSetToIdl for UnCallable {
        fn instruction_set_to_idl(_idl_definition: &mut IdlDefinition) -> Result<()> {
            Ok(())
        }
    }
}
