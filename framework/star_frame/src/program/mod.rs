pub mod system_program;

use crate::instruction::InstructionSet;
use bytemuck::Pod;
use solana_program::pubkey::Pubkey;
pub use star_frame_proc::StarFrameProgram;

/// A Solana program's definition.
pub trait StarFrameProgram {
    /// The instruction set used by this program.
    type InstructionSet<'a>: InstructionSet;
    type InstructionDiscriminant = <Self::InstructionSet<'static> as InstructionSet>::Discriminant;

    type AccountDiscriminant: Pod + Eq;
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant;

    const PROGRAM_ID: Pubkey;
}
