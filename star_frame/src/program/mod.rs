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

    const ID: Address;

    /// Handles errors returned from the program and then returns a [`ProgramError`].
    ///
    /// By default, it logs the error with [`Error::log`].
    #[inline]
    #[must_use]
    fn handle_error(error: Error) -> ProgramError {
        error.log();
        error.into()
    }

    /// The entrypoint for the program which calls in to [`InstructionSet::dispatch`] on [`Self::InstructionSet`]. This has the same signature as the Solana program entrypoint, and
    /// is called by [`star_frame_entrypoint`](crate::star_frame_entrypoint) macro.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn entrypoint(
        program_id: &'static Address,
        accounts: &[AccountView],
        instruction_data: &'static [u8],
    ) -> ProgramResult {
        Self::InstructionSet::dispatch(program_id, accounts, instruction_data)
            .map_err(Self::handle_error)
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
        pub const ID: $crate::prelude::Address =
            <$program as $crate::program::StarFrameProgram>::ID;

        #[doc = r" Returns `true` if given address is the program ID."]
        pub fn check_id(id: &$crate::prelude::Address) -> bool {
            id == &ID
        }

        #[doc = r" Returns the program ID."]
        pub const fn id() -> $crate::prelude::Address {
            ID
        }

        #[test]
        fn test_id() {
            assert!(check_id(&id()));
        }
    };
}
