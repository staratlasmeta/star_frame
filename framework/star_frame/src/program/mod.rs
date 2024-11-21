pub mod system_program;

use crate::instruction::InstructionSet;
use crate::prelude::SolanaRuntime;
use bytemuck::Pod;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
pub use star_frame_proc::StarFrameProgram;

/// A Solana program's definition. This should be derived using the [`StarFrameProgram`](star_frame_proc::StarFrameProgram) macro,
/// since it does more than just implement this trait.
pub trait StarFrameProgram {
    /// The instruction set used by this program.
    type InstructionSet: InstructionSet;

    type AccountDiscriminant: Pod + Eq;

    const PROGRAM_ID: Pubkey;

    /// The entrypoint for the program. This has the same signature as the Solana program entrypoint, and
    /// is called by [`star_frame_entrypoint`](crate::star_frame_entrypoint) macro.
    fn processor(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        Self::InstructionSet::handle_ix(
            program_id,
            accounts,
            instruction_data,
            &mut SolanaRuntime::new(*program_id),
        )
        .map_err(crate::errors::handle_error)
    }
}

/// Macro to define useful top level items for a `star_frame` program. This is called by the [`StarFrameProgram`](star_frame_proc::StarFrameProgram) derive macro.
#[macro_export]
macro_rules! program_setup {
    ($program:ty) => {
        #[allow(dead_code)]
        pub type StarFrameDeclaredProgram = $program;

        #[doc = r" The const program ID."]
        pub const ID: $crate::prelude::Pubkey =
            <$program as $crate::program::StarFrameProgram>::PROGRAM_ID;

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
