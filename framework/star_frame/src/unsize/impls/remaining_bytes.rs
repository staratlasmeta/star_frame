use crate::align1::Align1;
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::{ExclusiveWrapper, UnsizedTypeDataAccess};
use crate::unsize::UnsizedType;
use crate::unsize::{AsShared, ResizeOperation};
use crate::Result;
use advance::Advance;
use derive_more::{Deref, DerefMut};
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem::size_of;
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

    unsafe fn resize_notification(data: &mut &mut [u8], _operation: ResizeOperation) -> Result<()> {
        data.advance(data.len());
        Ok(())
    }
}

pub trait RemainingBytesExclusive {
    fn set_len(&mut self, len: usize) -> Result<()>;
}

impl<'a, 'info, O: ?Sized, A> RemainingBytesExclusive
    for ExclusiveWrapper<'a, 'info, RemainingBytesMut<'a>, O, A>
where
    O: UnsizedType,
    A: UnsizedTypeDataAccess<'info>,
{
    fn set_len(&mut self, len: usize) -> Result<()> {
        match self.len().cmp(&len) {
            Ordering::Less => {
                let bytes_to_add = len - self.len();
                let bytes: &mut [u8] = self;
                unsafe {
                    let end_ptr = bytes.as_mut_ptr().add(self.len()).cast();
                    ExclusiveWrapper::add_bytes(self, end_ptr, bytes_to_add, |_r| Ok(()))?;
                }
            }
            Ordering::Equal => return Ok(()),
            Ordering::Greater => unsafe {
                let start_ptr = self.as_ptr().add(len).cast();
                let end_ptr = self.as_ptr().add(self.len()).cast();
                ExclusiveWrapper::remove_bytes(self, start_ptr..end_ptr, |_r| Ok(()))?;
            },
        };
        unsafe {
            ExclusiveWrapper::set_inner(self, |bytes| {
                bytes.0 = &mut *ptr::from_raw_parts_mut(bytes.0.cast::<()>(), len);
            });
        }
        Ok(())
    }
}

impl UnsizedInit<DefaultInit> for RemainingBytes {
    const INIT_BYTES: usize = 0;

    unsafe fn init(_bytes: &mut &mut [u8], _arg: DefaultInit) -> Result<()> {
        Ok(())
    }
}

impl<const N: usize> UnsizedInit<&[u8; N]> for RemainingBytes {
    const INIT_BYTES: usize = N;

    unsafe fn init(bytes: &mut &mut [u8], array: &[u8; N]) -> Result<()> {
        bytes.advance(N).copy_from_slice(array);
        Ok(())
    }
}

impl<const N: usize> UnsizedInit<[u8; N]> for RemainingBytes {
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
        // assert_eq!(***bytes, byte_array);
        bytes.set_len(3)?;
        println!("{:?}", &**bytes);
        Ok(())
    }
}
