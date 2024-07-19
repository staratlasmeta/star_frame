pub mod checked;
pub mod init;
pub mod resize;

mod enum_stuff;
mod proc_test;
pub mod test;
mod test2;
pub mod test_sized_struct;
mod unsized_enum;

pub use star_frame_proc::unsized_type;

use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use anyhow::Result;
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
unsafe impl<T> LengthAccess<T> for False {
    type LengthData = ();

    fn from_len(_len: usize) -> Self::LengthData {}
    fn len(_data: Self::LengthData) -> usize {
        size_of::<T>()
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
