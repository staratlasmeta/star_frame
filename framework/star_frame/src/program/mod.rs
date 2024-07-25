pub mod system_program;

use crate::instruction::InstructionSet;
use bytemuck::Pod;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
pub use star_frame_proc::StarFrameProgram;
use crate::prelude::SolanaRuntime;

/// A Solana program's definition. This should be derived using the [`StarFrameProgram`](star_frame_proc::StarFrameProgram) macro,
/// since it does more than just implement this trait.
pub trait StarFrameProgram {
    /// The instruction set used by this program.
    type InstructionSet: InstructionSet;
    type InstructionDiscriminant = <Self::InstructionSet as InstructionSet>::Discriminant;

    type AccountDiscriminant: Pod + Eq;
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant;

    const PROGRAM_ID: Pubkey;

    fn processor(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        Self::InstructionSet::handle_ix(
            instruction_data,
            program_id,
            accounts,
            &mut SolanaRuntime::new(program_id),
        )
            .map_err(crate::errors::handle_error)
    }
}
