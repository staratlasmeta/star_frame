//! Data that can be read as unsized.

use crate::Result;
use bytemuck::{from_bytes, from_bytes_mut, AnyBitPattern, NoUninit};
use common_utils::align1::Align1;
use common_utils::Advance;
use std::mem::size_of;

/// Data that can be unsized.
///
/// # Safety
/// All from functions must return valid meta, same pointer as given, and advance the byte pointer by the size of return.
pub unsafe trait UnsizedData: 'static + Align1 {
    /// Metadata for the unsized data to be able to construct sub-contexts.
    /// Usually should be [`()`]
    type Metadata: 'static;

    /// Gets the minimum data size for this data.
    /// The size should be such that the data will always be invalid if the size is less than this.
    fn min_data_size() -> usize;
    /// Gets this data from the given bytes.
    /// Will return the same pointer.
    /// Will advance the bytes by the same amount as [`size_of_val`](std::mem::size_of_val) returns.
    fn from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<(&'a Self, Self::Metadata)>;
    /// Gets this data from the given mutable bytes.
    /// Will return the same pointer.
    /// Will advance the bytes by the same amount as [`size_of_val`](std::mem::size_of_val) returns.
    fn from_mut_bytes<'a>(bytes: &mut &'a mut [u8]) -> Result<(&'a mut Self, Self::Metadata)>;
}

// Safety: Pointers are the same as input.
unsafe impl<T: NoUninit + AnyBitPattern + Align1> UnsizedData for T {
    type Metadata = ();

    fn min_data_size() -> usize {
        size_of::<Self>()
    }

    fn from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<(&'a Self, Self::Metadata)> {
        Ok((from_bytes(bytes.try_advance(size_of::<T>())?), ()))
    }

    fn from_mut_bytes<'a>(bytes: &mut &'a mut [u8]) -> Result<(&'a mut Self, Self::Metadata)> {
        Ok((from_bytes_mut(bytes.try_advance(size_of::<T>())?), ()))
    }
}
