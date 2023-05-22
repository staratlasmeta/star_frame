use crate::SafeZeroCopy;
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use std::fmt::Debug;

/// A [`SafeZeroCopy`] type that can be unpacked for serialization.
pub trait Unpackable: SafeZeroCopy {
    /// The unpacked version of `Self`.
    type Unpacked: Unpacked<Packed = Self>;
    /// Unpacks `self`.
    fn unpack(self) -> Self::Unpacked;
}

/// An unpacked version of a [`SafeZeroCopy`] type.
pub trait Unpacked: Clone + AnchorSerialize + AnchorDeserialize + Debug {
    /// The packed version of `Self`.
    type Packed: Unpackable<Unpacked = Self>;
    /// Packs `self`.
    fn pack(&self) -> Self::Packed;
}
