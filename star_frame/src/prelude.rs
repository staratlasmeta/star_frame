//! Commonly used types and traits: `use star_frame::prelude::*`.

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use crate::idl::{
    seed_const, seed_path, AccountSetToIdl, AccountToIdl, InstructionSetToIdl, InstructionToIdl,
    ProgramToIdl, TypeToIdl,
};
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use star_frame_idl::{NodeToJson, ProgramNode};

#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
pub use crate::unsize::{NewByteSet, TestByteSet};

#[cfg(not(target_os = "solana"))]
pub use crate::client::MakeInstruction as _;

pub use crate::{
    account_set::prelude::*,
    address,
    align1::Align1,
    bail, borsh_with_bytemuck,
    client::{
        DeserializeAccount as _, DeserializeBorshAccount as _, DeserializeType as _,
        FindProgramAddress as _, SerializeAccount as _, SerializeBorshAccount as _,
        SerializeType as _,
    },
    context::Context,
    cpi::MakeCpi as _,
    create_unit_system,
    data_types::{
        AddressFor, ClockExt, GetAddressFor as _, GetOptionalAddressFor as _, OptionalAddress,
        OptionalAddressFor, PackedValue, SetAddressFor as _, UnitVal,
    },
    ensure, ensure_eq, ensure_ne, error,
    errors::{star_frame_error, Error, ErrorInfo as _},
    instruction::{
        star_frame_instruction, InstructionArgs, InstructionDiscriminant as _, InstructionSet,
        StarFrameInstruction,
    },
    program::{system::System, StarFrameProgram},
    unsize::prelude::*,
    util::{borsh_bytemuck, FastAddressEq as _},
    Result,
};

// ensure derive macros are in scope
pub use star_frame_proc::{zero_copy, InstructionToIdl, TypeToIdl};

// Solana stuff
pub use pinocchio::{
    account::AccountView, error::ProgramError, instruction::InstructionAccount, ProgramResult,
};

pub use pinocchio_log::log;
pub use solana_address::Address;
#[cfg(not(target_os = "solana"))]
pub use solana_instruction::AccountMeta;
pub use solana_msg::msg;

// bytemuck
pub use bytemuck::{CheckedBitPattern, NoUninit, Pod, Zeroable};

pub use borsh::{self, BorshDeserialize, BorshSerialize};

pub use core::fmt::Debug;

pub use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
