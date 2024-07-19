pub mod checked;
pub mod init;
pub mod resize;

mod unsized_enum;

#[cfg(test)]
mod enum_stuff;
#[cfg(test)]
mod test;

pub use star_frame_proc::unsized_type;

use crate::align1::Align1;
use crate::prelude::CombinedUnsized;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use anyhow::Result;
use bytemuck::{CheckedBitPattern, NoUninit, Zeroable};
use std::fmt::Debug;
use std::mem::size_of;
use typenum::{Bit, False, True};

// todo: @expand on this for better docs on what exactly this needs. Connect to RefWrapper, FromBytesReturn?
/// Allows for zero copy deserialization of sized and unsized types over a set of bytes with [`AsBytes`].
///
/// The [`unsized_type`] attribute macro allows for the creation of structs with multiple unsized fields
/// under the hood using normal Rust syntax.
///
/// # Example
/// ```
/// # use star_frame::serialize::test_helpers::TestByteSet;
/// use star_frame::prelude::*;
///
/// # fn main() -> Result<()> {
/// #[unsized_type]
/// pub struct UnsizedTest<T0, T1>
/// where T0: UnsizedGenerics, T1: UnsizedGenerics
/// {
///     pub sized1: T0,
///     pub sized2: u8,
///     #[unsized_start]
///     pub unsized1: List<u8>,
///     pub unsized2: List<T1>,
/// }
///
/// // TestByteSet allows us to emulate how DataAccount works with on-chain AccountInfo data
/// let mut bytes = TestByteSet::<UnsizedTest<u8, [u8; 1]>>::new(Zeroed)?;
/// let mut r = &mut bytes.mutable()?;
/// r.sized1 = 1;
/// r.unsized2()?.push([1])?;
/// r.unsized1()?.push(2)?;
///
/// assert_eq!(r.sized1, 1);
/// assert_eq!(r.unsized1()?.first().unwrap(), &2);
/// assert_eq!(&**(r.unsized2()?), &[[1]]);
///
/// # Ok(())
/// # }
/// ```
///
/// # Safety
/// [`UnsizedType::from_bytes`] must return correct values.
pub unsafe trait UnsizedType: 'static {
    type RefMeta: 'static + Copy;
    type RefData;
    type Owned;

    type IsUnsized: Bit + LengthAccess<Self>;

    /// # Safety
    /// TODO: Think through requirements here
    unsafe fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>>;

    /// # Safety
    /// `meta` must have come from a call to [`UnsizedType::from_bytes`] on the same `super_ref`.
    #[allow(unused_variables)]
    unsafe fn from_bytes_and_meta<S: AsBytes>(
        super_ref: S,
        meta: Self::RefMeta,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        unsafe { Self::from_bytes(super_ref) }
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned>;
}
/// This is a helper trait for the [`UnsizedType`] trait. The required supertraits meet the [`checked`] blanket implementation for [`UnsizedType`].
pub trait UnsizedGenerics: CheckedBitPattern + Align1 + NoUninit + Zeroable {}
impl<T> UnsizedGenerics for T where T: CheckedBitPattern + Align1 + NoUninit + Zeroable {}

/// This is a helper trait for the [`LengthAccess`] trait. It should probably not be implemented manually.
///
/// # Safety
/// [`Self::LENGTH`] must be the correct size of underlying type, which must be Sized
pub unsafe trait StaticLength: UnsizedType<IsUnsized = False> {
    const LENGTH: usize;
}
unsafe impl<T> StaticLength for T
where
    T: CheckedBitPattern + UnsizedType<IsUnsized = False>,
{
    const LENGTH: usize = size_of::<T>();
}
unsafe impl<T, U> StaticLength for CombinedUnsized<T, U>
where
    T: ?Sized + StaticLength,
    U: ?Sized + StaticLength,
{
    const LENGTH: usize = T::LENGTH + U::LENGTH;
}

/// # Safety
/// [`LengthAccess::len`] must return the same value that was passed to [`LengthAccess::from_len`].
pub unsafe trait LengthAccess<T: ?Sized> {
    type LengthData: 'static + Copy + Debug;

    fn from_len(len: usize) -> Self::LengthData;
    fn len(data: Self::LengthData) -> usize;
}
unsafe impl<T: ?Sized> LengthAccess<T> for True {
    type LengthData = usize;

    fn from_len(len: usize) -> Self::LengthData {
        len
    }
    fn len(data: Self::LengthData) -> usize {
        data
    }
}
unsafe impl<T> LengthAccess<T> for False
where
    T: StaticLength,
{
    type LengthData = ();

    fn from_len(_len: usize) -> Self::LengthData {}
    fn len(_data: Self::LengthData) -> usize {
        T::LENGTH
    }
}

#[derive(Debug, Copy, Clone)]
pub struct FromBytesReturn<S, R, M> {
    pub bytes_used: usize,
    pub meta: M,
    pub ref_wrapper: RefWrapper<S, R>,
}
impl<S, R, M> FromBytesReturn<S, R, M> {
    pub unsafe fn map_ref<R2>(self, f: impl FnOnce(&mut S, R) -> R2) -> FromBytesReturn<S, R2, M> {
        FromBytesReturn {
            ref_wrapper: unsafe { self.ref_wrapper.wrap_r(f) },
            meta: self.meta,
            bytes_used: self.bytes_used,
        }
    }

    pub unsafe fn map_meta<M2>(self, f: impl FnOnce(M) -> M2) -> FromBytesReturn<S, R, M2> {
        FromBytesReturn {
            ref_wrapper: self.ref_wrapper,
            meta: f(self.meta),
            bytes_used: self.bytes_used,
        }
    }
}
