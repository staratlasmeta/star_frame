#![allow(unused)]
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::{SharedWrapper, UnsizedTypeDataAccess};
use crate::unsize::UnsizedType;
use crate::Result;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::slice_from_raw_parts_mut;
use std::slice::from_raw_parts_mut;

use super::wrapper::{ExclusiveWrapper, ExclusiveWrapperTop, ExclusiveWrapperTopMeta};
use super::FromOwned;

#[derive(Debug)]
pub struct TestUnderlyingData<'info> {
    original_data_len: usize,
    data: RefCell<&'info mut [u8]>,
}
impl<'info> TestUnderlyingData<'info> {
    pub fn new(backing: &'info mut Vec<u8>, data_len: usize) -> Self {
        backing.resize(data_len + MAX_PERMITTED_DATA_INCREASE, 0);
        Self {
            original_data_len: data_len,
            data: RefCell::new(&mut backing[..data_len]),
        }
    }
}

impl<'info> UnsizedTypeDataAccess<'info> for TestUnderlyingData<'info> {
    unsafe fn realloc(this: &Self, new_len: usize, data: &mut &'info mut [u8]) -> Result<()> {
        assert!(
            new_len <= this.original_data_len + MAX_PERMITTED_DATA_INCREASE,
            "data too large"
        );

        unsafe {
            *data = from_raw_parts_mut(data.as_mut_ptr(), new_len);
        }
        Ok(())
    }
    fn data_ref(this: &Self) -> Result<Ref<&'info mut [u8]>> {
        this.data.try_borrow().map_err(Into::into)
    }

    fn data_mut(this: &Self) -> Result<RefMut<&'info mut [u8]>> {
        this.data.try_borrow_mut().map_err(Into::into)
    }
}

/// A way to work with [`UnsizedType`] types off-chain. Uses a [`TestUnderlyingData`] internally.
#[derive(Debug)]
pub struct TestByteSet<T: ?Sized + UnsizedType> {
    test_data: *mut TestUnderlyingData<'static>,
    data_ptr: *mut Vec<u8>,
    phantom_t: PhantomData<T>,
}

impl<T: ?Sized + UnsizedType> Drop for TestByteSet<T> {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.test_data) });
        drop(unsafe { Box::from_raw(self.data_ptr) });
    }
}

impl<T> TestByteSet<T>
where
    T: UnsizedType + ?Sized,
{
    fn initialize(size: usize, init: impl FnOnce(&mut &mut [u8]) -> Result<()>) -> Result<Self> {
        // Temporarily leak the data. It will be cleaned up in the Drop implementation.
        let data: &mut Vec<u8> = Box::leak(Box::default());
        let data_ptr: *mut Vec<u8> = data;
        let test_account = Box::leak(Box::new(TestUnderlyingData::new(data, size)));
        {
            let mut data = &mut UnsizedTypeDataAccess::data_mut(test_account)?[..];
            unsafe {
                init(&mut data)?;
            }
        }
        Ok(Self {
            test_data: test_account,
            phantom_t: PhantomData,
            data_ptr,
        })
    }

    /// # Safety
    ///
    /// We need to ensure this value is never exposed to the user, since it could outlive Self and lead
    /// to use after free errors.
    unsafe fn test_data_ref(&self) -> &'static TestUnderlyingData<'static> {
        unsafe { &*self.test_data }
    }

    /// Creates a new [`TestByteSet`] from the owned value
    pub fn new(owned: T::Owned) -> Result<Self>
    where
        T: FromOwned,
    {
        Self::initialize(T::byte_size(&owned), |data| {
            T::from_owned(owned, data).map(|_| ())
        })
    }

    /// Creates a new [`TestByteSet`] by initializing the type with an arg from [`UnsizedInit`].
    pub fn new_from_init<A>(arg: A) -> Result<Self>
    where
        T: UnsizedInit<A>,
    {
        Self::initialize(T::INIT_BYTES, |data| unsafe { T::init(data, arg) })
    }

    /// Creates a new [`TestByteSet`] using the default initializer.
    pub fn new_default() -> Result<Self>
    where
        T: UnsizedInit<DefaultInit>,
    {
        Self::new_from_init(DefaultInit)
    }

    pub fn data(&self) -> Result<SharedWrapper<'_, T::Ref<'_>>> {
        unsafe { SharedWrapper::new::<T>(self.test_data_ref()) }
    }

    pub fn data_mut(&self) -> Result<ExclusiveWrapperTop<'_, T, TestUnderlyingData<'static>>> {
        // SAFETY: test_data_ref is not being returned to the user; ExclusiveWrapper doesn't expose it.
        let test_data_ref = unsafe { self.test_data_ref() };
        ExclusiveWrapper::new(test_data_ref)
    }

    pub fn owned(&self) -> Result<T::Owned> {
        // SAFETY: test_data is not being returned to the user
        let test_data = unsafe { self.test_data_ref() };
        T::owned(&test_data.data.try_borrow()?)
    }

    pub fn underlying_data(&self) -> Result<Vec<u8>> {
        // SAFETY: test_data is not being returned to the user
        let test_data = unsafe { self.test_data_ref() };
        Ok(test_data.data.try_borrow()?.to_vec())
    }
}

#[macro_export]
macro_rules! assert_with_shared {
    ($the_mut:ident => $expr:expr $(, $($arg:tt)*)?) => {
        assert!($expr, $($($arg)*)*);
        {
            let $the_mut = $the_mut.as_shared();
            assert!($expr, $($($arg)*)*);
        }
    };
}

#[macro_export]
macro_rules! assert_eq_with_shared {
    ($the_mut:ident => $left:expr, $right:expr $(, $($arg:tt)*)?) => {
        assert_eq!($left, $right, $($($arg)*)*);
        {
            let $the_mut = $the_mut.as_shared();
            assert_eq!($left, $right, $($($arg)*)*)
        }
    };
}

pub trait NewByteSet: UnsizedType {
    fn new_byte_set(owned: Self::Owned) -> Result<TestByteSet<Self>>
    where
        Self: FromOwned,
    {
        TestByteSet::<Self>::new(owned)
    }

    fn new_default_byte_set() -> Result<TestByteSet<Self>>
    where
        Self: UnsizedInit<DefaultInit>,
    {
        TestByteSet::<Self>::new_default()
    }
}

impl<T> NewByteSet for T where T: UnsizedType + ?Sized {}

pub trait ModifyOwned: Clone {
    fn modify_owned<U>(
        &mut self,
        modify: impl for<'a, 'top> FnOnce(
            &'a mut ExclusiveWrapper<
                'top,
                'top,
                U::Mut<'top>,
                U,
                ExclusiveWrapperTopMeta<'top, TestUnderlyingData<'static>>,
            >,
        ) -> Result<()>,
    ) -> Result<()>
    where
        U: UnsizedType<Owned = Self> + FromOwned + ?Sized,
    {
        let self_byte_set = TestByteSet::<U>::new(self.clone())?;
        let mut bytes_mut = self_byte_set.data_mut()?;
        modify(&mut bytes_mut)?;
        drop(bytes_mut);
        *self = self_byte_set.owned()?;
        Ok(())
    }
}

impl<T> ModifyOwned for T where T: Clone {}
