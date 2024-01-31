use crate::ixs::test_ix1::TestInstruction1;
use crate::ixs::test_ix2::TestInstruction2;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::instruction::InstructionSet;
use star_frame::sys_calls::SysCalls;
use star_frame::unit_enum_from_repr::UnitEnumFromRepr;
use star_frame::Result;
use strum::EnumDiscriminants;

pub mod test_ix1;
pub mod test_ix2;

#[derive(EnumDiscriminants, InstructionSet)]
#[strum_discriminants(repr(u32), derive(UnitEnumFromRepr))]
#[repr(u8)]
pub enum TestProgramInstructions<'a> {
    /// The first test instruction
    TestInstruction1(&'a TestInstruction1),
    /// The second test instruction
    TestInstruction2(&'a TestInstruction2),
}

impl<'a> InstructionSet<'a> for TestProgramInstructions<'a> {
    type Discriminant = u32;

    fn handle_ix(
        self,
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        todo!()
    }
}
