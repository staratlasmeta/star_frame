#![feature(ptr_metadata)]

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
pub mod sys_calls;
pub mod unit_enum_from_repr;
pub mod util;

pub extern crate self as star_frame;
pub extern crate solana_program;
#[cfg(feature = "idl")]
pub extern crate star_frame_idl;
pub extern crate static_assertions;

pub use solana_program::instruction::Instruction as SolanaInstruction;

pub type Result<T, E = solana_program::program_error::ProgramError> = std::result::Result<T, E>;
