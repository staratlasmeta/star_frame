/// Macro to define the entrypoint for a `star_frame` program. This wraps the default [`solana_program::entrypoint!`] macro
/// and only needs to take in the [`StarFrameProgram`](crate::prelude::StarFrameProgram) type. This will be automatically called by the
/// [`StarFrameProgram`](star_frame_proc::StarFrameProgram) derive macro if the `no_entrypoint` argument
/// is not present.
///
/// # Example
/// ```
/// # #[macro_use] extern crate star_frame;
/// # fn main() {}
/// use star_frame::prelude::*;
///
/// #[derive(StarFrameProgram)]
/// #[program(
///     id = SystemProgram::PROGRAM_ID,
///     instruction_set = (),
/// // By default, the `StarFrameProgram` derive macro will already call `star_frame_entrypoint`
///     no_entrypoint
/// )]
/// pub struct MyProgram;
///
/// star_frame_entrypoint!(MyProgram);
/// ```
#[macro_export(local_inner_macros)]
macro_rules! star_frame_entrypoint (
    ($program:ty) => {
        // todo: should this be public?
        #[doc(hidden)]
        pub mod _entrypoint {
            use super::*;
            pub fn process_instruction<'info>(
                program_id: &'info $crate::prelude::Pubkey,
                accounts: &[$crate::prelude::AccountInfo<'info>],
                instruction_data: &[u8],
            ) -> $crate::solana_program::entrypoint::ProgramResult {
                <$program as $crate::program::StarFrameProgram>::processor(program_id, accounts, instruction_data)
            }
            #[cfg(not(feature = "no-entrypoint"))]
            $crate::solana_program::entrypoint!(process_instruction);
        }
    };
);
