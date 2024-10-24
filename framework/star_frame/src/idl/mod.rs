use crate::prelude::*;
use star_frame_idl::account::IdlAccountId;
use star_frame_idl::account_set::IdlAccountSetDef;
use star_frame_idl::instruction::IdlInstructionDef;
use star_frame_idl::{IdlDefinition, IdlMetadata, Version};
pub use star_frame_proc::InstructionToIdl;

mod ty;
pub use ty::*;

pub trait InstructionSetToIdl: InstructionSet {
    /// Returns the idl of this instruction set.
    fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> Result<()>;
}
pub trait InstructionToIdl<A>: Instruction {
    /// Returns the idl of this instruction.
    fn instruction_to_idl(idl_definition: &mut IdlDefinition, arg: A) -> Result<IdlInstructionDef>;
}
pub trait AccountSetToIdl<'info, A>: AccountSet<'info> {
    /// Returns the idl of this account set.
    fn account_set_to_idl(idl_definition: &mut IdlDefinition, arg: A) -> Result<IdlAccountSetDef>;
}
pub trait AccountToIdl: TypeToIdl {
    /// Returns the idl of this account.
    fn account_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlAccountId>;
}

pub trait ProgramToIdl: StarFrameProgram {
    fn version() -> Version;

    fn program_to_idl() -> Result<IdlDefinition>
    where
        <Self as StarFrameProgram>::InstructionSet: InstructionSetToIdl,
    {
        let mut out = IdlDefinition {
            address: Self::PROGRAM_ID,
            metadata: IdlMetadata {
                version: Self::version(),
                ..Default::default()
            },
            ..Default::default()
        };
        <Self as StarFrameProgram>::InstructionSet::instruction_set_to_idl(&mut out)?;
        Ok(out)
    }
}
