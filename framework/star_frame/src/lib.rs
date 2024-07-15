#![feature(ptr_metadata)]
#![cfg_attr(not(rust_1_75), feature(pointer_byte_offsets))]
#![cfg_attr(not(rust_1_76), feature(type_name_of_val))]
#![warn(
    clippy::pedantic,
    missing_copy_implementations,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn
)]
#![allow(
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

pub extern crate advance;
pub extern crate anyhow;
pub extern crate borsh;
pub extern crate bytemuck;
pub extern crate derivative;
pub extern crate fixed;
pub extern crate itertools;
pub extern crate num_traits;
pub extern crate paste;
pub extern crate self as star_frame;
pub extern crate serde;
#[cfg(feature = "idl")]
pub extern crate serde_json;
pub extern crate solana_program;
#[cfg(feature = "idl")]
pub extern crate star_frame_idl;
pub extern crate static_assertions;
pub extern crate typenum;

pub mod account_set;
pub mod align1;
// mod arbitrary_struct;
pub mod divisor;
pub mod entrypoint;
pub mod errors;
pub mod fixed_point;
#[cfg(feature = "idl")]
pub mod idl;
pub mod impls;
pub mod instruction;
pub mod packed_value;
pub mod prelude;
pub mod program;
pub mod serialize;
pub mod sys_calls;
pub mod unit_enum_from_repr;
pub mod unit_val;
pub mod util;

/// Internal paths mainly for use in macros. DO NOT USE MANUALLY. NOT PART OF THE PUBLIC API.
#[doc(hidden)]
pub mod __private;

pub use anyhow::Result;
pub use solana_program::instruction::Instruction as SolanaInstruction;
pub use star_frame_proc::pubkey;

#[allow(unused_imports)]
#[cfg(test)]
use tests::StarFrameDeclaredProgram;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl::ProgramToIdl;
    use crate::program::{ProgramIds, StarFrameProgram};
    use crate::util::Network;
    use solana_program::pubkey::Pubkey;
    use star_frame_idl::{IdlDefinition, Version};
    use star_frame_proc::program;

    #[program(Network::MainnetBeta, no_entrypoint)]
    pub struct MyProgram;

    impl StarFrameProgram for MyProgram {
        type InstructionSet<'a> = ();
        type InstructionDiscriminant = ();
        type AccountDiscriminant = ();
        const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();
        const PROGRAM_IDS: ProgramIds = ProgramIds::AllNetworks(&Pubkey::new_from_array([0; 32]));
    }

    impl ProgramToIdl for MyProgram {
        const VERSION: Version = Version {
            major: 0,
            minor: 0,
            patch: 0,
        };
        fn program_to_idl() -> Result<IdlDefinition> {
            unimplemented!()
        }
        fn idl_namespace() -> &'static str {
            "my_program"
        }
    }
}
