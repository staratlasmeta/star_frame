pub mod system_program;

use crate::instruction::InstructionSet;
use bytemuck::Pod;
use solana_program::pubkey::Pubkey;
pub use star_frame_proc::StarFrameProgram;

/// A Solana program's definition. This should be derived using the [`StarFrameProgram`](star_frame_proc::StarFrameProgram) macro,
/// since it does more than just implement this trait.
pub trait StarFrameProgram {
    /// The instruction set used by this program.
    type InstructionSet: InstructionSet;
    type InstructionDiscriminant = <Self::InstructionSet as InstructionSet>::Discriminant;

    type AccountDiscriminant: Pod + Eq;
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant;

    const PROGRAM_ID: Pubkey;
}
