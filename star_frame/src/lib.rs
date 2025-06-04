//! Anchor replacement.
// TODO: Expand docs
#![warn(
    clippy::pedantic,
    missing_copy_implementations,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn,
    // missing_docs
)]
#![allow(
    unexpected_cfgs,
    clippy::non_canonical_clone_impl,
    clippy::default_trait_access,
    clippy::manual_string_new,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::mut_mut,
    clippy::wildcard_imports,
    clippy::expl_impl_clone_on_copy,
    clippy::non_canonical_partial_ord_impl
)]

pub extern crate advancer;
pub extern crate anyhow;
pub extern crate borsh;
pub extern crate bytemuck;
pub extern crate derive_more;
pub extern crate derive_where;
pub extern crate fixed;
pub extern crate itertools;
pub extern crate num_traits;
pub extern crate paste;
pub extern crate pinocchio;
pub extern crate self as star_frame;
pub extern crate serde;
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub extern crate serde_json;
pub extern crate solana_instruction;
pub extern crate solana_pubkey;
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub extern crate star_frame_idl;
pub extern crate static_assertions;
pub extern crate typenum;

pub mod account_set;
pub mod align1;
pub mod client;
pub mod data_types;
pub mod entrypoint;
pub mod errors;

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl;
pub mod instruction;
pub mod prelude;
pub mod program;
pub mod syscalls;
pub mod unsize;
pub mod util;

/// Internal paths mainly for use in macros. DO NOT USE MANUALLY. NOT PART OF THE PUBLIC API.
#[doc(hidden)]
pub mod __private;

pub use anyhow::Result;
pub use solana_instruction::Instruction as SolanaInstruction;
pub use star_frame_proc::pubkey;
pub use star_frame_proc::sighash;

#[allow(unused_imports)]
#[cfg(test)]
use tests::StarFrameDeclaredProgram;

#[cfg(all(not(feature = "test_helpers"), any(doctest, test)))]
compile_error!("You must enable the `test_helpers` feature for running tests!");

#[cfg(all(test, feature = "test_helpers"))]
mod tests {
    use super::*;
    use crate::program::StarFrameProgram;
    use solana_pubkey::Pubkey;

    #[derive(StarFrameProgram)]
    #[program(
        instruction_set = (),
        id = Pubkey::new_from_array([0; 32]),
        no_entrypoint,
    )]
    pub struct MyProgram;

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    #[test]
    fn test_idl() {
        use crate::idl::ProgramToIdl;
        let idl = MyProgram::program_to_idl().unwrap();
        println!("{}", serde_json::to_string_pretty(&idl).unwrap());
    }
}
