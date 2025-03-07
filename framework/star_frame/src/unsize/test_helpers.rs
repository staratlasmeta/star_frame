#![allow(unused)]
use crate::unsize::init::{DefaultInit, UnsizedInit};
// use crate::unsize::wrapper::exclusive_passer::ExclusiveWrapperPasser;
use crate::unsize::wrapper::{ExclusiveWrapper, SharedWrapper, UnsizedTypeDataAccess};
use crate::unsize::UnsizedType;
use crate::Result;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ptr::slice_from_raw_parts_mut;

#[derive(Debug)]
pub struct TestAccountInfo<'info> {
    original_data_len: usize,
    data: RefCell<&'info mut [u8]>,
}
impl<'info> TestAccountInfo<'info> {
    pub fn new(backing: &'info mut Vec<u8>, data_len: usize) -> Self {
        backing.resize(data_len + MAX_PERMITTED_DATA_INCREASE, 0);
        Self {
            original_data_len: data_len,
            data: RefCell::new(&mut backing[..data_len]),
        }
    }
}

impl<'info> UnsizedTypeDataAccess<'info> for TestAccountInfo<'info> {
    unsafe fn realloc(&self, new_len: usize, data: &mut &'info mut [u8]) -> Result<()> {
        assert!(
            new_len <= self.original_data_len + MAX_PERMITTED_DATA_INCREASE,
            "data too large"
        );

        let ptr = data.as_mut_ptr();
        let slice = slice_from_raw_parts_mut(ptr, new_len);
        unsafe {
            *data = &mut *slice;
        }
        Ok(())
    }

    fn data(&self) -> &RefCell<&'info mut [u8]> {
        &self.data
    }
}

/// A way to test [`UnsizedType`] types. Uses a [`TestAccountInfo`] internally.
#[derive(Debug)]
pub struct TestByteSet<'a, T: ?Sized> {
    test_account: &'a TestAccountInfo<'a>,
    phantom_t: PhantomData<T>,
}

impl<'a, T> TestByteSet<'a, T>
where
    T: UnsizedType + ?Sized,
{
    /// Creates a new [`TestByteSet`] by initializing the type with an arg from [`UnsizedInit`].
    pub fn new<A>(arg: A) -> Result<Self>
    where
        T: UnsizedInit<A>,
    {
        let data: &mut Vec<u8> = Box::leak(Box::default());
        let test_account = Box::leak(Box::new(TestAccountInfo::new(data, T::INIT_BYTES)));
        {
            let mut data = &mut test_account.data().borrow_mut()[..];
            unsafe {
                T::init(&mut data, arg)?;
            }
        }
        Ok(Self {
            test_account,
            phantom_t: PhantomData,
        })
    }

    /// Creates a new [`TestByteSet`] by initializing the type with an arg from [`UnsizedInit`].
    pub fn new_default() -> Result<Self>
    where
        T: UnsizedInit<DefaultInit>,
    {
        Self::new(DefaultInit)
    }

    pub fn data_ref(&self) -> Result<SharedWrapper<'_, '_, T::Ref<'a>>> {
        unsafe { SharedWrapper::<T>::new(self.test_account) }
    }

    pub fn data_mut(&self) -> Result<ExclusiveWrapper<'a, '_, T::Mut<'_>, T, TestAccountInfo<'_>>> {
        unsafe { ExclusiveWrapper::new(self.test_account) }
    }
}
