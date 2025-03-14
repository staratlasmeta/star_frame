use crate::align1::Align1;
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::ExclusiveWrapper;
use crate::unsize::AsShared;
use crate::unsize::UnsizedType;
use crate::Result;
use advancer::Advance;
use anyhow::bail;
use derive_more::{Deref, DerefMut};
use star_frame_proc::unsized_impl;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;

#[derive(Debug, Deref, DerefMut, Align1)]
pub struct RemainingBytes([u8]);

#[derive(Copy, Clone, Debug)]
pub struct RemainingBytesRef<'a>(*const RemainingBytes, PhantomData<&'a ()>);

impl<'a> Deref for RemainingBytesRef<'a> {
    type Target = RemainingBytes;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
#[derive(Debug)]
pub struct RemainingBytesMut<'a>(*mut RemainingBytes, PhantomData<&'a ()>);

impl<'a> Deref for RemainingBytesMut<'a> {
    type Target = RemainingBytes;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<'a> DerefMut for RemainingBytesMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}
impl<'a> AsShared<'a> for RemainingBytesMut<'_> {
    type Shared<'b> = RemainingBytesRef<'b> where Self: 'a + 'b;

    fn as_shared(&'a self) -> RemainingBytesRef<'a> {
        RemainingBytesRef(self.0.cast_const(), PhantomData)
    }
}

unsafe impl UnsizedType for RemainingBytes {
    type Ref<'a> = RemainingBytesRef<'a>;
    type Mut<'a> = RemainingBytesMut<'a>;
    type Owned = Vec<u8>;
    const ZST_STATUS: bool = false;

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        let remaining_bytes = data.advance(data.len());
        let ptr = remaining_bytes.as_ptr();
        Ok(RemainingBytesRef(
            unsafe { &*ptr::from_raw_parts(ptr.cast(), remaining_bytes.len()) },
            PhantomData,
        ))
    }

    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>> {
        let remaining_bytes = data.advance(data.len());
        let ptr = remaining_bytes.as_mut_ptr();
        Ok(RemainingBytesMut(
            unsafe { &mut *ptr::from_raw_parts_mut(ptr.cast(), remaining_bytes.len()) },
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
                    let end_ptr = unsafe { source_ptr.byte_add(self_len) };
                    (source_ptr, end_ptr)
                };
                unsafe {
                    ExclusiveWrapper::add_bytes(self, source_ptr, end_ptr, bytes_to_add, |_r| {
                        Ok(())
                    })?;
                }
            }
            Ordering::Equal => return Ok(()),
            Ordering::Greater => {
                let source_ptr = self.0.cast_const().cast::<()>();
                let start_ptr = unsafe { source_ptr.byte_add(len) };
                let end_ptr = unsafe { source_ptr.byte_add(self_len) };
                unsafe {
                    ExclusiveWrapper::remove_bytes(self, source_ptr, start_ptr..end_ptr, |_r| {
                        Ok(())
                    })?;
                }
            }
        };
        unsafe {
            ExclusiveWrapper::set_inner(self, |bytes| {
                bytes.0 = &mut *ptr::from_raw_parts_mut(bytes.0.cast::<()>(), len);
                Ok(())
            })?;
        }
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
        bytes.advance(N).copy_from_slice(array);
        Ok(())
    }
}

unsafe impl<const N: usize> UnsizedInit<[u8; N]> for RemainingBytes {
    const INIT_BYTES: usize = <Self as UnsizedInit<&[u8; N]>>::INIT_BYTES;

    unsafe fn init(bytes: &mut &mut [u8], array: [u8; N]) -> Result<()> {
        unsafe { <Self as UnsizedInit<&[u8; N]>>::init(bytes, &array) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unsize::test_helpers::TestByteSet;

    #[test]
    fn test_remaining_bytes() -> Result<()> {
        let byte_array = [1, 2, 3, 4, 5];
        let test_bytes = TestByteSet::<RemainingBytes>::new(&byte_array)?;
        let mut bytes = test_bytes.data_mut()?;
        bytes.exclusive().set_len(3)?;
        println!("{:?}", &**bytes);
        Ok(())
    }
}
