#![allow(unused)]
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::{ExclusiveWrapper, MutWrapper, SharedWrapper, UnsizedTypeDataAccess};
use crate::unsize::UnsizedType;
use crate::Result;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::slice_from_raw_parts_mut;
use std::slice::from_raw_parts_mut;

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

/// A way to test [`UnsizedType`] types. Uses a [`TestUnderlyingData`] internally.
#[derive(Debug)]
pub struct TestByteSet<'a, T: ?Sized + UnsizedType> {
    test_data: &'a TestUnderlyingData<'a>,
    phantom_t: PhantomData<T>,
}

pub trait NewByteSet: UnsizedType {
    fn new_byte_set<'a>(owned: Self::Owned) -> Result<TestByteSet<'a, Self>>
    where
        Self: FromOwned,
    {
        TestByteSet::new(owned)
    }

    fn new_default_byte_set<'a>() -> Result<TestByteSet<'a, Self>>
    where
        Self: UnsizedInit<DefaultInit>,
    {
        TestByteSet::new_default()
    }
}

impl<T> NewByteSet for T where T: UnsizedType {}

impl<'a, T> TestByteSet<'a, T>
where
    T: UnsizedType + ?Sized,
{
    fn initialize(size: usize, init: impl FnOnce(&mut &mut [u8]) -> Result<()>) -> Result<Self> {
        let data: &mut Vec<u8> = Box::leak(Box::default());
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
        })
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

    pub fn data(&self) -> Result<SharedWrapper<'a, '_, T::Ref<'a>>> {
        unsafe { SharedWrapper::<T>::new(self.test_data) }
    }

    pub fn data_mut(&self) -> Result<MutWrapper<'a, '_, T, TestUnderlyingData<'_>>> {
        unsafe { MutWrapper::new(self.test_data) }
    }

    pub fn owned(&self) -> Result<T::Owned> {
        T::owned(&self.test_data.data.try_borrow()?)
    }

    pub fn underlying_data(&self) -> Result<Vec<u8>> {
        Ok(self.test_data.data.try_borrow()?.to_vec())
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
