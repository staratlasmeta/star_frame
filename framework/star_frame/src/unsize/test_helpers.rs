#![allow(unused)]
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::{
    ExclusiveWrapper, ExclusiveWrapperBorrowed, SharedWrapper, UnsizedTypeDataAccess,
};
use crate::unsize::UnsizedType;
use crate::Result;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::slice_from_raw_parts_mut;
use std::slice::from_raw_parts_mut;

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

        unsafe {
            *data = from_raw_parts_mut(data.as_mut_ptr(), new_len);
        }
        Ok(())
    }

    fn data(&self) -> &RefCell<&'info mut [u8]> {
        &self.data
    }
}

/// A way to test [`UnsizedType`] types. Uses a [`TestAccountInfo`] internally.
#[derive(Debug)]
pub struct TestByteSet<'a, T: ?Sized + UnsizedType> {
    test_account: &'a TestAccountInfo<'a>,
    phantom_t: PhantomData<T>,
    // exclusive_wrapper:
    //     MaybeUninit<ExclusiveWrapper<'a, 'a, <T as UnsizedType>::Mut<'a>, T, TestAccountInfo<'a>>>,
    // is_initialized: bool,
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
            // exclusive_wrapper: MaybeUninit::uninit(),
            // is_initialized: false,
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
        // let exclusive_wrapper = unsafe { ExclusiveWrapper::new(self.test_account) }?;
        // self.exclusive_wrapper.write(exclusive_wrapper);
        unsafe { ExclusiveWrapper::new(self.test_account) }
    }

    // pub fn data_mut(
    //     &mut self,
    // ) -> Result<
    //     ExclusiveWrapperBorrowed<'a, 'a, 'a, <T as UnsizedType>::Mut<'a>, T, TestAccountInfo<'a>>,
    // > {
    //     if self.is_initialized {
    //         unsafe { self.exclusive_wrapper.assume_init_drop() };
    //     }
    //     let exclusive_wrapper = unsafe { ExclusiveWrapper::new(self.test_account) }?;
    //     self.exclusive_wrapper.write(exclusive_wrapper);
    //     self.is_initialized = true;
    //     unsafe { Ok((*self.exclusive_wrapper.as_mut_ptr()).exclusive()) }
    // }
}
