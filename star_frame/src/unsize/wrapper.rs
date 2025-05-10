use super::{AsShared, UnsizedType};
use super::{OwnedRef, OwnedRefMut};
use crate::prelude::UnsizedInit;
use crate::Result;
use advancer::Advance;
use anyhow::{ensure, Context};
use core::ptr;
use derive_more::{Debug, Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memmove;
use std::cell::{Ref, RefMut};
use std::cmp::Ordering;
use std::collections::Bound;
use std::convert::Infallible;
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
pub struct SharedWrapper<'top, T> {
    r: OwnedRef<'top, T>,
}

impl<'a, T> SharedWrapper<'a, T> {
    /// # Safety
    /// todo
    pub unsafe fn new<'info, U>(
        underlying_data: &'a impl UnsizedTypeDataAccess<'info>,
    ) -> Result<Self>
    where
        'info: 'a,
        T: 'a,
        U: UnsizedType<Ref<'a> = T> + ?Sized,
    {
        // ensure no ZSTs in middle of struct
        let _ = U::ZST_STATUS;
        let data = UnsizedTypeDataAccess::data_ref(underlying_data)?;
        let r = OwnedRef::try_new(data, |r| U::get_ref(&mut &**r))?;
        Ok(SharedWrapper { r })
    }
}
impl<T> Deref for SharedWrapper<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

#[derive(Debug)]
pub enum ExclusiveWrapper<'parent, 'top, Mut, P> {
    Top {
        data: OwnedRefMut<'top, (P, Mut)>,
    },
    Inner {
        parent_lt: PhantomData<&'parent ()>,
        parent: *mut P, // 'parent
        field: *mut Mut,
    },
}

pub type ExclusiveWrapperTop<'top, Top, A> = ExclusiveWrapper<
    'top,
    'top,
    <Top as UnsizedType>::Mut<'top>,
    ExclusiveWrapperTopMeta<'top, Top, A>,
>;

#[derive(Debug)]
pub struct ExclusiveWrapperTopMeta<'top, Top, A>
where
    Top: UnsizedType + ?Sized,
{
    info: &'top A,
    data: *mut *mut [u8],                  // &'top mut &'info mut [u8]
    top_phantom: PhantomData<fn() -> Top>, // Giving this a `Top` to allow ExclusiveWrapperTop to be impl'd on
}

impl<'top, 'info, Top, A> ExclusiveWrapperTop<'top, Top, A>
where
    'info: 'top,
    Top: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
{
    pub fn new(info: &'top A) -> Result<Self> {
        // ensure no ZSTs in middle of struct
        let _ = Top::ZST_STATUS;
        let data: RefMut<'top, &mut [u8]> = UnsizedTypeDataAccess::data_mut(info)?;
        Ok(ExclusiveWrapper::Top {
            data: OwnedRefMut::try_new(data, |data| {
                anyhow::Ok((
                    ExclusiveWrapperTopMeta {
                        info,
                        data: ptr::from_mut(data).cast::<*mut [u8]>(),
                        top_phantom: PhantomData,
                    },
                    Top::get_mut(&mut &mut **data)?,
                ))
            })?,
        })
    }
}

impl<Mut, P> ExclusiveWrapper<'_, '_, Mut, P> {
    fn mut_ref(this: &Self) -> &Mut {
        match this {
            ExclusiveWrapper::Top { data, .. } => &data.1,
            ExclusiveWrapper::Inner { field, .. } => {
                // SAFETY:
                // We have shared access to self right now, so no mutable references to self can exist.
                // Field is created in ExclusiveWrapper::new, and this pointer is derived from ExclusiveWrapper::map,
                // which takes in &mut self, so no references to upper fields can exist at the same time.
                // Self cannot be used again until the mut_ref is dropped due to lifetimes, so no other references to field can be created
                // while mut_ref is still alive.
                unsafe { &**field }
            }
        }
    }

    fn mut_mut(this: &mut Self) -> &mut Mut {
        match this {
            ExclusiveWrapper::Top { data, .. } => &mut data.1,
            ExclusiveWrapper::Inner { field, .. } => {
                // SAFETY:
                // We have exclusive access to self right now, so no mutable references to self can exist.
                // Field is created in ExclusiveWrapper::new, and this pointer is derived from ExclusiveWrapper::map,
                // which takes in &mut self, so no references to upper fields can exist at the same time.
                // Self cannot be used again until the mut_mut is dropped due to lifetimes, so no other references to field can be created
                // while mut_mut is still alive.
                unsafe { &mut **field }
            }
        }
    }
}
mod sealed {
    use super::*;

    pub trait Sealed {}
    impl<Mut, P> Sealed for ExclusiveWrapper<'_, '_, Mut, P> {}
}

pub trait ExclusiveRecurse: sealed::Sealed + Sized {
    /// # Safety
    /// Is this actually unsafe? If bounds are checked, everything should be fine? We have exclusive access to self right now.
    unsafe fn add_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        start: *mut (),
        amount: usize,
    ) -> Result<()>;
    /// # Safety
    /// Is this actually unsafe? If bounds are checked, everything should be fine? We have exclusive access to self right now.
    unsafe fn remove_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        range: impl RangeBounds<*mut ()>,
    ) -> Result<()>;

    /// # Safety
    /// Is this actually unsafe? If bounds are checked, everything should be fine? We have exclusive access to self right now.
    unsafe fn compute_len<UT: UnsizedType + ?Sized>(
        wrapper: &mut Self,
        start_ptr: *const (),
    ) -> Result<usize>;
}

impl<Mut, P> ExclusiveRecurse for ExclusiveWrapper<'_, '_, Mut, P>
where
    P: ExclusiveRecurse,
{
    unsafe fn add_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        start: *mut (),
        amount: usize,
    ) -> Result<()> {
        match wrapper {
            ExclusiveWrapper::Top { .. } => unreachable!(),
            ExclusiveWrapper::Inner { parent, .. } => {
                // SAFETY:
                // We have exclusive access to self right now, and no other references to parent can exist.
                let parent = unsafe { &mut **parent };
                unsafe { P::add_bytes(parent, source_ptr, start, amount) }
            }
        }
    }

    unsafe fn remove_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        range: impl RangeBounds<*mut ()>,
    ) -> Result<()> {
        match wrapper {
            ExclusiveWrapper::Top { .. } => unreachable!(),
            ExclusiveWrapper::Inner { parent, .. } => {
                // SAFETY:
                // We have exclusive access to self right now, so no other references to parent can exist.
                let parent = unsafe { &mut **parent };
                unsafe { P::remove_bytes(parent, source_ptr, range) }
            }
        }
    }

    unsafe fn compute_len<UT: UnsizedType + ?Sized>(
        wrapper: &mut Self,
        start_ptr: *const (),
    ) -> Result<usize> {
        match wrapper {
            ExclusiveWrapper::Top { .. } => unreachable!(),
            ExclusiveWrapper::Inner { parent, .. } => {
                let parent = unsafe { &mut **parent };
                unsafe { P::compute_len::<UT>(parent, start_ptr) }
            }
        }
    }
}

impl<'top, 'info, Top, A> ExclusiveRecurse for ExclusiveWrapperTop<'top, Top, A>
where
    Top: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
    'info: 'top,
{
    unsafe fn add_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        start: *mut (),
        amount: usize,
    ) -> Result<()> {
        let s: &mut OwnedRefMut<'_, (ExclusiveWrapperTopMeta<'top, Top, A>, Top::Mut<'top>)> =
            match wrapper {
                ExclusiveWrapper::Top { data, .. } => data,
                ExclusiveWrapper::Inner { .. } => unreachable!(),
            };

        {
            // SAFETY: We are at the top level now, and all the child `field()`s can only contain mutable pointers to the data, so we are the only one
            let data: &'top mut &'info mut [u8] = unsafe { &mut *s.0.data.cast() };
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
            unsafe { UnsizedTypeDataAccess::realloc(s.0.info, new_len, data) }?;

            if start as usize != old_ptr as usize + old_len {
                unsafe {
                    sol_memmove(
                        start.cast::<u8>().add(amount),
                        start.cast::<u8>(),
                        old_len - (start as usize - data.as_ptr() as usize),
                    );
                }
            }
        }

        // TODO: Figure out the safety requirements of calling this. I think it is safe to call here assuming the UnsizedType is implemented correctly.
        unsafe {
            Top::resize_notification(&mut s.1, source_ptr, amount.try_into()?)?;
        }

        Ok(())
    }

    unsafe fn remove_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        range: impl RangeBounds<*mut ()>,
        // after_remove: impl FnOnce(&mut Self::T) -> Result<()>,
    ) -> Result<()> {
        let s = match wrapper {
            ExclusiveWrapper::Top { data, .. } => data,
            ExclusiveWrapper::Inner { .. } => unreachable!(),
        };

        let amount = {
            let data: &'top mut &'info mut [u8] = unsafe { &mut *s.0.data.cast() };
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
                Bound::Unbounded => data.as_mut_ptr(),
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
                Bound::Unbounded => unsafe { data.as_mut_ptr().add(old_len) },
            };

            let amount = end as usize - start as usize;
            if amount == 0 {
                return Ok(());
            }

            if end as usize != data.as_ptr() as usize + old_len {
                unsafe {
                    sol_memmove(
                        start,
                        end,
                        old_len - (end as usize - data.as_ptr() as usize),
                    );
                }
            }

            let new_len = old_len - amount;
            // TODO: Figure out the safety requirements of calling this
            unsafe {
                UnsizedTypeDataAccess::realloc(s.0.info, new_len, data)?;
            }

            amount
        };

        unsafe {
            Top::resize_notification(&mut s.1, source_ptr, -amount.try_into()?)?;
        }
        Ok(())
    }

    unsafe fn compute_len<UT: UnsizedType + ?Sized>(
        wrapper: &mut Self,
        start_ptr: *const (),
    ) -> Result<usize> {
        let s = match wrapper {
            ExclusiveWrapper::Top { data, .. } => data,
            ExclusiveWrapper::Inner { .. } => unreachable!(),
        };

        let data: &'top mut &'info mut [u8] = unsafe { &mut *s.0.data.cast() };
        let mut data = &data[..];

        let start_usize = start_ptr as usize;
        let data_usize = data.as_ptr() as usize;
        let start_offset = start_usize - data_usize;
        data.try_advance(start_offset).with_context(|| {
            format!(
                "Failed to advance {} bytes to start offset during compute_len for type {}",
                start_offset,
                std::any::type_name::<UT>()
            )
        })?;
        UT::get_ref(&mut data)?;
        let end_usize = data.as_ptr() as usize;
        Ok(end_usize - start_usize)
    }
}

impl<'top, Mut, P> ExclusiveWrapper<'_, 'top, Mut, P>
where
    Self: ExclusiveRecurse,
{
    /// # Safety
    /// O may not contain a mutable reference to T, but can contain a mutable pointer.
    pub unsafe fn map_mut<'child, O>(
        parent: &'child mut Self,
        mapper: impl FnOnce(&'child mut Mut) -> &'child mut O::Mut<'top>,
    ) -> ExclusiveWrapper<'child, 'top, O::Mut<'top>, Self>
    where
        O: UnsizedType + ?Sized,
    {
        unsafe { Self::try_map_mut::<O, Infallible>(parent, |m| Ok(mapper(m))) }.unwrap()
    }

    /// # Safety
    /// O may not contain a mutable reference to T, but can contain a mutable pointer.
    pub unsafe fn try_map_mut<'child, O, E>(
        parent: &'child mut Self,
        mapper: impl FnOnce(&'child mut Mut) -> Result<&'child mut O::Mut<'top>, E>,
    ) -> Result<ExclusiveWrapper<'child, 'top, O::Mut<'top>, Self>, E>
    where
        O: UnsizedType + ?Sized,
    {
        let parent_mut: *mut Self = parent;
        Ok(ExclusiveWrapper::Inner {
            parent_lt: PhantomData,
            parent: parent_mut,
            field: mapper(Self::mut_mut(parent))?,
        })
    }
}

impl<Mut, P> Deref for ExclusiveWrapper<'_, '_, Mut, P>
where
    Self: ExclusiveRecurse,
{
    type Target = Mut;

    fn deref(&self) -> &Self::Target {
        Self::mut_ref(self)
    }
}

impl<Mut, P> DerefMut for ExclusiveWrapper<'_, '_, Mut, P>
where
    Self: ExclusiveRecurse,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::mut_mut(self)
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct StartPointer<T> {
    start: *mut (),
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
    pub unsafe fn new(start: *mut (), data: T) -> Self {
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

impl<'top, Mut, P> ExclusiveWrapper<'_, 'top, StartPointer<Mut>, P>
where
    Self: ExclusiveRecurse,
{
    /// # Safety
    // TODO: I think it might be safe since we're relying on the UnsizedType implementation to be correct.
    pub unsafe fn set_start_pointer_data<U, I>(wrapper: &mut Self, init_arg: I) -> Result<()>
    where
        U: UnsizedType<Mut<'top> = StartPointer<Mut>> + UnsizedInit<I> + ?Sized,
    {
        // SAFETY:
        // TODO: might be safe
        let current_len = unsafe { Self::compute_len::<U>(wrapper, wrapper.start)? };
        let new_len = <U as UnsizedInit<I>>::INIT_BYTES;

        match current_len.cmp(&new_len) {
            Ordering::Less => {
                // TODO: might be safe
                unsafe {
                    ExclusiveWrapper::add_bytes(
                        wrapper,
                        wrapper.start,
                        wrapper.start,
                        new_len - current_len,
                    )
                }?;
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                // TODO: might be safe
                unsafe {
                    ExclusiveWrapper::remove_bytes(
                        wrapper,
                        wrapper.start,
                        wrapper.start..wrapper.start.byte_add(current_len - new_len),
                    )
                }?;
            }
        }
        wrapper.data = {
            // SAFETY:
            // We have exclusive access to the wrapper, so no external references to the underlying data exist.
            // No other references exist in this function either. We can assume the StartPointer is valid since it was created by the UnsizedType implementation.
            let slice = unsafe { from_raw_parts_mut(wrapper.start.cast::<u8>(), new_len) };
            // TODO: this is probably safe
            unsafe { <U as UnsizedInit<I>>::init(&mut &mut slice[..], init_arg)? };
            U::get_mut(&mut &mut slice[..])?.data
        };
        Ok(())
    }
}
