use crate::account_set::AccountSet;
use crate::instruction::Instruction;
use crate::program::StarFrameProgram;
use crate::program_account::ProgramAccount;
use crate::Result;
use star_frame::instruction::InstructionSet;
use star_frame_idl::account::AccountId;
use star_frame_idl::account_set::IdlAccountSetDef;
use star_frame_idl::instruction::IdlInstructionDef;
use star_frame_idl::{IdlDefinition, SemVer, Version};

pub mod ty;

pub trait AccountSetToIdl<'info, A>: AccountSet<'info> {
    /// Returns the idl of this account set.
    fn account_set_to_idl(idl_definition: &mut IdlDefinition, arg: A) -> Result<IdlAccountSetDef>;
}

pub trait AccountToIdl: ProgramAccount {
    type AssociatedProgram: ProgramToIdl;

    /// Returns the idl of this account.
    fn account_to_idl(idl_definition: &mut IdlDefinition) -> Result<AccountId>;
    fn account_program_versions() -> SemVer {
        SemVer::from_version(Self::AssociatedProgram::VERSION)
    }
}
pub trait InstructionToIdl<'a, A>: Instruction<'a> {
    /// Returns the idl of this instruction.
    fn instruction_to_idl(idl_definition: &mut IdlDefinition, arg: A) -> Result<IdlInstructionDef>;
}
pub trait InstructionSetToIdl<'a>: InstructionSet<'a> {
    /// Returns the idl of this instruction set.
    fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> Result<()>;
}
pub trait ProgramToIdl: StarFrameProgram {
    const VERSION: Version;
    fn program_to_idl() -> Result<IdlDefinition>;
    fn idl_namespace() -> &'static str;
}
