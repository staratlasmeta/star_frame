#![feature(ptr_metadata)]
#![feature(pointer_byte_offsets)]
#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    unsafe_op_in_unsafe_fn
)]
#![feature(type_name_of_val)]
#![feature(more_qualified_paths)]

pub extern crate advance;
pub extern crate borsh;
pub extern crate bytemuck;
pub extern crate derivative;
pub extern crate self as star_frame;
pub extern crate solana_program;
#[cfg(feature = "idl")]
pub extern crate star_frame_idl;
pub extern crate static_assertions;

pub mod account_set;
pub mod align1;
pub mod anchor_replacement;
#[cfg(feature = "idl")]
pub mod idl;
pub mod impls;
pub mod instruction;
pub mod packed_value;
pub mod program;
pub mod program_account;
pub mod serialize;
pub mod sys_calls;
pub mod unit_enum_from_repr;
pub mod util;

pub use solana_program::instruction::Instruction as SolanaInstruction;
pub use star_frame_proc::{declare_id, pubkey};

pub type Result<T, E = solana_program::program_error::ProgramError> = std::result::Result<T, E>;

#[allow(unused_imports)]
#[cfg(test)]
use tests::StarFrameDeclaredProgram;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl::{declare_program_type, ProgramToIdl};
    use crate::program::{ProgramIds, StarFrameProgram};
    use star_frame_idl::{IdlDefinition, Version};

    pub struct MyProgram;

    declare_program_type!(MyProgram);

    impl StarFrameProgram for MyProgram {
        type InstructionSet<'a> = ();
        type InstructionDiscriminant = ();
        const PROGRAM_IDS: ProgramIds = todo!();
        type AccountDiscriminant = ();
        const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();
    }

    impl ProgramToIdl for MyProgram {
        const VERSION: Version = Version {
            major: 0,
            minor: 0,
            patch: 0,
        };
        fn idl_namespace() -> &'static str {
            "my_program"
        }
        fn program_to_idl() -> Result<IdlDefinition> {
            todo!()
        }
    }
}
