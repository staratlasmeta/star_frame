use crate::program::ProgramIds;
use star_frame::program::StarFrameProgram;

pub trait ProgramAccount {
    type OwnerProgram: StarFrameProgram;

    fn discriminant() -> <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant;

    fn owner_program_id() -> ProgramIds {
        Self::OwnerProgram::PROGRAM_IDS
    }
}
