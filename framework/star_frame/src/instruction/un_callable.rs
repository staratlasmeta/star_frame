use crate::instruction::{InstructionSet, ToBytes};
use crate::sys_calls::SysCalls;
use crate::Result;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

pub struct UnCallable;

impl ToBytes for UnCallable {
    fn to_bytes(self, _output: &mut &mut [u8]) -> Result<()> {
        panic!("Cannot call to_bytes on Uncallable")
    }
}

impl<'a> InstructionSet<'a> for UnCallable {
    fn from_bytes(_bytes: &'a [u8]) -> Result<Self> {
        panic!("Cannot call from_bytes on Uncallable")
    }

    fn handle_ix(
        self,
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

    impl<'a> InstructionSetToIdl<'a> for UnCallable {
        fn instruction_set_to_idl(_idl_definition: &mut IdlDefinition) -> Result<()> {
            Ok(())
        }
    }
}
