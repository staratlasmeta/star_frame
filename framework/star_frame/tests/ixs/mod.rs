use crate::ixs::test_ix1::TestInstruction1;
use crate::ixs::test_ix2::TestInstruction2;
use star_frame::serialize::{FrameworkFromBytes, FrameworkSerialize};
use star_frame::unit_enum_from_repr::UnitEnumFromRepr;
use star_frame::Result;
use star_frame_proc::{instruction_set2, InstructionSetToIdl};
use strum::EnumDiscriminants;

pub mod test_ix1;
pub mod test_ix2;

impl<'a> FrameworkSerialize for TestProgramInstructions<'a> {
    fn to_bytes(&self, _output: &mut &mut [u8]) -> Result<()> {
        todo!()
    }
}

unsafe impl<'a> FrameworkFromBytes<'a> for TestProgramInstructions<'a> {
    fn from_bytes(_bytes: &mut &'a [u8]) -> Result<Self> {
        todo!()
    }
}

#[instruction_set2]
#[derive(EnumDiscriminants, InstructionSetToIdl)]
#[strum_discriminants(repr(u32), derive(UnitEnumFromRepr))]
#[repr(u32)]
pub enum TestProgramInstructions {
    /// The first test instruction
    TestInstruction1(TestInstruction1),
    /// The second test instruction
    TestInstruction2(TestInstruction2),
}
