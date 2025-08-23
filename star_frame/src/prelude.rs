#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use crate::idl::{
    seed_const, seed_path, AccountSetToIdl, AccountToIdl, FindIdlSeeds, InstructionSetToIdl,
    InstructionToIdl, ProgramToIdl, TypeToIdl,
};
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use star_frame_idl::{NodeTrait as _, ProgramNode};

#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
pub use crate::{
    assert_eq_with_shared, assert_with_shared,
    unsize::{NewByteSet, TestByteSet},
};

pub use crate::{
    account_set::*,
    align1::Align1,
    anyhow::{self, anyhow, bail, Context as _},
    client::{
        ClientAccountSet, CpiAccountSet, CpiBuilder, FindProgramAddress, MakeCpi, MakeInstruction,
    },
    context::Context,
    create_unit_system,
    data_types::*,
    instruction::*,
    program::{system::System, StarFrameProgram},
    pubkey,
    unsize::{
        impls::*,
        init::{DefaultInit, UnsizedInit},
        unsized_impl, unsized_type,
        wrapper::{
            ExclusiveRecurse, ExclusiveWrapper, ExclusiveWrapperTop, SharedWrapper,
            UnsizedTypeDataAccess,
        },
        AsShared, FromOwned, UnsizedType,
    },
    util::borsh_bytemuck,
    Result,
};

// ensure derive macros are in scope
pub use star_frame_proc::{InstructionToIdl, TypeToIdl};

// Solana stuff
pub use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError, ProgramResult};
pub use solana_instruction::AccountMeta;
pub use solana_pubkey::Pubkey;

// bytemuck
pub use bytemuck::{CheckedBitPattern, NoUninit, Pod, Zeroable};

pub use borsh::{self, BorshDeserialize, BorshSerialize};

pub use std::fmt::Debug;
