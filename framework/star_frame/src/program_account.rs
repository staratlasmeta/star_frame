use crate::program::ProgramIds;
use star_frame::program::Program;

pub trait ProgramAccount {
    type OwnerProgram: Program;

    fn discriminant() -> <Self::OwnerProgram as Program>::InstructionDiscriminant;

    fn owner_program_id() -> ProgramIds {
        Self::OwnerProgram::program_id()
    }
}
