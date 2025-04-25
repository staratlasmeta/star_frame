use super::{AsShared, UnsizedType};
use crate::prelude::UnsizedInit;
use crate::Result;
use advancer::Advance;
use anyhow::{ensure, Context};
use core::fmt;
use core::ptr;
use derive_more::{Debug, Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memmove;
use std::any::type_name;
use std::cell::{Ref, RefMut};
use std::cmp::Ordering;
use std::collections::Bound;
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
    unsafe { &mut *ptr::from_mut::<&'info mut [u8]>(the_ref).cast::<*mut [u8]>() }
}

pub(crate) unsafe fn change_ref_back<'info, 'top>(
    the_ptr: &'top mut *mut [u8],
) -> &'top mut &'info mut [u8] {
    unsafe { &mut *ptr::from_mut::<*mut [u8]>(the_ptr).cast::<&'info mut [u8]>() }
}

enum MaybeOwned<'parent, 'top, O> {
    Owned {
        r: RefMut<'top, *mut [u8]>,
        outer_data: O,
    },
    Borrowed {
        r: &'parent mut RefMut<'top, *mut [u8]>,
        outer_data: *mut O,
    },
}

impl<'parent, 'top, O> MaybeOwned<'parent, 'top, O> {
    fn borrowed<'a>(&'a mut self) -> MaybeOwned<'a, 'top, O>
    where
        'parent: 'a,
    {
        match self {
            Self::Owned { r, outer_data } => MaybeOwned::Borrowed { r, outer_data },
            Self::Borrowed { r, outer_data } => MaybeOwned::Borrowed {
                r,
                outer_data: *outer_data,
            },
        }
    }

    fn r(&mut self) -> &mut RefMut<'top, *mut [u8]> {
        match self {
            MaybeOwned::Owned { r, .. } => r,
            MaybeOwned::Borrowed { r, .. } => r,
        }
    }

    fn outer_data(&self) -> *const O {
        match self {
            MaybeOwned::Owned { outer_data, .. } => outer_data,
            MaybeOwned::Borrowed { outer_data, .. } => *outer_data,
        }
    }

    fn outer_data_mut(&mut self) -> *mut O {
        match self {
            MaybeOwned::Owned { outer_data, .. } => outer_data,
            MaybeOwned::Borrowed { outer_data, .. } => *outer_data,
        }
    }
}

impl<O> fmt::Debug for MaybeOwned<'_, '_, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (variant, r, data): (_, _, *const O) = match self {
            Self::Owned { r, outer_data } => ("Owned", r, outer_data),
            Self::Borrowed { r, outer_data } => ("Borrowed", *r, *outer_data),
        };
        f.debug_struct(variant)
            .field("r", r)
            .field("data", &data)
            .finish()
    }
}

#[derive(Debug)]
pub struct ExclusiveWrapper<'parent, 'top, 'info, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    underlying_data: &'top A,
    maybe_owned: MaybeOwned<'parent, 'top, O::Mut<'top>>,
    phantom_o: PhantomData<fn() -> &'info O>,
    // Data must be None while maybe_owned is Owned
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
            maybe_owned: MaybeOwned::Owned {
                r,
                outer_data: data,
            },
            phantom_o: PhantomData,
            data: None,
        })
    }
}

impl<'top, T, O, A> ExclusiveWrapper<'_, 'top, '_, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    /// # Safety
    /// T and O must be the same type if data is None
    unsafe fn data(&self) -> *const T {
        if let Some(ptr) = self.data { ptr } else {
            // Not ideal, but we can't use TypeId because T isn't 'static
            debug_assert!(type_name::<T>() == type_name::<O::Mut<'top>>());
            self.maybe_owned.outer_data().cast::<T>()
        }
    }

    unsafe fn data_mut(&mut self) -> *mut T {
        match self.data {
            Some(ptr) => ptr,
            None => self.maybe_owned.outer_data_mut().cast::<T>(),
        }
    }
}

impl<T, O, A> Deref for ExclusiveWrapper<'_, '_, '_, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data() }
    }
}
impl<T, O, A> DerefMut for ExclusiveWrapper<'_, '_, '_, T, O, A>
where
    O: UnsizedType + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data_mut() }
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
        let data = ptr::from_mut(f(unsafe { &mut *wrapper.data_mut() })?);
        Ok(ExclusiveWrapper {
            underlying_data: wrapper.underlying_data,
            maybe_owned: wrapper.maybe_owned.borrowed(),
            data: Some(data),
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
            let data = unsafe { change_ref_back(wrapper.maybe_owned.r()) };
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

        after_add(unsafe { &mut *wrapper.data_mut() })?;

        unsafe {
            <O as UnsizedType>::resize_notification(
                &mut *wrapper.maybe_owned.outer_data_mut(),
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
            let data = unsafe { change_ref_back(wrapper.maybe_owned.r()) };
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

        after_remove(unsafe { &mut *wrapper.data_mut() })?;

        unsafe {
            <O as UnsizedType>::resize_notification(
                &mut *wrapper.maybe_owned.outer_data_mut(),
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
        let mut data = &unsafe { change_ref_back(wrapper.maybe_owned.r()) }[..];
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
