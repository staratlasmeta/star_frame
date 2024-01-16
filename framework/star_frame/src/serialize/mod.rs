pub mod borsh;
pub mod combined_unsized;
pub mod pointer_breakup;
pub mod serialize_with;

use crate::align1::Align1;
use crate::Result;
use advance::Advance;
use bytemuck::{from_bytes, from_bytes_mut, Pod};
use star_frame::serialize::pointer_breakup::PointerBreakup;
use std::mem::size_of;

pub trait ResizeFn<'a, M>: FnMut(usize, M) -> Result<()> + 'a {}
impl<'a, T, M> ResizeFn<'a, M> for T where T: FnMut(usize, M) -> Result<()> + 'a {}

pub trait FrameworkSerialize {
    /// Writes this type to a set of bytes.
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()>;
}
impl<'a, T> FrameworkSerialize for &'a T
where
    T: Align1 + Pod,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        output
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(*self));
        Ok(())
    }
}
impl<'a, T> FrameworkSerialize for &'a mut T
where
    T: Align1 + Pod,
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
/// If `Self` is pointer type [`from_bytes`](FrameworkFromBytes::from_bytes) must return the same pointer that was
/// passed in. Metadata may be different.
pub unsafe trait FrameworkFromBytes<'a>: Sized + FrameworkSerialize {
    /// Deserializes this type from a set of bytes.
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self>;
}

unsafe impl<'a, T> FrameworkFromBytes<'a> for &'a T
where
    T: Align1 + Pod,
{
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self> {
        Ok(from_bytes(bytes.try_advance(size_of::<T>())?))
    }
}

/// Allows this type to be referenced mutably from a set of bytes.
///
/// # Safety
/// If `Self` is pointer type [`from_bytes_mut`](FrameworkFromBytesMut::from_bytes_mut) must return the same pointer that
/// was passed in. Metadata may be different.
pub unsafe trait FrameworkFromBytesMut<'a>:
    Sized + FrameworkSerialize + PointerBreakup
{
    /// Deserializes this type from a set of bytes mutably.
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self>;
}
unsafe impl<'a, T> FrameworkFromBytesMut<'a> for &'a mut T
where
    T: Align1 + Pod,
{
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        _resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self> {
        Ok(from_bytes_mut(bytes.try_advance(size_of::<T>())?))
    }
}
