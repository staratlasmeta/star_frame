pub mod system_program;

use crate::instruction::InstructionSet;
use bytemuck::Pod;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
pub use star_frame_proc::StarFrameProgram;
use crate::prelude::SolanaRuntime;

/// A Solana program's definition.
pub trait StarFrameProgram {
    /// The instruction set used by this program.
    type InstructionSet<'a>: InstructionSet;
    type InstructionDiscriminant = <Self::InstructionSet<'static> as InstructionSet>::Discriminant;

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
