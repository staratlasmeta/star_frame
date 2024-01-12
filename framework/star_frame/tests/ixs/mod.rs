use crate::ixs::test_ix1::TestInstruction1;
use crate::ixs::test_ix2::TestInstruction2;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use star_frame::idl::{InstructionSetToIdl, InstructionToIdl};
use star_frame::instruction::InstructionSet;
use star_frame::sys_calls::SysCalls;
use star_frame::unit_enum_from_repr::UnitEnumFromRepr;
use star_frame::Result;
use star_frame_idl::instruction::IdlInstruction;
use star_frame_idl::IdlDefinition;
use strum::EnumDiscriminants;

pub mod test_ix1;
pub mod test_ix2;

#[derive(EnumDiscriminants)]
#[strum_discriminants(repr(u32), derive(UnitEnumFromRepr))]
pub enum TestProgramInstructions<'a> {
    TestInstruction1(&'a TestInstruction1),
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

impl<'a> InstructionSetToIdl<'a> for TestProgramInstructions<'a> {
    fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> Result<()> {
        let test_instruction_1 = <&'a TestInstruction1>::instruction_to_idl(idl_definition, ())?;
        idl_definition.instructions.insert(
            "TestInstruction1".to_string(),
            IdlInstruction {
                name: "Test Instruction 1".to_string(),
                description: "The first test instruction".to_string(),
                discriminant: serde_json::to_value(
                    TestProgramInstructionsDiscriminants::TestInstruction1.into_repr(),
                )
                .expect("Cannot serialize u32?"),
                definition: test_instruction_1,
                extension_fields: Default::default(),
            },
        );
        let test_instruction_2 = <&'a TestInstruction2>::instruction_to_idl(idl_definition, ())?;
        idl_definition.instructions.insert(
            "TestInstruction2".to_string(),
            IdlInstruction {
                name: "Test Instruction 2".to_string(),
                description: "The second test instruction".to_string(),
                discriminant: serde_json::to_value(
                    TestProgramInstructionsDiscriminants::TestInstruction2.into_repr(),
                )
                .expect("Cannot serialize u32?"),
                definition: test_instruction_2,
                extension_fields: Default::default(),
            },
        );
        Ok(())
    }
}
