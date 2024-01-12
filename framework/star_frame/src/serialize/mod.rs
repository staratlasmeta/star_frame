pub mod borsh;

use crate::align1::Align1;
use advance::Advance;
use bytemuck::{from_bytes, from_bytes_mut, Pod};
use std::mem::size_of;

/// Writes this type to a set of bytes and reads this type from bytes.
///
/// # Safety
/// If `Self` is pointer type [`from_bytes`](FrameworkSerialize::from_bytes) and
/// [`from_bytes_mut`](FrameworkSerialize::from_bytes_mut) must return the same pointer that was passed in. Metadata may
/// be different.
pub unsafe trait FrameworkSerialize<'a>: Sized {
    /// Writes this type to a set of bytes.
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()>;
    /// Deserializes this type from a set of bytes.
    fn from_bytes(bytes: &mut &'a [u8]) -> crate::Result<Self>;
}

unsafe impl<'a, T> FrameworkSerialize<'a> for &'a T
where
    T: Align1 + Pod,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        output
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(*self));
        Ok(())
    }

    fn from_bytes(bytes: &mut &'a [u8]) -> crate::Result<Self> {
        Ok(from_bytes(bytes.try_advance(size_of::<T>())?))
    }
}

/// Allows this type to be referenced mutably from a set of bytes.
///
/// # Safety
/// [`from_bytes`](FrameworkSerialize::from_bytes) and [`from_bytes_mut`](FrameworkSerialize::from_bytes_mut) must
/// return the same pointer that was passed in. Metadata may be different.
pub unsafe trait FrameworkSerializeMut<'a>
where
    Self: 'a,
    &'a Self: FrameworkSerialize<'a>,
{
    /// Deserializes this type from a set of bytes mutably.
    fn from_bytes_mut(bytes: &mut &'a mut [u8]) -> crate::Result<&'a mut Self>;
}

unsafe impl<'a, T> FrameworkSerializeMut<'a> for T
where
    T: Align1 + Pod,
{
    fn from_bytes_mut(bytes: &mut &'a mut [u8]) -> crate::Result<&'a mut Self> {
        Ok(from_bytes_mut(bytes.try_advance(size_of::<T>())?))
    }
}
