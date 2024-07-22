#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
use crate::instruction::InstructionSet;
#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
use crate::program::StarFrameProgram;
#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
use crate::Result;
#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
use solana_program::account_info::AccountInfo;
#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
use solana_program::pubkey::Pubkey;

#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
pub fn try_star_frame_entrypoint<T: StarFrameProgram>(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<()> {
    let mut syscalls = crate::prelude::SolanaRuntime::new(program_id);
    T::InstructionSet::handle_ix(instruction_data, program_id, accounts, &mut syscalls)
}

/// Macro to define the entrypoint for a `star_frame` program. This wraps the default solana entrypoint macro
/// and only needs to take in the [`StarFrameProgram`] type.
#[macro_export(local_inner_macros)]
macro_rules! star_frame_entrypoint (
    ($program:ident) => {
        #[cfg(all(not(feature = "no-entrypoint"), any(target_os = "solana", feature = "fake_solana_os")))]
        mod __entrypoint {
            use super::$program;
            fn process_instruction(
                program_id: &$crate::prelude::Pubkey,
                accounts: &[$crate::prelude::AccountInfo],
                instruction_data: &[u8],
            ) -> $crate::solana_program::entrypoint::ProgramResult {
                $crate::entrypoint::try_star_frame_entrypoint::<$program>(program_id, accounts, instruction_data)
                    .map_err($crate::errors::handle_error)
            }
            $crate::solana_program::entrypoint!(process_instruction);
        }
    };
);
