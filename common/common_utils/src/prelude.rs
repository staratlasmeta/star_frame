//! Prelude for easy importing

pub use crate::{
    binary_heap::BinaryHeap, boxed_error::Result, boxed_error::*, chain_exact::ChainExact,
    define_unit, err, error, normalize_rent, require, require_eq, require_gt, require_gte,
    require_keys_eq, require_keys_neq, require_neq, safe_zero_copy, safe_zero_copy_account,
    strong_type::*, Advance, AdvanceArray, Bytes, DataSize, DivUnit, List, ListLength, MulUnit,
    OptionFlatMap, PackedValue, RemainingData, RemainingDataWithArg, SafeZeroCopy,
    SafeZeroCopyAccount, Seeds, StrongTypedAccountLoader, StrongTypedStruct, UnitEnumFromRepr,
    UnitType, Unitless, UnorderedList, Unpackable, Unpacked, UtilError, WrappableAccount,
    WrapperList, WrapperListUnorderedList, ZeroCopyWrapper,
};
pub use anchor_lang::{prelude::*, InstructionData};
pub use anchor_spl::token::{self, Mint, Token, TokenAccount};
pub use bitflags::bitflags;
pub use bytemuck::{bytes_of, bytes_of_mut, checked, from_bytes, from_bytes_mut};
pub use derivative::Derivative;
pub use derive_more;
pub use itertools::Itertools;
pub use num_traits::ToPrimitive;
pub use solana_program::system_program;

#[cfg(feature = "client")]
pub use __testing::*;
#[cfg(feature = "client")]
mod __testing {
    pub use crate::client::*;
    pub use anyhow;
    pub use async_trait;
    pub use futures;
    pub use solana_account_decoder;
    pub use solana_client;
    pub use solana_sdk;
    pub use solana_transaction_status;
    pub use tokio;

    pub use solana_client::nonblocking::rpc_client::RpcClient;
    pub use solana_sdk::instruction::{AccountMeta, Instruction};
    pub use solana_sdk::signature::Signer as SignerFunctions;
    pub use solana_sdk::signature::{Keypair, Signature};
}
