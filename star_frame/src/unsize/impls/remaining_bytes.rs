use crate::align1::Align1;
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::ResizeExclusive;
use crate::unsize::{AsShared, FromOwned, UnsizedType};
use crate::Result;
use advancer::Advance;
use anyhow::bail;
use anyhow::Context;
use derive_more::{Deref, DerefMut};
use ptr_meta::Pointee;
use star_frame_proc::unsized_impl;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Deref, DerefMut, Align1, Pointee)]
#[repr(transparent)]
pub struct RemainingBytes([u8]);

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use crate::idl::ty::impl_type_to_idl_for_primitive;
    impl_type_to_idl_for_primitive!(super::RemainingBytes: RemainingBytes);
}

#[derive(Copy, Clone, Debug)]
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

impl AsShared for RemainingBytesMut<'_> {
    type Ref<'a>
        = RemainingBytesRef<'a>
    where
        Self: 'a;
    fn as_shared(&self) -> Self::Ref<'_> {
        RemainingBytes::mut_as_ref(self)
    }
}

unsafe impl UnsizedType for RemainingBytes {
    type Ref<'a> = RemainingBytesRef<'a>;
    type Mut<'a> = RemainingBytesMut<'a>;
    type Owned = Vec<u8>;
    const ZST_STATUS: bool = false;

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
            unsafe { &*ptr_meta::from_raw_parts(ptr.cast::<()>(), remaining_bytes.len()) },
            PhantomData,
        ))
    }

    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>> {
        let remaining_bytes = data.try_advance(data.len()).with_context(|| {
            format!(
                "Failed to read remaining mutable {} bytes for RemainingBytes",
                data.len()
            )
        })?;
        let ptr = remaining_bytes.as_mut_ptr();
        Ok(RemainingBytesMut(
            ptr_meta::from_raw_parts_mut(ptr.cast::<()>(), remaining_bytes.len()),
            PhantomData,
        ))
    }

    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned> {
        Ok(r.to_vec())
    }

    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()> {
        let self_ptr = self_mut.0;
        match source_ptr.cmp(&self_ptr.cast_const().cast()) {
            Ordering::Less => self_mut.0 = unsafe { self_ptr.byte_offset(change) },
            Ordering::Equal => {}
            Ordering::Greater => {
                bail!("Resize occurred after RemainingBytes, which shouldn't be possible")
            }
        }
        Ok(())
    }
}

unsafe impl FromOwned for RemainingBytes {
    fn byte_size(owned: &Self::Owned) -> usize {
        owned.len()
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        bytes.try_advance(owned.len())?.copy_from_slice(&owned);
        Ok(owned.len())
    }
}

#[unsized_impl(inherent)]
impl RemainingBytes {
    #[exclusive]
    pub fn set_len(&mut self, len: usize) -> Result<()> {
        let self_len = self.len();
        match self.len().cmp(&len) {
            Ordering::Less => {
                let bytes_to_add = len - self_len;
                let (source_ptr, end_ptr) = {
                    let source_ptr = self.0.cast_const().cast::<()>();
                    let end_ptr = unsafe { source_ptr.byte_add(self_len) };
                    (source_ptr, end_ptr)
                };
                unsafe {
                    ResizeExclusive::add_bytes(self, source_ptr, end_ptr, bytes_to_add)?;
                }
            }
            Ordering::Equal => return Ok(()),
            Ordering::Greater => {
                let source_ptr = self.0.cast_const().cast::<()>();
                let start_ptr = unsafe { source_ptr.byte_add(len) };
                let end_ptr = unsafe { source_ptr.byte_add(self_len) };
                unsafe {
                    ResizeExclusive::remove_bytes(self, source_ptr, start_ptr..end_ptr)?;
                }
            }
        };
        self.0 = ptr_meta::from_raw_parts_mut(self.0.cast::<()>(), len);
        Ok(())
    }
}

unsafe impl UnsizedInit<DefaultInit> for RemainingBytes {
    const INIT_BYTES: usize = 0;

    unsafe fn init(_bytes: &mut &mut [u8], _arg: DefaultInit) -> Result<()> {
        Ok(())
    }
}

unsafe impl<const N: usize> UnsizedInit<&[u8; N]> for RemainingBytes {
    const INIT_BYTES: usize = N;

    unsafe fn init(bytes: &mut &mut [u8], array: &[u8; N]) -> Result<()> {
        bytes
            .try_advance(N)
            .with_context(|| {
                format!("Failed to advance {N} bytes for RemainingBytes reference initialization")
            })?
            .copy_from_slice(array);
        Ok(())
    }
}

unsafe impl<const N: usize> UnsizedInit<[u8; N]> for RemainingBytes {
    const INIT_BYTES: usize = <Self as UnsizedInit<&[u8; N]>>::INIT_BYTES;

    unsafe fn init(bytes: &mut &mut [u8], array: [u8; N]) -> Result<()> {
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
