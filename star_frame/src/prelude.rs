//! Commonly used types and traits: `use star_frame::prelude::*`.

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use crate::idl::{
    seed_const, seed_path, AccountSetToIdl, AccountToIdl, InstructionSetToIdl, InstructionToIdl,
    ProgramToIdl, TypeToIdl,
};
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use star_frame_idl::{NodeTrait as _, ProgramNode};

#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
pub use crate::{
    assert_eq_with_shared, assert_with_shared,
    unsize::{NewByteSet, TestByteSet},
};

pub use crate::{
    account_set::prelude::*,
    align1::Align1,
    borsh_with_bytemuck,
    client::{
        DeserializeAccount as _, DeserializeBorshAccount as _, DeserializeType as _,
        FindProgramAddress as _, MakeInstruction as _, SerializeAccount as _,
        SerializeBorshAccount as _, SerializeType as _,
    },
    context::Context,
    cpi::MakeCpi as _,
    create_unit_system,
    data_types::{
        ClockExt, GetKeyFor as _, GetOptionalKeyFor as _, KeyFor, OptionalKeyFor, OptionalPubkey,
        PackedValue, SetKeyFor as _, UnitVal,
    },
    ensure_eq, ensure_ne,
    instruction::{
        star_frame_instruction, InstructionArgs, InstructionDiscriminant as _, InstructionSet,
        StarFrameInstruction,
    },
    program::{system::System, StarFrameProgram},
    pubkey,
    unsize::prelude::*,
    util::{borsh_bytemuck, FastPubkeyEq as _},
    Result,
};

// ensure derive macros are in scope
pub use star_frame_proc::{zero_copy, InstructionToIdl, TypeToIdl};

// Solana stuff
pub use eyre::{bail, ensure, eyre};
pub use pinocchio::{
    account_info::AccountInfo, instruction::AccountMeta as PinocchioAccountMeta, msg,
    program_error::ProgramError, ProgramResult,
};
pub use solana_instruction::AccountMeta;
pub use solana_pubkey::Pubkey;

// bytemuck
pub use bytemuck::{CheckedBitPattern, NoUninit, Pod, Zeroable};

pub use borsh::{self, BorshDeserialize, BorshSerialize};

pub use std::fmt::Debug;
