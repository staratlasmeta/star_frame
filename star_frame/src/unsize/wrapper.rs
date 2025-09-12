//! Safe wrappers for accessing and modifying unsized type data.
//!
//! This module provides [`SharedWrapper`] and [`ExclusiveWrapper`] types that enable safe access
//! to unsized data structures stored in account buffers. The wrappers handle lifetime management,
//! memory reallocation, and provide safe interfaces for reading and modifying dynamically-sized
//! data while maintaining Rust's safety guarantees. Almost all interactions with the
//! Unsized Type system will be through these wrappers.
use super::{AsShared, UnsizedType};
use crate::{
    account_set::single_set::SingleAccountSet,
    unsize::{init::UnsizedInit, FromOwned, UnsizedTypeMut},
    Result,
};
use eyre::ensure;
use core::ptr;
use derive_more::{Debug, Deref, DerefMut};
use pinocchio::account_info::AccountInfo;
use solana_program_memory::sol_memmove;
use std::{
    cmp::Ordering,
    collections::Bound,
    convert::Infallible,
    marker::PhantomData,
    ops::{Deref, DerefMut, RangeBounds},
};

/// A trait for types that can be used to access the underlying data of an [`UnsizedType`].
///
/// This is used to implement [`ExclusiveWrapper`] and [`SharedWrapper`].
///
/// # Safety
///
/// [`UnsizedTypeDataAccess::unsized_data_realloc`] must properly check the new length of the underlying data pointer.
pub unsafe trait UnsizedTypeDataAccess {
    /// # Safety
    /// `data` must actually point to the same data that is returned by [`UnsizedTypeDataAccess::data_ref`] and [`UnsizedTypeDataAccess::data_mut`].
    /// There must be no other live references to the buffer that data points to.
    unsafe fn unsized_data_realloc(this: &Self, data: &mut *mut [u8], new_len: usize)
        -> Result<()>;
    fn data_ref(this: &Self) -> Result<impl Deref<Target = [u8]>>;
    fn data_mut<'a>(this: &'a Self) -> Result<(*mut [u8], Box<dyn DataMutDrop + 'a>)>;
}

/// A marker trait implemented for types that [`UnsizedTypeDataAccess::data_mut`] returns so it can prevent the Ref from being dropped.
pub trait DataMutDrop {}

impl<T: ?Sized> DataMutDrop for pinocchio::account_info::RefMut<'_, T> {}

/// # Safety
/// We are checking the length of the underlying data pointer in [`Self::unsized_data_realloc`].
unsafe impl UnsizedTypeDataAccess for AccountInfo {
    #[inline]
    unsafe fn unsized_data_realloc(
        this: &Self,
        data: &mut *mut [u8],
        new_len: usize,
    ) -> Result<()> {
        // Set the data len on the account (This will check that the increase is within bounds)
        //
        // SAFETY:
        // `unsized_data_realloc` requires that no other references to this data exist, which satisfies the preconditions of this function.
        unsafe { this.resize_unchecked(new_len) }?;
        // Then recreate the local slice with the new length
        *data = ptr_meta::from_raw_parts_mut(data.cast(), new_len);
        Ok(())
    }

    #[inline]
    fn data_ref(this: &Self) -> Result<impl Deref<Target = [u8]>> {
        this.account_data()
    }

    #[inline]
    fn data_mut<'a>(this: &'a Self) -> Result<(*mut [u8], Box<dyn DataMutDrop + 'a>)> {
        let ref_mut = this.account_data_mut()?;
        let current_len = this.data_len();
        let data_ptr = this.data_ptr();
        let ptr = ptr_meta::from_raw_parts_mut(data_ptr.cast(), current_len);
        Ok((ptr, Box::new(ref_mut)))
    }
}

#[derive(derive_more::Debug)]
pub struct SharedWrapper<'top, T> {
    top_ref: T,
    #[debug(skip)]
    _to_drop: Box<dyn Deref<Target = [u8]> + 'top>,
    phantom: PhantomData<&'top ()>,
}

impl<'a, T> SharedWrapper<'a, T> {
    /// # Safety
    #[inline]
    pub fn new<U>(underlying_data: &'a impl UnsizedTypeDataAccess) -> Result<Self>
    where
        T: 'a,
        U: UnsizedType<Ref<'a> = T> + ?Sized,
    {
        // ensure no ZSTs in middle of struct
        let _ = U::ZST_STATUS;
        let data = UnsizedTypeDataAccess::data_ref(underlying_data)?;
        let data_ptr = ptr::from_ref(&*data);

        // SAFETY:
        // We are technically extending the lifetime here of the returned data, but it's okay because we keep data alive in the to_drop,
        // and the reference is never exposed. We do this to get a U::Ref<'a>, but since UnsizedType::Ref cannot be copied out from behind a reference,
        // that lifetime cannot escape through the Deref/DerefMut on SharedWrapper.
        let mut data_bytes: &'a [u8] = unsafe { &*data_ptr };
        Ok(SharedWrapper {
            _to_drop: Box::new(data),
            top_ref: U::get_ref(&mut data_bytes)?,
            phantom: PhantomData,
        })
    }
}
impl<T> Deref for SharedWrapper<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.top_ref
    }
}

/// The heart of the `UnsizedType` system. This wrapper enables resizing through the [`ExclusiveRecurse`] trait, and mapping to
/// child wrappers through [`Self::try_map_mut`]. In addition, this implements [`Deref`] and [`DerefMut`] for easy access to Mut type.
#[derive(Debug)]
pub struct ExclusiveWrapper<'parent, 'top, Mut, P>(ExclusiveWrapperEnum<'parent, 'top, Mut, P>);

/// Private enum for [`ExclusiveWrapper`].
#[derive(Debug)]
enum ExclusiveWrapperEnum<'parent, 'top, Mut, P> {
    Top {
        exclusive_top: P,
        // We Box the Mut so its derived from a separate allocation, so dereferencing parent in inners wont invalidate the top Mut.
        top_mut: Box<Mut>,
        #[debug(skip)]
        _to_drop: Box<dyn DataMutDrop + 'top>,
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

/// The generic `P` for an [`ExclusiveWrapper`] that is at the top level of the wrapper stack.
#[derive(Debug)]
pub struct ExclusiveWrapperTopMeta<'top, Top, A>
where
    Top: UnsizedType + ?Sized,
{
    info: &'top A,
    /// The pointer to the contiguous allocated slice. The len metadata may be shorter than the actual length of the allocated slice.
    /// It's lifetimes should match `&'top mut [u8]` when run in [`ExclusiveWrapperTop::new`].
    data: *mut [u8],
    /// This allows inherent implemenations on [`ExclusiveWrapperTop`].
    top_phantom: PhantomData<fn() -> Top>,
}

impl<'top, Top, A> ExclusiveWrapperTop<'top, Top, A>
where
    Top: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess,
{
    #[inline]
    pub fn new(info: &'top A) -> Result<Self> {
        // ensure no ZSTs in middle of struct
        let _ = Top::ZST_STATUS;
        let (mut data_ptr, to_drop) = UnsizedTypeDataAccess::data_mut(info)?;
        // We are technically extending the lifetime here of the returned data, but it's okay because we keep data alive in the to_drop,
        // and the reference is never exposed.
        let data = data_ptr;
        Ok(Self(ExclusiveWrapperEnum::Top {
            _to_drop: to_drop,
            top_mut: Box::new(unsafe { Top::get_mut(&mut data_ptr)? }),
            exclusive_top: ExclusiveWrapperTopMeta {
                info,
                data,
                top_phantom: PhantomData,
            },
        }))
    }
}

impl<Mut, P> ExclusiveWrapper<'_, '_, Mut, P> {
    #[inline]
    fn mut_ref(this: &Self) -> &Mut {
        match &this.0 {
            ExclusiveWrapperEnum::Top { top_mut, .. } => top_mut,
            ExclusiveWrapperEnum::Inner { field, .. } => {
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

    #[inline]
    fn mut_mut(this: &mut Self) -> *mut Mut {
        match &mut this.0 {
            ExclusiveWrapperEnum::Top { top_mut, .. } => &raw mut **top_mut,
            ExclusiveWrapperEnum::Inner { field, .. } => {
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
        start: *const (),
        amount: usize,
    ) -> Result<()>;
    /// # Safety
    /// Is this actually unsafe? If bounds are checked, everything should be fine? We have exclusive access to self right now.
    unsafe fn remove_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        range: impl RangeBounds<*const ()>,
    ) -> Result<()>;
}

impl<Mut, P> ExclusiveRecurse for ExclusiveWrapper<'_, '_, Mut, P>
where
    P: ExclusiveRecurse,
{
    #[inline]
    unsafe fn add_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        start: *const (),
        amount: usize,
    ) -> Result<()> {
        match &mut wrapper.0 {
            ExclusiveWrapperEnum::Top { .. } => unreachable!(),
            ExclusiveWrapperEnum::Inner { parent, .. } => {
                // SAFETY:
                // We have exclusive access to self right now, and no other references to parent can exist.
                let parent = unsafe { &mut **parent };
                unsafe { P::add_bytes(parent, source_ptr, start, amount) }
            }
        }
    }

    #[inline]
    unsafe fn remove_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        range: impl RangeBounds<*const ()>,
    ) -> Result<()> {
        match &mut wrapper.0 {
            ExclusiveWrapperEnum::Top { .. } => unreachable!(),
            ExclusiveWrapperEnum::Inner { parent, .. } => {
                // SAFETY:
                // We have exclusive access to self right now, so no other references to parent can exist.
                let parent = unsafe { &mut **parent };
                unsafe { P::remove_bytes(parent, source_ptr, range) }
            }
        }
    }
}

impl<Top, A> ExclusiveRecurse for ExclusiveWrapperTop<'_, Top, A>
where
    Top: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess,
{
    unsafe fn add_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        start: *const (),
        amount: usize,
    ) -> Result<()> {
        let (top_meta, top_mut) = match &mut wrapper.0 {
            ExclusiveWrapperEnum::Top {
                exclusive_top,
                top_mut,
                ..
            } => (exclusive_top, top_mut),
            ExclusiveWrapperEnum::Inner { .. } => unreachable!(),
        };

        {
            // SAFETY:
            // We are at the top level now, and all the child `field()`s can only contain mutable pointers to the data, so we are the only one
            let data_ptr = &mut top_meta.data;
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
            unsafe {
                UnsizedTypeDataAccess::unsized_data_realloc(top_meta.info, data_ptr, new_len)
            }?;

            if start_addr != data_addr + old_len {
                let dst = start as usize + amount;
                let src = start as usize;
                // SAFETY:
                // todo
                unsafe {
                    sol_memmove(
                        // Use provenance of main data ptr
                        data_ptr.with_addr(dst).cast::<u8>(),
                        data_ptr.with_addr(src).cast::<u8>(),
                        old_len - (start_addr - data_addr),
                    );
                }
            }
        }

        // TODO: Figure out the safety requirements of calling this. I think it is safe to call here assuming the UnsizedType is implemented correctly.
        unsafe {
            Top::resize_notification(top_mut, source_ptr, amount.try_into()?)?;
        }

        Ok(())
    }

    unsafe fn remove_bytes(
        wrapper: &mut Self,
        source_ptr: *const (),
        range: impl RangeBounds<*const ()>,
        // after_remove: impl FnOnce(&mut Self::T) -> Result<()>,
    ) -> Result<()> {
        let (top_meta, top_mut) = match &mut wrapper.0 {
            ExclusiveWrapperEnum::Top {
                exclusive_top,
                top_mut,
                ..
            } => (exclusive_top, top_mut),
            ExclusiveWrapperEnum::Inner { .. } => unreachable!(),
        };

        let amount = {
            // SAFETY:
            // we have exclusive access to self, so no one else has a mutable reference to the data pointer.
            let data_ptr = &mut top_meta.data;
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
                    sol_memmove(
                        data_ptr.with_addr(start as usize).cast(),
                        data_ptr.with_addr(end as usize).cast(),
                        old_len - (end as usize - data_addr),
                    );
                }
            }

            let new_len = old_len - amount;
            // SAFETY:
            // Data ptr is derived from the info.
            unsafe {
                UnsizedTypeDataAccess::unsized_data_realloc(top_meta.info, data_ptr, new_len)?;
            }

            amount
        };

        unsafe {
            Top::resize_notification(top_mut, source_ptr, -amount.try_into()?)?;
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
    #[inline]
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
    #[inline]
    pub unsafe fn try_map_mut<'child, O, E>(
        parent: &'child mut Self,
        mapper: impl FnOnce(*mut Mut) -> Result<*mut O::Mut<'top>, E>,
    ) -> Result<ExclusiveWrapper<'child, 'top, O::Mut<'top>, Self>, E>
    where
        O: UnsizedType + ?Sized,
    {
        let parent_mut: *mut Self = parent;
        Ok(ExclusiveWrapper(ExclusiveWrapperEnum::Inner {
            parent_lt: PhantomData,
            parent: parent_mut,
            field: mapper(Self::mut_mut(parent))?,
        }))
    }
}

impl<Mut, P> Deref for ExclusiveWrapper<'_, '_, Mut, P>
where
    Self: ExclusiveRecurse,
{
    type Target = Mut;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Self::mut_ref(self)
    }
}

impl<Mut, P> DerefMut for ExclusiveWrapper<'_, '_, Mut, P>
where
    Self: ExclusiveRecurse,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *Self::mut_mut(self) }
    }
}

#[derive(Debug, Deref, DerefMut)]
#[repr(C)]
pub struct StartPointer<T> {
    #[deref]
    #[deref_mut]
    pub data: T,
    start: *mut (),
}

impl<T> AsShared for StartPointer<T>
where
    T: AsShared,
{
    type Ref<'b>
        = T::Ref<'b>
    where
        Self: 'b;

    #[inline]
    fn as_shared(&self) -> Self::Ref<'_> {
        self.data.as_shared()
    }
}

unsafe impl<T> UnsizedTypeMut for StartPointer<T>
where
    T: UnsizedTypeMut,
{
    type UnsizedType = T::UnsizedType;
}

impl<T> StartPointer<T> {
    #[inline]
    pub fn start_ptr(this: &Self) -> *mut () {
        this.start
    }

    /// # Safety
    /// todo
    #[inline]
    pub unsafe fn new(data: T, start: *mut ()) -> Self {
        Self { data, start }
    }

    /// # Safety
    /// todo
    #[inline]
    pub unsafe fn handle_resize_notification(s: &mut Self, source_ptr: *const (), change: isize) {
        if source_ptr < s.start {
            s.start = s.start.wrapping_byte_offset(change);
        }
    }
}

impl<'top, Mut, P, U> ExclusiveWrapper<'_, 'top, Mut, P>
where
    Self: ExclusiveRecurse,
    U: ?Sized + UnsizedType<Mut<'top> = Mut>,
    Mut: UnsizedTypeMut<UnsizedType = U>,
{
    #[inline]
    fn set_data_inner<I>(
        &mut self,
        arg: I,
        new_len: usize,
        initialize: impl FnOnce(&mut &mut [u8], I) -> Result<()>,
    ) -> Result<()> {
        let current_len = <U as UnsizedType>::data_len(self);
        let start_ptr = <U as UnsizedType>::start_ptr(self);

        match current_len.cmp(&new_len) {
            Ordering::Less => {
                // TODO: might be safe
                unsafe { Self::add_bytes(self, start_ptr, start_ptr, new_len - current_len) }?;
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                // TODO: might be safe
                unsafe {
                    Self::remove_bytes(
                        self,
                        start_ptr,
                        start_ptr.cast_const()..start_ptr.wrapping_byte_add(current_len - new_len),
                    )
                }?;
            }
        }
        **self = {
            // SAFETY:
            // We have exclusive access to the wrapper, so no external references to the underlying data exist.
            // No other references exist in this function either. We can assume the StartPointer is valid since it was created by the UnsizedType implementation.
            let mut slice_ptr: *mut [u8] = ptr_meta::from_raw_parts_mut(start_ptr, new_len);
            {
                // SAFETY:
                // we just made the slice pointer. No one else has access to the data, so we can dereference it as we please.
                let mut slice = unsafe { &mut *slice_ptr };
                initialize(&mut slice, arg)?;
            }
            // SAFETY:
            // The underlying data is valid for 'top, and we just created the slice so it's length is valid.
            // We just resizd the underlying data to be enough for `len`
            let res: U::Mut<'top> = unsafe { U::get_mut(&mut slice_ptr)? };
            debug_assert_eq!(<U as UnsizedType>::data_len(&res), new_len);
            debug_assert_eq!(<U as UnsizedType>::start_ptr(&res), start_ptr);
            res
        };
        Ok(())
    }

    pub fn set_from_init<I>(&mut self, init_arg: I) -> Result<()>
    where
        U: UnsizedInit<I>,
    {
        Self::set_data_inner(
            self,
            init_arg,
            <U as UnsizedInit<I>>::INIT_BYTES,
            |slice, arg| <U as UnsizedInit<I>>::init(slice, arg),
        )
    }

    pub fn set_from_owned(&mut self, owned: U::Owned) -> Result<()>
    where
        U: FromOwned,
    {
        let new_len = U::byte_size(&owned);
        Self::set_data_inner(self, owned, new_len, |slice, owned| {
            U::from_owned(owned, slice).map(|_| ())
        })
    }
}
