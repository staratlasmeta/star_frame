//! Raw byte buffer type that consumes all remaining data.
//!
//! This module provides [`RemainingBytes`], an unsized type that represents all remaining bytes
//! in a data buffer. It's useful for handling unstructured data, variable-length content, or
//! implementing custom parsing logic while working within the unsized type system.
use crate::{
    align1::Align1,
    unsize::{
        init::{DefaultInit, UnsizedInit},
        wrapper::ExclusiveRecurse,
        AsShared, FromOwned, RawSliceAdvance, UnsizedType, UnsizedTypeMut,
    },
    Result,
};
use advancer::Advance;
use derive_more::{Deref, DerefMut};
use eyre::{bail, Context};
use ptr_meta::Pointee;
use star_frame_proc::unsized_impl;
use std::{
    cmp::Ordering,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Deref, DerefMut, Align1, Pointee)]
#[repr(transparent)]
pub struct RemainingBytes([u8]);

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::impl_type_to_idl_for_primitive;
    impl_type_to_idl_for_primitive!(super::RemainingBytes: RemainingBytes);
}

#[derive(Debug)]
pub struct RemainingBytesRef<'a>(*const RemainingBytes, PhantomData<&'a ()>);

impl Deref for RemainingBytesRef<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
#[derive(Debug)]
pub struct RemainingBytesMut<'a>(*mut RemainingBytes, PhantomData<&'a ()>);

impl Deref for RemainingBytesMut<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl DerefMut for RemainingBytesMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl AsShared for RemainingBytesRef<'_> {
    type Ref<'a>
        = RemainingBytesRef<'a>
    where
        Self: 'a;
    fn as_shared(&self) -> Self::Ref<'_> {
        RemainingBytesRef(self.0, PhantomData)
    }
}
impl AsShared for RemainingBytesMut<'_> {
    type Ref<'a>
        = RemainingBytesRef<'a>
    where
        Self: 'a;
    fn as_shared(&self) -> Self::Ref<'_> {
        RemainingBytes::mut_as_ref(self)
    }
}

unsafe impl UnsizedTypeMut for RemainingBytesMut<'_> {
    type UnsizedType = RemainingBytes;
}

unsafe impl UnsizedType for RemainingBytes {
    type Ref<'a> = RemainingBytesRef<'a>;
    type Mut<'a> = RemainingBytesMut<'a>;
    type Owned = Vec<u8>;
    const ZST_STATUS: bool = false;

    fn ref_as_ref<'a>(r: &'a Self::Ref<'_>) -> Self::Ref<'a> {
        RemainingBytesRef(r.0, PhantomData)
    }

    fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a> {
        RemainingBytesRef(m.0, PhantomData)
    }

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        let remaining_bytes = data.try_advance(data.len()).with_context(|| {
            format!(
                "Failed to read remaining {} bytes for RemainingBytes",
                data.len()
            )
        })?;
        let ptr = remaining_bytes.as_ptr();
        Ok(RemainingBytesRef(
            &raw const *ptr_meta::from_raw_parts(ptr.cast::<()>(), remaining_bytes.len()),
            PhantomData,
        ))
    }

    unsafe fn get_mut<'a>(data: &mut *mut [u8]) -> Result<Self::Mut<'a>> {
        let remaining_bytes = data.try_advance(data.len()).with_context(|| {
            format!(
                "Failed to read remaining mutable {} bytes for RemainingBytes",
                data.len()
            )
        })?;
        Ok(RemainingBytesMut(remaining_bytes as _, PhantomData))
    }

    #[inline]
    fn data_len(m: &Self::Mut<'_>) -> usize {
        m.len()
    }

    #[inline]
    fn start_ptr(m: &Self::Mut<'_>) -> *mut () {
        m.0.cast::<()>()
    }

    fn owned_from_ref(r: &Self::Ref<'_>) -> Result<Self::Owned> {
        Ok(r.to_vec())
    }

    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()> {
        let self_ptr = self_mut.0;
        match source_ptr.cmp(&self_ptr.cast_const().cast()) {
            Ordering::Less => self_mut.0 = self_ptr.wrapping_byte_offset(change),
            Ordering::Equal => {}
            Ordering::Greater => {
                bail!("Resize occurred after RemainingBytes, which shouldn't be possible")
            }
        }
        Ok(())
    }
}

impl FromOwned for RemainingBytes {
    fn byte_size(owned: &Self::Owned) -> usize {
        owned.len()
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        bytes.try_advance(owned.len())?.copy_from_slice(&owned);
        Ok(owned.len())
    }
}

#[unsized_impl]
impl RemainingBytes {
    #[exclusive]
    pub fn set_len(&mut self, len: usize) -> Result<()> {
        let self_len = self.len();
        match self.len().cmp(&len) {
            Ordering::Less => {
                let bytes_to_add = len - self_len;
                let (source_ptr, end_ptr) = {
                    let source_ptr = self.0.cast_const().cast::<()>();
                    let end_ptr = source_ptr.wrapping_byte_add(self_len);
                    (source_ptr, end_ptr)
                };
                unsafe {
                    ExclusiveRecurse::add_bytes(self, source_ptr, end_ptr, bytes_to_add)?;
                }
            }
            Ordering::Equal => return Ok(()),
            Ordering::Greater => {
                let source_ptr = self.0.cast_const().cast::<()>();
                let start_ptr = source_ptr.wrapping_byte_add(len);
                let end_ptr = source_ptr.wrapping_byte_add(self_len);
                unsafe {
                    ExclusiveRecurse::remove_bytes(self, source_ptr, start_ptr..end_ptr)?;
                }
            }
        }
        self.0 = ptr_meta::from_raw_parts_mut(self.0.cast::<()>(), len);
        Ok(())
    }
}

impl UnsizedInit<DefaultInit> for RemainingBytes {
    const INIT_BYTES: usize = 0;

    fn init(_bytes: &mut &mut [u8], _arg: DefaultInit) -> Result<()> {
        Ok(())
    }
}

impl<const N: usize> UnsizedInit<&[u8; N]> for RemainingBytes {
    const INIT_BYTES: usize = N;

    fn init(bytes: &mut &mut [u8], array: &[u8; N]) -> Result<()> {
        bytes
            .try_advance(N)
            .with_context(|| {
                format!("Failed to advance {N} bytes for RemainingBytes reference initialization")
            })?
            .copy_from_slice(array);
        Ok(())
    }
}

impl<const N: usize> UnsizedInit<[u8; N]> for RemainingBytes {
    const INIT_BYTES: usize = <Self as UnsizedInit<&[u8; N]>>::INIT_BYTES;

    fn init(bytes: &mut &mut [u8], array: [u8; N]) -> Result<()> {
        bytes
            .try_advance(N)
            .with_context(|| {
                format!("Failed to advance {N} bytes for RemainingBytes initialization")
            })?
            .copy_from_slice(&array);
        Ok(())
    }
}

#[cfg(all(test, feature = "test_helpers"))]
mod tests {
    use super::*;
    use crate::unsize::test_helpers::TestByteSet;

    #[test]
    fn test_remaining_bytes() -> Result<()> {
        let byte_array = [1, 2, 3, 4, 5];
        let test_bytes = TestByteSet::<RemainingBytes>::new_from_init(&byte_array)?;
        let mut bytes = test_bytes.data_mut()?;
        bytes.set_len(3)?;
        println!("{:?}", &**bytes);
        Ok(())
    }
}
