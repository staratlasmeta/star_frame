#![feature(ptr_metadata)]
#![warn(
    missing_debug_implementations,
    missing_copy_implementations,
    clippy::pedantic,
    missing_docs,
    clippy::undocumented_unsafe_blocks
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::wildcard_imports,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expl_impl_clone_on_copy,
    clippy::trait_duplication_in_bounds,
    clippy::type_repetition_in_bounds,
    clippy::result_large_err,
    clippy::mut_mut
)]

//! common utilities for `scream`

// This is just for tests, should not be used otherwise.
declare_id!("HSYTaTeNpVV78Mysp6C2wL9DecHU77xSE5ASW1Bhzdjd");

extern crate self as common_utils;

mod advance;
mod binary_heap;
mod boxed_error;
mod chain_exact;
#[cfg(feature = "client")]
mod client;
mod data_size;
mod error;
mod fixed_chunks;
mod normalize_rent;
mod option_flat_map;
mod safe_zero_copy;
mod strong_type;
mod to_seeds;
mod token;
mod unpacked;
mod zero_copy_wrapper;

pub mod align1;
pub mod custom_clock;
pub mod prelude;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod util;
pub mod versioned_account;

pub use advance::*;
pub use binary_heap::*;
pub use boxed_error::*;
pub use data_size::*;
pub use error::*;
pub use fixed_chunks::*;
pub use normalize_rent::*;
pub use option_flat_map::*;
pub use safe_zero_copy::*;
pub use strong_type::*;
pub use to_seeds::*;
pub use token::*;
pub use unpacked::*;
pub use zero_copy_wrapper::*;

pub use anchor_lang;
pub use bytemuck;
pub use common_proc::*;
pub use itertools;
pub use static_assertions;

pub use common_proc::enum_refs;

use bytemuck::{Pod, Zeroable};
use common_utils::prelude::*;
use derivative::Derivative;
use std::fmt::Debug;

/// A Single packed value.
#[derive(Copy, Derivative, Align1)]
#[derivative(
    Debug(bound = "T: Debug + Copy"),
    Clone(bound = "T: Copy"),
    PartialEq(bound = "T: PartialEq + Copy"),
    Eq(bound = "T: Eq + Copy"),
    PartialOrd(bound = "T: PartialOrd + Copy"),
    Ord(bound = "T: Ord + Copy")
)]
#[repr(C, packed)]
pub struct PackedValue<T>(pub T);
// Safety: This is safe because `PackedValue` is `#[repr(C, packed)]` around `T` and `T` is `Zeroable`
unsafe impl<T> Zeroable for PackedValue<T> where T: Zeroable {}
// Safety: This is safe because `PackedValue` is `#[repr(C, packed)]` around `T`
unsafe impl<T> Pod for PackedValue<T> where T: Pod {}
impl<T> DataSize for PackedValue<T>
where
    T: DataSize,
{
    const MIN_DATA_SIZE: usize = T::MIN_DATA_SIZE;
}
