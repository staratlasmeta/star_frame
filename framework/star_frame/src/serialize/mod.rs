pub mod borsh;
pub mod combined_unsized;
pub mod list;
pub mod ref_wrapper;
#[cfg(feature = "test_helpers")]
pub mod test_helpers;
pub mod unsize;

use crate::align1::Align1;
use crate::Result;
use advance::Advance;
use bytemuck::{checked, CheckedBitPattern, NoUninit};
use std::mem::size_of;

pub trait StarFrameSerialize {
    /// Writes this type to a set of bytes.
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()>;
}
impl<'a, T> StarFrameSerialize for &'a T
where
    T: CheckedBitPattern + NoUninit,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        output
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(*self));
        Ok(())
    }
}
impl<'a, T> StarFrameSerialize for &'a mut T
where
    T: CheckedBitPattern + NoUninit,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        output
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(*self));
        Ok(())
    }
}

/// Writes this type to a set of bytes and reads this type from bytes.
///
/// # Safety
/// If `Self` is pointer type [`from_bytes`](StarFrameFromBytes::from_bytes) must return the same pointer that was
/// passed in. Metadata may be different.
pub unsafe trait StarFrameFromBytes<'a>: Sized + StarFrameSerialize {
    /// Deserializes this type from a set of bytes.
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self>;
}
unsafe impl<'a, T> StarFrameFromBytes<'a> for &'a T
where
    T: Align1 + CheckedBitPattern + NoUninit,
{
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self> {
        checked::try_from_bytes(bytes.try_advance(size_of::<T>())?).map_err(Into::into)
    }
}
