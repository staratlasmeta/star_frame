use super::{AsShared, UnsizedType};
use super::{OwnedRef, OwnedRefMut};
use crate::prelude::UnsizedInit;
use crate::Result;
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
use std::ops::{AddAssign, Deref, DerefMut, RangeBounds, SubAssign};

/// # Safety
/// [`Self::unsized_data_realloc`] must properly check the new length of the underlying data pointer.
pub unsafe trait UnsizedTypeDataAccess<'info> {
    /// # Safety
    /// `data` must actually point to the same data that is returned by [`UnsizedTypeDataAccess::data_ref`] and [`UnsizedTypeDataAccess::data_mut`].
    unsafe fn unsized_data_realloc(this: &Self, data: &mut *mut [u8], new_len: usize)
        -> Result<()>;
    fn data_ref(this: &Self) -> Result<Ref<&'info mut [u8]>>;
    fn data_mut(this: &Self) -> Result<RefMut<&'info mut [u8]>>;
}

/// # Safety
/// We are checking the length of the underlying data pointer in [`Self::unsized_data_realloc`].
unsafe impl<'info> UnsizedTypeDataAccess<'info> for AccountInfo<'info> {
    /// Sets the data length in the serialized data. This is identical to how [`AccountInfo::realloc`] works, minus the `RefCell::borrow_mut` call.
    /// # Safety
    /// `data` must actually point to the underlying data of the account, and `Self` needs to be from the entrypoint of a program or
    /// have its memory laid out in the same way. The fact that you can create `AccountInfo`s that aren't like this is a tragedy, but it is what it is.
    unsafe fn unsized_data_realloc(
        this: &Self,
        data: &mut *mut [u8],
        new_len: usize,
    ) -> Result<()> {
        // Return early if the length increase from the original serialized data
        // length is too large and would result in an out of bounds allocation.
        let original_data_len = unsafe { this.original_data_len() };
        ensure!(
            new_len.saturating_sub(original_data_len) <= MAX_PERMITTED_DATA_INCREASE,
            "Tried to realloc data to {new_len}. An increase over {MAX_PERMITTED_DATA_INCREASE} is not permitted",
        );

        // Set new length in the serialized data. Very questionable, but it's what solana does in `Self::realloc`.
        unsafe {
            data.cast::<u8>()
                .wrapping_offset(-8)
                .cast::<u64>()
                .write_unaligned(new_len as u64);
        }

        // Then recreate the local slice with the new length
        *data = ptr_meta::from_raw_parts_mut(data.cast(), new_len);
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

/// The heart of the `UnsizedType` system. This wrapper enables resizing through the [`ExclusiveRecurse`] trait, and mapping to
/// child wrappers through [`Self::try_map_mut`]. In addition, this impl's [`DerefMut`] and [`Deref`] for easy access to the
///
#[derive(Debug)]
pub enum ExclusiveWrapper<'parent, 'top, Mut, P> {
    Top {
        // We Box the Mut so its derived from a separate allocation, so dereferencing parent in inners wont invalidate the top Mut.
        data: OwnedRefMut<'top, (P, Box<Mut>)>,
    },
    Inner {
        parent_lt: PhantomData<&'parent ()>,
        parent: *mut P, // 'parent
        field: *mut Mut,
    },
}

type ExclusiveWrapperTopVariant<'top, Top, A> = OwnedRefMut<
    'top,
    (
        ExclusiveWrapperTopMeta<'top, Top, A>,
        Box<<Top as UnsizedType>::Mut<'top>>,
    ),
>;

pub type ExclusiveWrapperTop<'top, Top, A> = ExclusiveWrapper<
    'top,
    'top,
    <Top as UnsizedType>::Mut<'top>,
    ExclusiveWrapperTopMeta<'top, Top, A>,
>;

/// The generic `P` for an [`ExclusiveWrapper`] that is at the top level of the wrapper stack.
#[derive(Debug)]
pub struct ExclusiveWrapperTopMeta<'top, Top, A>
where
    Top: UnsizedType + ?Sized,
{
    info: &'top A,
    /// The pointer to the contiguous allocated slice. The len metadata may be shorter than the actual length of the allocated slice.
    /// It's lifetimes should match `&'top mut &'info mut [u8]` when run in [`ExclusiveWrapperTop::new`].
    data: *mut *mut [u8],
    /// This allows inherent implemenations on [`ExclusiveWrapperTop`].
    top_phantom: PhantomData<fn() -> Top>,
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
                let data_ptr = ptr::from_mut(data).cast::<*mut [u8]>();
                // SAFETY:
                // we just made this pointer, so its safe to dereference.
                let mut data = unsafe { *data_ptr }; // pointer lasts for 'top.
                anyhow::Ok((
                    ExclusiveWrapperTopMeta {
                        info,
                        data: data_ptr,
                        top_phantom: PhantomData,
                    },
                    // SAFETY:
                    // The pointer lasts for 'top, and we made the pointer from a properly formed slice, so its metadata is valid.
                    unsafe { Box::new(Top::get_mut(&mut data)?) },
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

    fn mut_mut(this: &mut Self) -> *mut Mut {
        match this {
            ExclusiveWrapper::Top { data, .. } => &raw mut *(data.1),
            ExclusiveWrapper::Inner { field, .. } => {
                // SAFETY:
                // We have exclusive access to self right now, so no mutable references to self can exist.
                // Field is created in ExclusiveWrapper::new, and this pointer is derived from ExclusiveWrapper::map,
                // which takes in &mut self, so no references to upper fields can exist at the same time.
                // Self cannot be used again until the mut_mut is dropped due to lifetimes, so no other references to field can be created
                // while mut_mut is still alive.
                *field
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
        let s: &mut ExclusiveWrapperTopVariant<'top, Top, A> = match wrapper {
            ExclusiveWrapper::Top { data, .. } => data,
            ExclusiveWrapper::Inner { .. } => unreachable!(),
        };

        {
            // SAFETY:
            // We are at the top level now, and all the child `field()`s can only contain mutable pointers to the data, so we are the only one
            let data_ptr = unsafe { &mut *s.0.data };
            let old_len = data_ptr.len();

            let data_addr = data_ptr.addr();
            let start_addr = start.addr();

            ensure!(start_addr >= data_addr);
            ensure!(start_addr <= data_addr + old_len);

            // Return early if length hasn't changed
            if amount == 0 {
                return Ok(());
            }
            let new_len = old_len + amount;

            // realloc
            unsafe { UnsizedTypeDataAccess::unsized_data_realloc(s.0.info, data_ptr, new_len) }?;

            if start_addr != data_addr + old_len {
                // SAFETY:
                // todo
                unsafe {
                    sol_memmove(
                        start.cast::<u8>().wrapping_add(amount),
                        start.cast::<u8>(),
                        old_len - (start_addr - data_addr),
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
            // SAFETY:
            // we have exclusive access to self, so no one else has a mutable reference to the data pointer.
            let data_ptr = unsafe { &mut *s.0.data };
            let old_len = data_ptr.len();

            let data_addr = data_ptr.addr();

            let start = match range.start_bound() {
                Bound::Included(start) => {
                    ensure!(*start as usize >= data_addr);
                    ensure!(*start as usize <= data_addr + old_len);
                    start.cast::<u8>()
                }
                Bound::Excluded(start) => {
                    ensure!(*start as usize >= data_addr);
                    ensure!(*start as usize <= data_addr + old_len);
                    start.cast::<u8>().wrapping_add(1)
                }
                Bound::Unbounded => data_ptr.cast(),
            };

            let end = match range.end_bound() {
                Bound::Included(end) => {
                    ensure!(*end as usize >= start as usize);
                    ensure!((*end as usize) < data_addr + old_len);
                    end.cast::<u8>().wrapping_add(1)
                }
                Bound::Excluded(end) => {
                    ensure!(*end as usize >= start as usize);
                    ensure!(*end as usize <= data_addr + old_len);
                    end.cast::<u8>()
                }
                Bound::Unbounded => data_ptr.cast::<u8>().wrapping_add(old_len),
            };

            let amount = end as usize - start as usize;
            if amount == 0 {
                return Ok(());
            }

            if end as usize != data_addr + old_len {
                unsafe {
                    sol_memmove(start, end, old_len - (end as usize - data_addr));
                }
            }

            let new_len = old_len - amount;
            // SAFETY:
            // Data ptr is derived from the info.
            unsafe {
                UnsizedTypeDataAccess::unsized_data_realloc(s.0.info, data_ptr, new_len)?;
            }

            amount
        };

        unsafe {
            Top::resize_notification(&mut s.1, source_ptr, -amount.try_into()?)?;
        }
        Ok(())
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
        mapper: impl FnOnce(*mut Mut) -> *mut O::Mut<'top>,
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
        mapper: impl FnOnce(*mut Mut) -> Result<*mut O::Mut<'top>, E>,
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
        unsafe { &mut *Self::mut_mut(self) }
    }
}

#[derive(Debug, Deref, DerefMut)]
#[repr(C)]
pub struct LengthTracker<T> {
    #[deref]
    #[deref_mut]
    pub data: T,
    start: *mut (),
    len: usize,
}

impl<T> AsShared for LengthTracker<T>
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

impl<T> LengthTracker<T> {
    /// # Safety
    /// todo
    pub unsafe fn new(data: T, start: *mut (), len: usize) -> Self {
        Self { data, start, len }
    }

    /// # Safety
    /// todo
    pub unsafe fn handle_resize_notification(
        s: &mut Self,
        source_ptr: *const (),
        change: isize,
        zst_status: bool,
    ) {
        if source_ptr < s.start {
            s.start = unsafe { s.start.byte_offset(change) };
        }
        // if we are a ZST (ZST_STATUS is false), if changes aren't happening before us, they must be within us since we are the last field.
        else if !zst_status || source_ptr < s.start.wrapping_byte_add(s.len) {
            match change.cmp(&0) {
                Ordering::Less => {
                    s.len.sub_assign(change.unsigned_abs());
                }
                Ordering::Equal => {}
                Ordering::Greater => {
                    s.len.add_assign(change.unsigned_abs());
                }
            }
        }
    }
}

impl<'top, Mut, P> ExclusiveWrapper<'_, 'top, LengthTracker<Mut>, P>
where
    Self: ExclusiveRecurse,
{
    /// # Safety
    // TODO: I think it might be safe since we're relying on the UnsizedType implementation to be correct.
    pub unsafe fn set_length_tracker_data<U, I>(wrapper: &mut Self, init_arg: I) -> Result<()>
    where
        U: UnsizedType<Mut<'top> = LengthTracker<Mut>> + UnsizedInit<I> + ?Sized,
    {
        let new_len = <U as UnsizedInit<I>>::INIT_BYTES;
        let current_len = wrapper.len;

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
                        wrapper.start..wrapper.start.wrapping_byte_add(current_len - new_len),
                    )
                }?;
            }
        }
        wrapper.data = {
            // SAFETY:
            // We have exclusive access to the wrapper, so no external references to the underlying data exist.
            // No other references exist in this function either. We can assume the StartPointer is valid since it was created by the UnsizedType implementation.
            let slice_ptr: *mut [u8] = ptr_meta::from_raw_parts_mut(wrapper.start.cast(), new_len);
            let mut slice = unsafe { &mut *slice_ptr };
            // let slice = unsafe { from_raw_parts_mut(wrapper.start.cast::<u8>(), new_len) };
            // TODO: this is probably safe
            unsafe { <U as UnsizedInit<I>>::init(&mut slice, init_arg)? };
            unsafe {
                U::get_mut(&mut slice_ptr.clone() /* None */)?.data
            }
        };
        wrapper.len = new_len;
        Ok(())
    }
}
