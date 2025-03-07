use super::{ResizeOperation, UnsizedType};
use crate::Result;
use anyhow::ensure;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::program_memory::sol_memmove;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::Bound;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, RangeBounds};
use std::slice::from_raw_parts_mut;

pub trait UnsizedTypeDataAccess<'info> {
    unsafe fn realloc(&self, new_len: usize, data: &mut &'info mut [u8]) -> Result<()>;
    fn data(&self) -> &RefCell<&'info mut [u8]>;
}

impl<'info> UnsizedTypeDataAccess<'info> for AccountInfo<'info> {
    unsafe fn realloc(&self, new_len: usize, data: &mut &'info mut [u8]) -> Result<()> {
        // Return early if the length increase from the original serialized data
        // length is too large and would result in an out of bounds allocation.
        let original_data_len = unsafe { self.original_data_len() };
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

    fn data(&self) -> &RefCell<&'info mut [u8]> {
        &self.data
    }
}

#[derive(Debug)]
pub struct SharedWrapper<'a, 'info, T: ?Sized> {
    r: Ref<'a, &'info mut [u8]>,
    data: T,
}

impl<'a, 'info, O> SharedWrapper<'a, 'info, O>
where
    O: UnsizedType + ?Sized,
{
    pub unsafe fn new(
        underlying_data: &'a impl UnsizedTypeDataAccess<'info>,
    ) -> Result<SharedWrapper<'a, 'info, O::Ref<'a>>> {
        let data = underlying_data.data().borrow();
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

unsafe fn change_ref<'info, 'a>(the_ref: &'a mut &'info mut [u8]) -> &'a mut *mut [u8] {
    unsafe { &mut *std::ptr::from_mut::<&'info mut [u8]>(the_ref).cast::<*mut [u8]>() }
}

unsafe fn change_ref_back<'info, 'a>(the_ptr: &'a mut *mut [u8]) -> &'a mut &'info mut [u8] {
    unsafe { &mut *std::ptr::from_mut::<*mut [u8]>(the_ptr).cast::<&'info mut [u8]>() }
}

#[derive(Debug)]
pub struct ExclusiveWrapper<'a, 'info, T, O: ?Sized, A> {
    underlying_data: &'a A,
    r: RefMut<'a, *mut [u8]>,
    phantom_o: PhantomData<fn() -> &'info O>,
    data: T,
}

impl<'a, 'info, O, A> ExclusiveWrapper<'a, 'info, O::Mut<'_>, O, A>
where
    'info: 'a,
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
{
    pub unsafe fn new(underlying_data: &'a A) -> Result<Self> {
        let mut r = RefMut::map(underlying_data.data().borrow_mut(), |r| unsafe {
            change_ref(r)
        });
        let data = O::get_mut(unsafe { &mut &mut **r })?;
        Ok(Self {
            underlying_data,
            r,
            phantom_o: PhantomData,
            data,
        })
    }
}

impl<'a, 'info, T, O: UnsizedType + ?Sized, A: UnsizedTypeDataAccess<'info>>
    ExclusiveWrapper<'a, 'info, T, O, A>
{
    // TODO: Maybe can not be unsafe, but maybe not?
    /// # Safety
    /// TODO
    pub unsafe fn map<U>(r: Self, f: impl FnOnce(T) -> U) -> ExclusiveWrapper<'a, 'info, U, O, A> {
        ExclusiveWrapper {
            underlying_data: r.underlying_data,
            r: r.r,
            data: f(r.data),
            phantom_o: PhantomData,
        }
    }

    /// # Safety
    /// TODO
    pub unsafe fn set_inner<U>(r: &mut Self, f: impl FnOnce(&mut T) -> U) -> U {
        f(&mut r.data)
    }
    /// # Safety
    /// TODO
    pub unsafe fn add_bytes(
        r: &mut Self,
        start: *const (),
        amount: usize,
        after_add: impl FnOnce(&mut T) -> Result<()>,
    ) -> Result<()> {
        let data = unsafe { change_ref_back(&mut r.r) };
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
        unsafe { r.underlying_data.realloc(new_len, data) }?;

        if start as usize != old_ptr as usize + old_len {
            unsafe {
                sol_memmove(
                    start.cast::<u8>().add(amount).cast_mut(),
                    start.cast_mut().cast::<u8>(),
                    old_len - (start as usize - data.as_ptr() as usize),
                );
            }
        }

        after_add(&mut r.data)?;

        unsafe {
            <O as UnsizedType>::resize_notification(
                &mut &mut data[..],
                ResizeOperation::Add { start, amount },
            )?;
        }
        Ok(())
    }
    /// # Safety
    /// TODO
    pub unsafe fn remove_bytes(
        r: &mut Self,
        range: impl RangeBounds<*const ()>,
        after_remove: impl FnOnce(&mut T) -> Result<()>,
    ) -> Result<()> {
        let data = unsafe { change_ref_back(&mut r.r) };
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
            r.underlying_data.realloc(new_len, data)?;
        }

        after_remove(&mut r.data)?;

        unsafe {
            <O as UnsizedType>::resize_notification(
                &mut &mut data[..],
                ResizeOperation::Remove {
                    start: start.cast(),
                    end: end.cast(),
                },
            )?;
        }
        Ok(())
    }
}
impl<T, O: ?Sized, A> Deref for ExclusiveWrapper<'_, '_, T, O, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T, O: ?Sized, A> DerefMut for ExclusiveWrapper<'_, '_, T, O, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
