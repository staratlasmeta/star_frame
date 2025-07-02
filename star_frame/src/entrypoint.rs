/// Macro to define the entrypoint for a `star_frame` program. This wraps the default [`pinocchio::entrypoint!`] macro
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
///     id = System::ID,
///     instruction_set = (),
/// // By default, the `StarFrameProgram` derive macro will already call `star_frame_entrypoint`
///     no_entrypoint
/// )]
/// pub struct MyProgram;
///
/// star_frame_entrypoint!(MyProgram);
/// ```
#[macro_export]
macro_rules! star_frame_entrypoint (
    ($program:ty) => {
        // todo: should this be public?
        #[doc(hidden)]
        #[allow(unexpected_cfgs)]
        pub mod _entrypoint {
            use super::*;
            pub fn process_instruction<'info>(
                program_id: &'info [u8; 32],
                accounts: &[$crate::prelude::AccountInfo],
                instruction_data: &[u8],
            ) -> $crate::pinocchio::ProgramResult {
                <$program as $crate::program::StarFrameProgram>::processor(program_id, accounts, instruction_data)
            }
            #[cfg(not(any(feature = "no-entrypoint", feature = "no_entrypoint")))]
            $crate::pinocchio::entrypoint!(process_instruction);
        }
    };
);
