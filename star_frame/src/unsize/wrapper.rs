use super::{AsShared, UnsizedType};
use crate::prelude::UnsizedInit;
use crate::Result;
use advancer::Advance;
use anyhow::{ensure, Context};
use core::fmt;
use derive_more::{Debug, Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memmove;
use std::cell::{Ref, RefMut};
use std::cmp::Ordering;
use std::collections::Bound;
use std::fmt::Pointer;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, RangeBounds};
use std::slice::from_raw_parts_mut;

pub trait UnsizedTypeDataAccess<'info> {
    /// # Safety
    /// todo
    unsafe fn realloc(this: &Self, new_len: usize, data: &mut &'info mut [u8]) -> Result<()>;
    fn data_ref(this: &Self) -> Result<Ref<&'info mut [u8]>>;
    fn data_mut(this: &Self) -> Result<RefMut<&'info mut [u8]>>;
}

impl<'info> UnsizedTypeDataAccess<'info> for AccountInfo<'info> {
    unsafe fn realloc(this: &Self, new_len: usize, data: &mut &'info mut [u8]) -> Result<()> {
        // Return early if the length increase from the original serialized data
        // length is too large and would result in an out of bounds allocation.
        let original_data_len = unsafe { this.original_data_len() };
        ensure!(
            new_len.saturating_sub(original_data_len) <= MAX_PERMITTED_DATA_INCREASE,
            "Tried to realloc data to {new_len}. An increase over {MAX_PERMITTED_DATA_INCREASE} is not permitted",
        );
        let data_ptr = data.as_mut_ptr();

        // First set new length in the serialized data
        unsafe {
            data_ptr
                .offset(-8)
                .cast::<u64>()
                .write_unaligned(new_len as u64);
        }

        // Then recreate the local slice with the new length
        unsafe {
            *data = from_raw_parts_mut(data.as_mut_ptr(), new_len);
        }
        Ok(())
    }

    fn data_ref(this: &Self) -> Result<Ref<&'info mut [u8]>> {
        this.data
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)
            .with_context(|| format!("Error borrowing data on account {}", this.key))
    }

    fn data_mut(this: &Self) -> Result<RefMut<&'info mut [u8]>> {
        this.data
            .try_borrow_mut()
            .map_err(|_| ProgramError::AccountBorrowFailed)
            .with_context(|| format!("Error borrowing data on account {}", this.key))
    }
}

#[derive(Debug)]
pub struct SharedWrapper<'top, 'info, T: ?Sized> {
    r: Ref<'top, &'info mut [u8]>,
    data: T,
}

impl<'a, 'info, O> SharedWrapper<'a, 'info, O>
where
    O: UnsizedType + ?Sized,
{
    /// # Safety
    /// todo
    pub unsafe fn new(
        underlying_data: &'a impl UnsizedTypeDataAccess<'info>,
    ) -> Result<SharedWrapper<'a, 'info, O::Ref<'a>>> {
        let data = UnsizedTypeDataAccess::data_ref(underlying_data)?;
        let ptr = *data as *const [u8];
        Ok(SharedWrapper {
            r: data,
            data: O::get_ref(&mut unsafe { &*ptr })?,
        })
    }
}

impl<'a, 'info, T> SharedWrapper<'a, 'info, T> {
    pub fn map<U>(r: Self, f: impl FnOnce(T) -> U) -> SharedWrapper<'a, 'info, U> {
        SharedWrapper {
            r: r.r,
            data: f(r.data),
        }
    }
}

impl<T> Deref for SharedWrapper<'_, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub(crate) unsafe fn change_ref<'info, 'top>(
    the_ref: &'top mut &'info mut [u8],
) -> &'top mut *mut [u8] {
    unsafe { &mut *std::ptr::from_mut::<&'info mut [u8]>(the_ref).cast::<*mut [u8]>() }
}

pub(crate) unsafe fn change_ref_back<'info, 'top>(
    the_ptr: &'top mut *mut [u8],
) -> &'top mut &'info mut [u8] {
    unsafe { &mut *std::ptr::from_mut::<*mut [u8]>(the_ptr).cast::<&'info mut [u8]>() }
}

#[derive(Debug)]
pub enum MaybeOwnedR<'parent, 'top> {
    Owned(RefMut<'top, *mut [u8]>), // ptr is lifetime 'info
    Borrowed(&'parent mut RefMut<'top, *mut [u8]>),
}

impl<'parent, 'top> MaybeOwnedR<'parent, 'top> {
    pub fn get_ref<'a>(&'a mut self) -> &'a mut RefMut<'top, *mut [u8]> {
        match self {
            Self::Owned(r) => r,
            Self::Borrowed(r) => r,
        }
    }

    #[must_use]
    pub fn get_borrowed<'a>(&'a mut self) -> MaybeOwnedR<'a, 'top>
    where
        'parent: 'a,
    {
        match self {
            Self::Owned(r) => MaybeOwnedR::Borrowed(r),
            Self::Borrowed(r) => MaybeOwnedR::Borrowed(r),
        }
    }
}

pub enum MaybeOwnedPtr<O> {
    Owned(O),
    Borrowed(*mut O),
}

impl<O> fmt::Debug for MaybeOwnedPtr<O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = self.get_ptr();
        Pointer::fmt(&ptr, f)
    }
}

impl<O> MaybeOwnedPtr<O> {
    pub fn get_ptr(&self) -> *const O {
        match self {
            Self::Owned(o) => o,
            Self::Borrowed(o) => *o,
        }
    }

    pub fn get_mut_ptr(&mut self) -> *mut O {
        match self {
            Self::Owned(o) => o,
            Self::Borrowed(o) => *o,
        }
    }

    #[must_use]
    pub fn get_borrowed(&mut self) -> Self {
        match self {
            Self::Owned(o) => MaybeOwnedPtr::Borrowed(o),
            Self::Borrowed(o) => MaybeOwnedPtr::Borrowed(*o),
        }
    }
}

#[derive(Debug)]
pub struct ExclusiveWrapper<'parent, 'top, 'info, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    underlying_data: &'top A,
    r: MaybeOwnedR<'parent, 'top>,
    outer_data: MaybeOwnedPtr<<O as UnsizedType>::Mut<'top>>,
    phantom_o: PhantomData<fn() -> &'info O>,
    data: Option<*mut T>, // ptr is lifetime 'top
}

/// A convenience type where T is passed in as the [`UnsizedType`], instead of `UnsizedType::Mut`
pub type ExclusiveWrapperT<'parent, 'top, 'info, T, O, A> =
    ExclusiveWrapper<'parent, 'top, 'info, <T as UnsizedType>::Mut<'top>, O, A>;

impl<'top, 'info, O, A> ExclusiveWrapperT<'top, 'top, 'info, O, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
    'info: 'top,
{
    /// # Safety
    /// TODO:
    pub unsafe fn new(underlying_data: &'top A) -> Result<Self> {
        let mut r = RefMut::map(
            UnsizedTypeDataAccess::data_mut(underlying_data)?,
            |r| unsafe { change_ref(r) },
        );
        // ensure no ZSTs in middle of struct
        let _ = O::ZST_STATUS;

        // TODO: This is lifetime extension. Is this okay?
        let data = O::get_mut(unsafe { &mut &mut **r })?;

        Ok(Self {
            underlying_data,
            r: MaybeOwnedR::Owned(r),
            outer_data: MaybeOwnedPtr::Owned(data),
            phantom_o: PhantomData,
            data: None,
        })
    }
}

impl<T, O, A> ExclusiveWrapper<'_, '_, '_, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    /// # Safety
    /// T and O must be the same type if data is None
    unsafe fn get_data(&self) -> *const T {
        match self.data {
            Some(ptr) => ptr,
            None => self.outer_data.get_ptr().cast::<T>(),
        }
    }

    unsafe fn get_data_mut(&mut self) -> *mut T {
        match self.data {
            Some(ptr) => ptr,
            None => self.outer_data.get_mut_ptr().cast::<T>(),
        }
    }
}

impl<T, O, A> Deref for ExclusiveWrapper<'_, '_, '_, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.get_data() }
    }
}
impl<T, O, A> DerefMut for ExclusiveWrapper<'_, '_, '_, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.get_data_mut() }
    }
}

impl<'top, 'info, T, O, A> ExclusiveWrapper<'_, 'top, 'info, T, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
{
    /// # Safety
    /// todo
    pub unsafe fn map_ref<'child, U: 'top>(
        wrapper: &'child mut Self,
        f: impl FnOnce(&'top mut T) -> &'top mut U,
    ) -> ExclusiveWrapper<'child, 'top, 'info, U, O, A>
    where
        T: 'top,
    {
        unsafe {
            Self::try_map_ref(
                wrapper,
                #[inline]
                |data| Ok(f(data)),
            )
        }
        .unwrap()
    }

    /// # Safety
    /// TODO
    pub unsafe fn try_map_ref<'child, U: 'top>(
        wrapper: &'child mut Self,
        f: impl FnOnce(&'top mut T) -> Result<&'top mut U>,
    ) -> Result<ExclusiveWrapper<'child, 'top, 'info, U, O, A>>
    where
        T: 'top,
    {
        let data = core::ptr::from_mut(f(unsafe { &mut *wrapper.get_data_mut() })?);
        Ok(ExclusiveWrapper {
            underlying_data: wrapper.underlying_data,
            outer_data: wrapper.outer_data.get_borrowed(),
            r: wrapper.r.get_borrowed(),
            data: data.into(),
            phantom_o: PhantomData,
        })
    }

    /// # Safety
    /// TODO
    pub unsafe fn set_inner<U>(
        wrapper: &mut Self,
        f: impl FnOnce(&'_ mut T) -> Result<U>,
    ) -> Result<U> {
        f(wrapper)
    }

    /// # Safety
    /// TODO
    pub unsafe fn add_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        start: *const (),
        amount: usize,
        after_add: impl FnOnce(&mut T) -> Result<()>,
    ) -> Result<()> {
        {
            let data = unsafe { change_ref_back(wrapper.r.get_ref()) };
            let old_ptr = data.as_ptr();
            let old_len = data.len();

            ensure!(start as usize >= data.as_ptr() as usize);
            ensure!(start as usize <= data.as_ptr() as usize + old_len);

            // Return early if length hasn't changed
            if amount == 0 {
                return Ok(());
            }
            let new_len = old_len + amount;

            // realloc
            unsafe { UnsizedTypeDataAccess::realloc(wrapper.underlying_data, new_len, data) }?;

            if start as usize != old_ptr as usize + old_len {
                unsafe {
                    sol_memmove(
                        start.cast::<u8>().add(amount).cast_mut(),
                        start.cast_mut().cast::<u8>(),
                        old_len - (start as usize - data.as_ptr() as usize),
                    );
                }
            }
        }

        after_add(unsafe { &mut *wrapper.get_data_mut() })?;

        unsafe {
            <O as UnsizedType>::resize_notification(
                &mut *wrapper.outer_data.get_mut_ptr(),
                source_ptr,
                amount.try_into()?,
            )?;
        }
        Ok(())
    }

    /// # Safety
    /// TODO
    pub unsafe fn remove_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        range: impl RangeBounds<*const ()>,
        after_remove: impl FnOnce(&mut T) -> Result<()>,
    ) -> Result<()> {
        let amount = {
            let data = unsafe { change_ref_back(wrapper.r.get_ref()) };
            let old_len = data.len();

            let start = match range.start_bound() {
                Bound::Included(start) => {
                    ensure!(*start as usize >= data.as_ptr() as usize);
                    ensure!(*start as usize <= data.as_ptr() as usize + old_len);
                    start.cast::<u8>()
                }
                Bound::Excluded(start) => {
                    ensure!(*start as usize >= data.as_ptr() as usize);
                    ensure!(*start as usize <= data.as_ptr() as usize + old_len);
                    unsafe { start.cast::<u8>().add(1) }
                }
                Bound::Unbounded => data.as_ptr(),
            };

            let end = match range.end_bound() {
                Bound::Included(end) => {
                    ensure!(*end as usize >= start as usize);
                    ensure!((*end as usize) < data.as_ptr() as usize + old_len);
                    unsafe { end.cast::<u8>().add(1) }
                }
                Bound::Excluded(end) => {
                    ensure!(*end as usize >= start as usize);
                    ensure!(*end as usize <= data.as_ptr() as usize + old_len);
                    end.cast::<u8>()
                }
                Bound::Unbounded => unsafe { data.as_ptr().add(old_len) },
            };

            let amount = end as usize - start as usize;
            if amount == 0 {
                return Ok(());
            }

            if end as usize != data.as_ptr() as usize + old_len {
                unsafe {
                    sol_memmove(
                        start.cast_mut(),
                        end.cast_mut(),
                        old_len - (end as usize - data.as_ptr() as usize),
                    );
                }
            }

            let new_len = old_len - amount;
            // realloc
            unsafe {
                UnsizedTypeDataAccess::realloc(wrapper.underlying_data, new_len, data)?;
            }

            amount
        };

        after_remove(unsafe { &mut *wrapper.get_data_mut() })?;

        unsafe {
            <O as UnsizedType>::resize_notification(
                &mut *wrapper.outer_data.get_mut_ptr(),
                source_ptr,
                -amount.try_into()?,
            )?;
        }
        Ok(())
    }

    /// # Safety
    /// TODO
    pub unsafe fn compute_len<U>(wrapper: &mut Self, start_ptr: *const ()) -> Result<usize>
    where
        U: UnsizedType + ?Sized,
    {
        let start_usize = start_ptr as usize;
        let mut data = &unsafe { change_ref_back(wrapper.r.get_ref()) }[..];
        let data_usize = data.as_ptr() as usize;
        let start_offset = start_usize - data_usize;
        data.try_advance(start_offset).with_context(|| {
            format!(
                "Failed to advance {} bytes to start offset during compute_len for type {}",
                start_offset,
                std::any::type_name::<U>()
            )
        })?;
        U::get_ref(&mut data)?;
        let end_usize = data.as_ptr() as usize;
        Ok(end_usize - start_usize)
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct StartPointer<T> {
    start: *const (),
    #[deref]
    #[deref_mut]
    pub data: T,
}

impl<T> AsShared for StartPointer<T>
where
    T: AsShared,
{
    type Ref<'b>
        = T::Ref<'b>
    where
        Self: 'b;
    fn as_shared(&self) -> Self::Ref<'_> {
        self.data.as_shared()
    }
}

impl<T> StartPointer<T> {
    /// # Safety
    /// todo
    pub unsafe fn new(start: *const (), data: T) -> Self {
        Self { start, data }
    }

    /// # Safety
    /// todo
    pub unsafe fn handle_resize_notification(s: &mut Self, source_ptr: *const (), change: isize) {
        if source_ptr < s.start {
            s.start = unsafe { s.start.byte_offset(change) };
        }
    }
}

impl<'ptr, 'info, O, A, T> ExclusiveWrapper<'_, '_, 'info, StartPointer<T>, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
{
    /// # Safety
    /// todo
    // todo: maybe rename this?
    pub unsafe fn set_start_pointer_data<U, I>(wrapper: &mut Self, init_arg: I) -> Result<()>
    where
        U: UnsizedType<Mut<'ptr> = StartPointer<T>> + UnsizedInit<I>,
    {
        let current_len = unsafe { ExclusiveWrapper::compute_len::<U>(wrapper, wrapper.start)? };
        let new_len = <U as UnsizedInit<I>>::INIT_BYTES;

        match current_len.cmp(&new_len) {
            Ordering::Less => {
                unsafe {
                    ExclusiveWrapper::add_bytes(
                        wrapper,
                        wrapper.start,
                        wrapper.start,
                        new_len - current_len,
                        |_| Ok(()),
                    )
                }?;
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                unsafe {
                    ExclusiveWrapper::remove_bytes(
                        wrapper,
                        wrapper.start,
                        wrapper.start..wrapper.start.byte_add(current_len - new_len),
                        |_| Ok(()),
                    )
                }?;
            }
        }
        unsafe {
            ExclusiveWrapper::set_inner(wrapper, |data: &mut StartPointer<T>| {
                let slice = from_raw_parts_mut(data.start.cast_mut().cast::<u8>(), new_len);
                <U as UnsizedInit<I>>::init(&mut &mut slice[..], init_arg)?;
                let new_data = U::get_mut(&mut &mut slice[..])?;
                data.data = new_data.data;
                Ok(())
            })
        }?;
        Ok(())
    }
}
