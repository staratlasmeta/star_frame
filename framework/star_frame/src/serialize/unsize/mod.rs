pub mod checked;
pub mod init;
pub mod resize;

mod enum_stuff;
mod proc_test;
pub mod test;
mod unsized_enum;

pub use star_frame_proc::unsized_type;

use crate::prelude::CombinedUnsized;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use anyhow::Result;
use bytemuck::CheckedBitPattern;
use std::fmt::Debug;
use std::mem::size_of;
use typenum::{Bit, False, True};

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
