#![allow(unused)]
use pinocchio::account_info::MAX_PERMITTED_DATA_INCREASE;

use crate::{
    unsize::{
        init::{DefaultInit, UnsizedInit},
        wrapper::{DataMutDrop, SharedWrapper, UnsizedDataMut, UnsizedTypeDataAccess},
        UnsizedType,
    },
    Result,
};
use alloc::{boxed::Box, vec, vec::Vec};
use core::{
    cell::{Cell, Ref, RefCell, RefMut},
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Range,
    ptr::slice_from_raw_parts_mut,
    slice::from_raw_parts_mut,
};

use super::{
    wrapper::{ExclusiveWrapper, ExclusiveWrapperTop, ExclusiveWrapperTopMeta},
    FromOwned,
};

#[derive(Debug)]
pub struct TestUnderlyingData {
    len: Cell<usize>,
    original_len: usize,
    data: RefCell<Vec<u8>>,
}
impl TestUnderlyingData {
    #[must_use]
    pub fn new(data_len: usize) -> Self {
        let vec = vec![0u8; data_len + MAX_PERMITTED_DATA_INCREASE];
        Self {
            len: Cell::new(data_len),
            original_len: data_len,
            data: RefCell::new(vec),
        }
    }
}

impl<T: ?Sized> DataMutDrop for RefMut<'_, T> {}

/// # Safety
/// We are properly checking the bounds in `unsized_data_realloc`.
unsafe impl UnsizedTypeDataAccess for TestUnderlyingData {
    unsafe fn unsized_data_realloc(
        this: &Self,
        data: &mut *mut [u8],
        new_len: usize,
    ) -> Result<()> {
        assert!(
            new_len <= this.original_len + MAX_PERMITTED_DATA_INCREASE,
            "data too large"
        );

        let current_len = this.len.get();
        let difference = i64::try_from(new_len)? - i64::try_from(current_len)?;

        this.len.set(new_len);

        if difference > 0 {
            // SAFETY:
            // dst is valid and properly aligned. We have exclusive access to it right now
            unsafe {
                core::ptr::write_bytes(
                    data.cast::<u8>().wrapping_byte_add(current_len),
                    0,
                    difference.try_into()?,
                );
            }
        }

        unsafe {
            *data = ptr_meta::from_raw_parts_mut(data.cast::<()>(), new_len);
        }
        Ok(())
    }

    fn data_ref(this: &Self) -> Result<impl core::ops::Deref<Target = [u8]>> {
        Ok(Ref::map(this.data.borrow(), |data| &data[..this.len.get()]))
    }

    fn data_mut(this: &Self) -> Result<UnsizedDataMut<'_>> {
        let mut data = this.data.borrow_mut();
        let ptr: *mut [u8] =
            ptr_meta::from_raw_parts_mut(data.as_mut_ptr().cast::<()>(), this.len.get());
        let range = ptr.addr()..(ptr.addr() + this.original_len + MAX_PERMITTED_DATA_INCREASE);
        Ok((ptr, range, Box::new(data)))
    }
}

/// A way to work with [`UnsizedType`] types off-chain. Uses a [`TestUnderlyingData`] internally.
#[derive(Debug)]
pub struct TestByteSet<T: ?Sized + UnsizedType> {
    test_data: TestUnderlyingData,
    phantom_t: PhantomData<T>,
}

impl<T> TestByteSet<T>
where
    T: UnsizedType + ?Sized,
{
    fn initialize(size: usize, init: impl FnOnce(&mut &mut [u8]) -> Result<()>) -> Result<Self> {
        // Temporarily leak the data. It will be cleaned up in the Drop implementation.
        let test_account = TestUnderlyingData::new(size);
        {
            let (mut data, .., _guard) = UnsizedTypeDataAccess::data_mut(&test_account)?;
            // SAFETY:
            // We can mutate the data while the guard is alive. No one else has access to the data
            let mut data = unsafe { &mut *data };
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

    pub fn data(&self) -> Result<SharedWrapper<'_, T::Ptr>> {
        SharedWrapper::new::<T>(&self.test_data)
    }

    pub fn data_mut(&self) -> Result<ExclusiveWrapperTop<'_, T, TestUnderlyingData>> {
        ExclusiveWrapper::new(&self.test_data)
    }

    pub fn owned(&self) -> Result<T::Owned> {
        T::owned(&self.test_data.data.try_borrow()?[..])
    }

    pub fn underlying_data(&self) -> Result<Vec<u8>> {
        Ok(self.test_data.data.try_borrow()?.to_vec())
    }
}

#[doc(hidden)]
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

pub trait ModifyOwned: UnsizedType + FromOwned<Owned: Clone> {
    fn modify_owned(
        owned: &mut Self::Owned,
        modify: impl for<'a, 'top> FnOnce(
            &'a mut ExclusiveWrapperTop<'top, Self, TestUnderlyingData>,
        ) -> Result<()>,
    ) -> Result<()> {
        let self_byte_set = TestByteSet::<Self>::new(owned.clone())?;
        let mut bytes_mut = self_byte_set.data_mut()?;
        modify(&mut bytes_mut)?;
        drop(bytes_mut);
        *owned = self_byte_set.owned()?;
        Ok(())
    }
}

impl<T> ModifyOwned for T where T: UnsizedType + FromOwned<Owned: Clone> + ?Sized {}
