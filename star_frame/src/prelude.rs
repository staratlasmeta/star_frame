pub use crate::{
    account_set::*,
    context::Context,
    data_types::*,
    instruction::*,
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
};

pub use crate::align1::Align1;

pub use crate::client::{
    ClientAccountSet, CpiAccountSet, CpiBuilder, FindProgramAddress, MakeCpi, MakeInstruction,
};

pub use crate::{
    program::{system::System, StarFrameProgram},
    pubkey,
};

pub use crate::Result;

pub use pinocchio::{account_info::AccountInfo, msg, program_error::ProgramError};

pub use solana_instruction::AccountMeta;
pub use solana_pubkey::Pubkey;

pub use crate::{
    anyhow::{self, anyhow, bail, Context as _},
    create_unit_system,
};

// bytemuck
pub use bytemuck::{CheckedBitPattern, NoUninit, Pod, Zeroable};

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use crate::idl::{
    seed_const, seed_path, AccountSetToIdl, AccountToIdl, FindIdlSeeds, InstructionSetToIdl,
    InstructionToIdl, ProgramToIdl, TypeToIdl,
};

// ensure derive macros are in scope
pub use star_frame_proc::{InstructionToIdl, TypeToIdl};

pub use std::fmt::Debug;

#[cfg(all(feature = "test_helpers", not(target_os = "solana")))]
pub use crate::{
    assert_eq_with_shared, assert_with_shared,
    unsize::{NewByteSet, TestByteSet},
};
