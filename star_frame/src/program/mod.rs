//! Core program definitions and utilities for Star Frame programs. Provides the foundational traits and macros needed to define and execute Solana programs with type safety.

pub mod system;

use crate::prelude::*;

pub use star_frame_proc::StarFrameProgram;

/// A Solana program's definition and the main entrypoint in to a Star Frame program. This should be derived using the [`StarFrameProgram`](derive@StarFrameProgram) macro,
/// since it does more than just implement this trait.
pub trait StarFrameProgram {
    /// The instruction set used by this program.
    type InstructionSet: InstructionSet;

    type AccountDiscriminant: Pod + Eq;

    const ID: Pubkey;

    /// The entrypoint for the program which calls in to [`InstructionSet::dispatch`] on [`Self::InstructionSet`]. This has the same signature as the Solana program entrypoint, and
    /// is called by [`star_frame_entrypoint`](crate::star_frame_entrypoint) macro.
    #[inline]
    fn entrypoint(
        program_id: &'static pinocchio::pubkey::Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let program_id = bytemuck::cast_ref(program_id);
        Self::InstructionSet::dispatch(program_id, accounts, instruction_data)
            .map_err(crate::errors::handle_error)
    }
}

/// Defines useful top level items for a `star_frame` program.
///
/// This is called by the [`StarFrameProgram`](star_frame_proc::StarFrameProgram) derive macro.
#[macro_export]
macro_rules! program_setup {
    ($program:ty) => {
        #[allow(dead_code)]
        pub type StarFrameDeclaredProgram = $program;

        #[doc = r" The const program ID."]
        pub const ID: $crate::prelude::Pubkey = <$program as $crate::program::StarFrameProgram>::ID;

        #[doc = r" Returns `true` if given pubkey is the program ID."]
        pub fn check_id(id: &$crate::prelude::Pubkey) -> bool {
            id == &ID
        }

        #[doc = r" Returns the program ID."]
        pub const fn id() -> $crate::prelude::Pubkey {
            ID
        }

        #[test]
        fn test_id() {
            assert!(check_id(&id()));
        }
    };
}
