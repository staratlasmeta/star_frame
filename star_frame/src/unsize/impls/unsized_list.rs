use crate::{
    align1::Align1,
    data_types::PackedValue,
    unsize::{
        init::{DefaultInit, UnsizedInit},
        unsized_impl,
        wrapper::{ExclusiveRecurse, ExclusiveWrapper},
        AsShared, FromOwned, RawSliceAdvance, UnsizedType, UnsizedTypeMut,
    },
    Result,
};
use advancer::{Advance, AdvanceArray};
use anyhow::{bail, ensure, Context};
use bytemuck::{bytes_of, cast_slice_mut, Pod, Zeroable};
use core::slice;
use itertools::Itertools;
use num_traits::ToPrimitive;
use ptr_meta::Pointee;
use solana_program_memory::sol_memmove;
use std::{
    borrow::Borrow,
    cmp::Ordering,
    iter,
    iter::FusedIterator,
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
    ops::{Deref, DerefMut, RangeBounds},
};

type PackedU32 = PackedValue<u32>;

unsafe impl UnsizedListOffset for PackedValue<u32> {
    type ListOffsetInit = ();
    #[inline]
    fn to_usize_offset(&self) -> usize {
        self.0 as usize
    }
    #[inline]
    fn as_mut_offset(&mut self) -> &mut PackedU32 {
        self
    }
    #[inline]
    fn as_offset(&self) -> &PackedU32 {
        self
    }

    #[inline]
    fn from_usize_offset(offset: usize, _init: Self::ListOffsetInit) -> Result<Self> {
        Ok(PackedValue(offset.try_into()?))
    }
}

impl PackedU32 {
    #[inline]
    #[must_use]
    pub fn usize(self) -> usize {
        self.0 as usize
    }
}
pub(super) const U32_SIZE: usize = size_of::<u32>();
/// # Safety
/// The offset provided in [`Self::from_usize_offset`] must be the same value returned by the getter methods.
pub unsafe trait UnsizedListOffset: Pod + Align1 {
    type ListOffsetInit;
    fn to_usize_offset(&self) -> usize;
    // TODO: this locks the offset type into a packed u32. Potentially consider making this more generic
    fn as_mut_offset(&mut self) -> &mut PackedU32;
    fn as_offset(&self) -> &PackedU32;
    fn from_usize_offset(offset: usize, init: Self::ListOffsetInit) -> Result<Self>;
}

#[derive(Debug, Align1, Pointee)]
#[repr(C)]
pub struct UnsizedList<T, C = PackedU32>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    phantom_t: PhantomData<fn() -> T>,
    pub(super) unsized_size: PackedU32,
    len: PackedU32,
    pub(super) offset_list: [C],
    // copy of len
    // bytes of unsized data
}

impl<T, C> UnsizedList<T, C>
where
    T: UnsizedType + FromOwned + ?Sized,
    C: UnsizedListOffset,
{
    pub(super) fn from_owned_byte_size<I>(items: I) -> usize
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Borrow<T::Owned>,
    {
        let items = items.into_iter();
        let len = items.len();
        U32_SIZE + // unsized size
            U32_SIZE + // len
        len * size_of::<C>() + // offset list
        U32_SIZE + // copy of len
        items.zip_eq(0..len).fold(0, |acc, (item,_)| acc + <T as FromOwned>::byte_size(item.borrow()))
    }

    pub(super) fn from_owned_from_iter<I>(items: I, bytes: &mut &mut [u8]) -> Result<usize>
    where
        I: IntoIterator<Item = (T::Owned, C::ListOffsetInit)>,
        I::IntoIter: ExactSizeIterator,
    {
        let owned = items.into_iter();
        let owned_len = owned.len();
        let owned_len_bytes = u32::try_from(owned_len)?.to_le_bytes();
        let unsized_size_bytes = bytes.try_advance_array::<U32_SIZE>().with_context(|| {
            format!(
                "Failed to read unsized size bytes for {} FromOwned",
                std::any::type_name::<Self>()
            )
        })?;

        let len_bytes = bytes.try_advance_array::<U32_SIZE>().with_context(|| {
            format!(
                "Failed to read len bytes for {} FromOwned",
                std::any::type_name::<Self>()
            )
        })?;
        *len_bytes = owned_len_bytes;

        let offset_list_bytes =
            bytes
                .try_advance(owned_len * size_of::<C>())
                .with_context(|| {
                    format!(
                        "Failed to read offset list bytes for {} FromOwned",
                        std::any::type_name::<Self>()
                    )
                })?;
        let offset_array = bytemuck::try_cast_slice_mut::<_, C>(offset_list_bytes)?;

        let copy_of_len_bytes = bytes.try_advance_array::<U32_SIZE>().with_context(|| {
            format!(
                "Failed to read copy of len bytes for {} FromOwned",
                std::any::type_name::<Self>()
            )
        })?;
        *copy_of_len_bytes = owned_len_bytes;

        let mut unsized_bytes_written = 0;
        owned
            .zip_eq(offset_array.iter_mut())
            .try_for_each(|((item, init), offset_item)| {
                *offset_item = C::from_usize_offset(unsized_bytes_written, init)?;
                unsized_bytes_written += T::from_owned(item, bytes)?;
                anyhow::Ok(())
            })?;

        *unsized_size_bytes = u32::try_from(unsized_bytes_written)?.to_le_bytes();

        Ok(U32_SIZE * 3 + owned_len * size_of::<C>() + unsized_bytes_written)
    }
}

unsafe impl<T, C> FromOwned for UnsizedList<T, C>
where
    T: UnsizedType + FromOwned + ?Sized,
    C: UnsizedListOffset<ListOffsetInit = ()>,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        Self::from_owned_byte_size(owned.iter())
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        Self::from_owned_from_iter(owned.into_iter().map(|item| (item, ())), bytes)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::{idl::TypeToIdl, prelude::System};
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

    impl<T, C> TypeToIdl for UnsizedList<T, C>
    where
        T: UnsizedType + ?Sized + TypeToIdl,
        C: UnsizedListOffset + TypeToIdl,
    {
        type AssociatedProgram = System;
        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::UnsizedList {
                item_ty: T::type_to_idl(idl_definition)?.into(),
                offset_ty: C::type_to_idl(idl_definition)?.into(),
                len_ty: IdlTypeDef::U32.into(),
            })
        }
    }
}

impl<T, C> UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    #[inline]
    pub fn len(&self) -> usize {
        self.offset_list.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.offset_list.is_empty()
    }

    /// Gets a pointer to the start of the unsized data, but with not great provenance.
    #[inline]
    fn unsized_data_ptr(&self) -> *const u8 {
        self.offset_list
            .as_ptr()
            .wrapping_add(self.len())
            .wrapping_byte_add(U32_SIZE)
            .cast()
    }

    #[inline]
    fn unsized_data_ptr_mut(&mut self) -> *mut u8 {
        self.offset_list
            .as_mut_ptr()
            .wrapping_add(self.len())
            .wrapping_byte_add(U32_SIZE)
            .cast()
    }

    #[inline]
    fn unsized_bytes(&self) -> &[u8] {
        // SAFETY:
        // we have shared access to self, and we were the ones that allocated the underlying data. We
        // are properly keeping track of the len of the unsized bytes we control via unsized_size.
        unsafe { slice::from_raw_parts(self.unsized_data_ptr(), self.unsized_size.usize()) }
    }

    #[inline]
    fn unsized_bytes_mut(&mut self) -> &mut [u8] {
        // SAFETY:
        // we have exclusive access to self, and we were the ones that allocated the underlying data. We
        // are properly keeping track of the len of the unsized bytes we control via unsized_size.
        unsafe { slice::from_raw_parts_mut(self.unsized_data_ptr_mut(), self.unsized_size.usize()) }
    }

    fn unsized_list_len(&mut self) -> &mut PackedU32 {
        let unsized_len_ptr = self
            .unsized_data_ptr_mut()
            .wrapping_byte_sub(U32_SIZE)
            .cast::<PackedU32>();
        unsafe { &mut *unsized_len_ptr }
    }

    fn adjust_offsets(&mut self, start_index: usize, change: isize) -> Result<()> {
        debug_assert!(
            self.offset_list[start_index..].is_sorted_by(|a, b| { a.as_offset() < b.as_offset() })
        );
        if self.offset_list.is_empty() {
            return Ok(());
        }

        match change.cmp(&0) {
            Ordering::Less => {
                if let Some((first, rest)) = self.offset_list[start_index..].split_first_mut() {
                    let change: u32 = (-change).try_into()?;
                    // First item to change should be smallest, so this makes sure none of the offsets underflow
                    first.as_mut_offset().0 = first
                        .as_mut_offset()
                        .0
                        .checked_sub(change)
                        .context("Failed to decrease bytes to first offset")?;

                    for item in rest.iter_mut() {
                        item.as_mut_offset().0 =
                            unsafe { item.as_mut_offset().0.unchecked_sub(change) }
                    }
                }
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                if let Some((last, rest)) = self.offset_list[start_index..].split_last_mut() {
                    let change: u32 = change.try_into()?;
                    // Last item should be largest, so this makes sure none of the offsets overflow
                    last.as_mut_offset().0 = last
                        .as_mut_offset()
                        .0
                        .checked_add(change)
                        .context("Failed to increase bytes to last offset")?;

                    for item in rest.iter_mut() {
                        item.as_mut_offset().0 =
                            unsafe { item.as_mut_offset().0.unchecked_add(change) }
                    }
                }
            }
        }

        Ok(())
    }

    fn adjust_offsets_from_ptr(&mut self, source_ptr: *const (), change: isize) -> Result<()> {
        if self.offset_list.is_empty() {
            return Ok(());
        }
        let adjusted_source = source_ptr as usize - self.unsized_data_ptr() as usize;
        let start_index = match self
            .offset_list
            .binary_search_by(|offset| offset.to_usize_offset().cmp(&adjusted_source))
        {
            Ok(index) => index + 1,
            Err(index) => index,
        };
        self.adjust_offsets(start_index, change)
    }

    #[must_use]
    #[inline]
    pub fn total_byte_size(&self) -> usize {
        size_of_val(self) + U32_SIZE + self.unsized_size.usize()
    }

    #[inline]
    fn get_offset(&self, index: usize) -> usize {
        self.offset_list.get(index).map_or_else(
            || self.unsized_size.usize(),
            UnsizedListOffset::to_usize_offset,
        )
    }

    pub(super) fn get_unsized_range(&self, index: usize) -> Option<(usize, usize)> {
        let start_index = self.offset_list.get(index)?;
        let start_bound = start_index.to_usize_offset();
        let end_bound = self.offset_list.get(index + 1).map_or(
            self.unsized_size.usize(),
            UnsizedListOffset::to_usize_offset,
        );
        Some((start_bound, end_bound))
    }

    pub fn get(&self, index: usize) -> Result<Option<T::Ref<'_>>> {
        let Some((start, end)) = self.get_unsized_range(index) else {
            return Ok(None);
        };
        let unsized_bytes = self.unsized_bytes();
        T::get_ref(&mut &unsized_bytes[start..end]).map(Some)
    }

    pub fn get_mut(&mut self, index: usize) -> Result<Option<T::Mut<'_>>> {
        let Some((start, _end)) = self.get_unsized_range(index) else {
            return Ok(None);
        };
        let unsized_data_bytes = self.unsized_bytes_mut();
        let mut unsized_data_ptr = core::ptr::from_mut(unsized_data_bytes);
        unsized_data_ptr.try_advance(start)?;
        unsafe { T::get_mut(&mut unsized_data_ptr).map(Some) }
    }

    #[inline]
    pub fn first(&self) -> Result<Option<T::Ref<'_>>> {
        self.get(0)
    }

    #[inline]
    pub fn first_mut(&mut self) -> Result<Option<T::Mut<'_>>> {
        self.get_mut(0)
    }

    #[inline]
    pub fn last(&self) -> Result<Option<T::Ref<'_>>> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.get(self.len() - 1)
        }
    }

    #[inline]
    pub fn last_mut(&mut self) -> Result<Option<T::Mut<'_>>> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.get_mut(self.len() - 1)
        }
    }

    #[inline]
    pub fn index(&self, index: usize) -> Result<T::Ref<'_>> {
        self.get(index).transpose().context("Index out of bounds")?
    }

    #[inline]
    pub fn index_mut(&mut self, index: usize) -> Result<T::Mut<'_>> {
        self.get_mut(index)
            .transpose()
            .context("Index out of bounds")?
    }

    pub(super) fn iter_with_offsets(&self) -> UnsizedListWithOffsetIter<'_, T, C> {
        UnsizedListWithOffsetIter {
            list: self,
            index: 0,
        }
    }

    pub(super) fn iter_with_offsets_mut(&mut self) -> UnsizedListWithOffsetIterMut<'_, T, C> {
        UnsizedListWithOffsetIterMut {
            list: self,
            index: 0,
        }
    }

    #[inline]
    pub fn iter(&self) -> UnsizedListIter<'_, T, C> {
        UnsizedListIter {
            iter: self.iter_with_offsets(),
        }
    }
    #[inline]
    pub fn iter_mut(&mut self) -> UnsizedListIterMut<'_, T, C> {
        UnsizedListIterMut {
            iter: self.iter_with_offsets_mut(),
        }
    }
}

#[derive(derive_where::DeriveWhere)]
#[derive_where(Debug)]
pub struct UnsizedListRef<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    list_ptr: *const UnsizedList<T, C>,
    phantom: PhantomData<&'a ()>,
}

impl<T, C> Deref for UnsizedListRef<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Target = UnsizedList<T, C>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.list_ptr }
    }
}

#[derive(Debug)]
pub struct UnsizedListMut<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    list_ptr: *mut UnsizedList<T, C>,
    // We use MaybeUninit here to allow us to access the pointer directly without reborrowing (TODO: Fully understand why this is neccesary lol)
    inner_exclusive: Box<MaybeUninit<T::Mut<'a>>>,
    inner_initialized: bool,
    phantom: PhantomData<&'a ()>,
}

impl<T, C> UnsizedListMut<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    #[inline]
    fn unsized_bytes_mut_ptr(&mut self) -> *mut [u8] {
        ptr_meta::from_raw_parts_mut(
            self.list_ptr
                .with_addr(self.unsized_data_ptr_mut() as usize)
                .cast(),
            self.unsized_size.usize(),
        )
    }
}

impl<T, C> Drop for UnsizedListMut<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    fn drop(&mut self) {
        if self.inner_initialized {
            // SAFETY:
            // we only set inner_initialized when it is truly initialized
            unsafe {
                self.inner_exclusive.assume_init_drop();
            }
        }
    }
}

impl<T, C> Deref for UnsizedListMut<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Target = UnsizedList<T, C>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.list_ptr }
    }
}

impl<T, C> DerefMut for UnsizedListMut<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.list_ptr }
    }
}

impl<T, C> AsShared for UnsizedListRef<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Ref<'b>
        = UnsizedListRef<'b, T, C>
    where
        Self: 'b;
    fn as_shared(&self) -> Self::Ref<'_> {
        UnsizedList::ref_as_ref(self)
    }
}
impl<T, C> AsShared for UnsizedListMut<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Ref<'b>
        = UnsizedListRef<'b, T, C>
    where
        Self: 'b;
    fn as_shared(&self) -> Self::Ref<'_> {
        UnsizedList::mut_as_ref(self)
    }
}

unsafe impl<T, C> UnsizedTypeMut for UnsizedListMut<'_, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type UnsizedType = UnsizedList<T, C>;
}

unsafe impl<T, C> UnsizedType for UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Ref<'a> = UnsizedListRef<'a, T, C>;
    type Mut<'a> = UnsizedListMut<'a, T, C>;
    type Owned = Vec<T::Owned>;
    const ZST_STATUS: bool = {
        assert!(
            T::ZST_STATUS,
            "T cannot be a zero sized type in UnsizedList<T>"
        );
        true
    };

    fn ref_as_ref<'a>(r: &'a Self::Ref<'_>) -> Self::Ref<'a> {
        UnsizedListRef {
            list_ptr: r.list_ptr,
            phantom: PhantomData,
        }
    }

    fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a> {
        UnsizedListRef {
            list_ptr: m.list_ptr,
            phantom: PhantomData,
        }
    }

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        let ptr = data.as_ptr();
        let unsized_size_fail = || {
            format!(
                "Failed to read unsized size bytes for {}",
                std::any::type_name::<Self>()
            )
        };
        let length_bytes_fail = || {
            format!(
                "Failed to read length bytes for {}",
                std::any::type_name::<Self>()
            )
        };
        let offset_list_fail = |length| {
            format!(
                "Failed to read offset list of length {} for {}",
                length,
                std::any::type_name::<Self>()
            )
        };
        let length_copy_fail = || {
            format!(
                "Failed to read length copy for {}",
                std::any::type_name::<Self>()
            )
        };
        let unsized_data_fail = |size| {
            format!(
                "Failed to read unsized data of size {} for {}",
                size,
                std::any::type_name::<Self>()
            )
        };

        let unsized_size_bytes = data
            .try_advance_array::<U32_SIZE>()
            .with_context(unsized_size_fail)?;
        let unsized_size = u32::from_le_bytes(*unsized_size_bytes) as usize;

        let length_bytes = data
            .try_advance_array::<U32_SIZE>()
            .with_context(length_bytes_fail)?;
        let length = u32::from_le_bytes(*length_bytes) as usize;

        let _offset_list = data
            .try_advance(length * size_of::<C>())
            .with_context(|| offset_list_fail(length))?;

        let _length_copy = data.try_advance(U32_SIZE).with_context(length_copy_fail)?;

        let _unsized_data = data
            .try_advance(unsized_size)
            .with_context(|| unsized_data_fail(unsized_size))?;

        Ok(UnsizedListRef {
            list_ptr: &raw const *ptr_meta::from_raw_parts(ptr.cast::<()>(), length),
            phantom: PhantomData,
        })
    }

    unsafe fn get_mut<'a>(data: &mut *mut [u8]) -> Result<Self::Mut<'a>> {
        let unsized_size_fail = || {
            format!(
                "Failed to read unsized size bytes for {}",
                std::any::type_name::<Self>()
            )
        };
        let length_fail = || {
            format!(
                "Failed to read length bytes for {}",
                std::any::type_name::<Self>()
            )
        };
        let offset_list_fail = |length| {
            format!(
                "Failed to read offset list of length {} for {}",
                length,
                std::any::type_name::<Self>()
            )
        };
        let length_copy_fail = || {
            format!(
                "Failed to read length copy for {}",
                std::any::type_name::<Self>()
            )
        };
        let unsized_data_fail = |size| {
            format!(
                "Failed to read unsized data of size {} for {}",
                size,
                std::any::type_name::<Self>()
            )
        };

        let ptr = *data;

        let unsized_size_ptr = data.try_advance(U32_SIZE).with_context(unsized_size_fail)?;
        // SAFETY:
        // The advanced ptr must be the proper length per get_mut's safety contract, and it is safe to read.
        let unsized_bytes = unsafe { unsized_size_ptr.cast::<[u8; U32_SIZE]>().read() };
        let unsized_size: usize = u32::from_le_bytes(unsized_bytes) as usize;

        let length_bytes_ptr = data.try_advance(U32_SIZE).with_context(length_fail)?;
        // SAFETY:
        // The advanced ptr must be the proper length per get_mut's safety contract, and it is safe to read.
        let length_bytes = unsafe { length_bytes_ptr.cast::<[u8; U32_SIZE]>().read() };
        let length: usize = u32::from_le_bytes(length_bytes) as usize;

        let _offset_list = data
            .try_advance(length * size_of::<C>())
            .with_context(|| offset_list_fail(length))?;
        let _length_copy = data.try_advance(U32_SIZE).with_context(length_copy_fail)?;
        let _unsized_data = data
            .try_advance(unsized_size)
            .with_context(|| unsized_data_fail(unsized_size))?;

        Ok(UnsizedListMut {
            list_ptr: ptr_meta::from_raw_parts_mut(ptr.cast::<()>(), length),
            inner_exclusive: Box::new_uninit(),
            inner_initialized: false,
            phantom: PhantomData,
        })
    }

    #[inline]
    fn data_len(m: &Self::Mut<'_>) -> usize {
        m.total_byte_size()
    }

    #[inline]
    fn start_ptr(m: &Self::Mut<'_>) -> *mut () {
        m.list_ptr.cast::<()>()
    }

    fn owned_from_ref(r: &Self::Ref<'_>) -> Result<Self::Owned> {
        let mut owned = Vec::with_capacity(r.len());
        let unsized_bytes = r.unsized_bytes();
        for offset in &r.offset_list {
            let t_ref = T::get_ref(&mut &unsized_bytes[offset.to_usize_offset()..])?;
            owned.push(T::owned_from_ref(&t_ref)?);
        }
        Ok(owned)
    }

    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()> {
        let self_ptr = self_mut.list_ptr;
        if source_ptr < self_ptr.cast_const().cast() {
            // the change happened before me
            self_mut.list_ptr = self_ptr.wrapping_byte_offset(change);
        } else if source_ptr == self_ptr.cast_const().cast() {
            // updating offset list should be handled by UnsizedList directly. Do nothing!
        } else if source_ptr
            < self_ptr
                .cast_const()
                .cast::<()>()
                .wrapping_byte_add(self_mut.total_byte_size())
        {
            // An element in me is changing its size!!
            if self_mut.inner_initialized {
                unsafe {
                    T::resize_notification(
                        self_mut.inner_exclusive.assume_init_mut(),
                        source_ptr,
                        change,
                    )?;
                }
            } else {
                bail!("An element UnsizedList is changing its size, but the inner Mut is not initialized. This should never happen");
            }
            let new_unsized_len: isize = self_mut
                .unsized_size
                .to_isize()
                .context("Failed to convert unsized_size to isize. This should never happen")?
                + change;
            self_mut.unsized_size = new_unsized_len
                .to_u32()
                .context(
                    "Failed to convert updated unsized_size size to u32. This should never happen",
                )?
                .into();
            self_mut.adjust_offsets_from_ptr(source_ptr, change)?;
        } else {
            // The change happened after me. Do nothing!
        }
        Ok(())
    }
}

pub(crate) fn unsized_list_exclusive_fn<'top, T, C>(
    list_ptr: *mut UnsizedListMut<'top, T, C>,
    start: usize,
) -> Result<*mut T::Mut<'top>>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    let data = unsafe { &mut *list_ptr };
    let mut bytes: *mut [u8] = data.unsized_bytes_mut_ptr();
    bytes.try_advance(start)?;
    let t: T::Mut<'top> = unsafe { T::get_mut(&mut bytes)? };
    *data.inner_exclusive = MaybeUninit::new(t);
    data.inner_initialized = true;

    let inner_exclusive: *mut MaybeUninit<_> = unsafe { &raw mut *(*list_ptr).inner_exclusive };
    Ok(inner_exclusive.cast())
}

#[unsized_impl]
impl<T, C> UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    #[exclusive]
    pub fn get_exclusive<'child>(
        &'child mut self,
        index: usize,
    ) -> Result<Option<ExclusiveWrapper<'child, 'top, T::Mut<'top>, Self>>> {
        let Some((start, _end)) = self.get_unsized_range(index) else {
            return Ok(None);
        };
        unsafe {
            ExclusiveWrapper::try_map_mut::<T, _>(self, |data_ptr| {
                unsized_list_exclusive_fn(data_ptr, start)
            })
        }
        .map(Some)
    }

    #[exclusive]
    #[inline]
    pub fn index_exclusive<'child>(
        &'child mut self,
        index: usize,
    ) -> Result<ExclusiveWrapper<'child, 'top, T::Mut<'top>, Self>> {
        self.get_exclusive(index)
            .transpose()
            .context("Index out of bounds")?
    }

    #[exclusive]
    #[inline]
    pub fn first_exclusive<'child>(
        &'child mut self,
    ) -> Result<Option<ExclusiveWrapper<'child, 'top, T::Mut<'top>, Self>>> {
        self.get_exclusive(0)
    }

    #[exclusive]
    #[inline]
    pub fn last_exclusive<'child>(
        &'child mut self,
    ) -> Result<Option<ExclusiveWrapper<'child, 'top, T::Mut<'top>, Self>>> {
        if self.is_empty() {
            Ok(None)
        } else {
            self.get_exclusive(self.len() - 1)
        }
    }

    #[exclusive]
    pub fn push<Init>(&mut self, item: Init) -> Result<()>
    where
        T: UnsizedInit<Init>,
        C: UnsizedListOffset<ListOffsetInit = ()>,
    {
        self.push_with_offset(item, ())
    }

    #[exclusive]
    pub fn push_with_offset<Init, CI>(&mut self, item: Init, offset_item: CI) -> Result<()>
    where
        T: UnsizedInit<Init>,
        C: UnsizedListOffset<ListOffsetInit = CI>,
    {
        self.insert_with_offset(self.len(), item, offset_item)
    }

    #[exclusive]
    pub fn push_all<I, Init>(&mut self, items: I) -> Result<()>
    where
        T: UnsizedInit<Init>,
        C: UnsizedListOffset<ListOffsetInit = ()>,
        I: IntoIterator<Item = Init>,
        I::IntoIter: ExactSizeIterator,
    {
        self.insert_all(self.len(), items)
    }

    #[exclusive]
    pub fn push_all_with_offsets<I, Init, CInit>(&mut self, items: I) -> Result<()>
    where
        T: UnsizedInit<Init>,
        C: UnsizedListOffset<ListOffsetInit = CInit>,
        I: Iterator<Item = (Init, CInit)> + ExactSizeIterator,
    {
        self.insert_all_with_offsets(self.len(), items)
    }

    #[exclusive]
    pub fn insert<I>(&mut self, index: usize, item: I) -> Result<()>
    where
        T: UnsizedInit<I>,
        C: UnsizedListOffset<ListOffsetInit = ()>,
    {
        self.insert_all(index, iter::once(item))
    }

    #[exclusive]
    pub fn insert_with_offset<Init, CInit>(
        &mut self,
        index: usize,
        item: Init,
        offset: CInit,
    ) -> Result<()>
    where
        T: UnsizedInit<Init>,
        C: UnsizedListOffset<ListOffsetInit = CInit>,
    {
        self.insert_all_with_offsets(index, iter::once((item, offset)))
    }

    #[exclusive]
    pub fn insert_all<I, Init>(&mut self, index: usize, items: I) -> Result<()>
    where
        T: UnsizedInit<Init>,
        I: IntoIterator<Item = Init>,
        C: UnsizedListOffset<ListOffsetInit = ()>,
        I::IntoIter: ExactSizeIterator,
    {
        self.insert_all_with_offsets(index, items.into_iter().map(|i| (i, ())))
    }

    #[exclusive]
    pub fn insert_all_with_offsets<I, Init, CInit>(&mut self, index: usize, items: I) -> Result<()>
    where
        T: UnsizedInit<Init>,
        I: Iterator<Item = (Init, CInit)> + ExactSizeIterator,
        C: UnsizedListOffset<ListOffsetInit = CInit>,
    {
        let (source_ptr, add_bytes_start, insertion_offset) = {
            if index > self.len() {
                bail!("Index out of bounds");
            }
            let offset = self.get_offset(index);
            let start_ptr = self.unsized_data_ptr_mut().wrapping_byte_add(offset);
            (
                self.list_ptr.cast_const().cast::<()>(),
                start_ptr.cast::<()>(),
                offset,
            )
        };

        let to_add = items.len();
        let add_amount = (T::INIT_BYTES + size_of::<C>()) * to_add;

        unsafe { ExclusiveRecurse::add_bytes(self, source_ptr, add_bytes_start, add_amount)? };
        {
            let list = &mut **self;
            {
                list.list_ptr =
                    ptr_meta::from_raw_parts_mut(list.list_ptr.cast::<()>(), list.len() + to_add);
            }
            {
                // We have added bytes at the unsized list insertion index. We now need
                // to shift all the bytes from the index in the offset list up to immediately before we
                // inserted the new bytes by the size of an offset list element to fit the new offset in
                let offset_list_ptr = list.offset_list.as_mut_ptr();
                let new_offset_start = offset_list_ptr.wrapping_add(index);
                let dst_ptr = new_offset_start.wrapping_add(to_add); // shift down by size of offset counter element
                unsafe {
                    sol_memmove(
                        dst_ptr.cast(),
                        new_offset_start.cast(),
                        add_bytes_start as usize - new_offset_start as usize,
                    );
                }
            }
            {
                let new_len_u32: u32 = list.len().try_into()?;
                list.len.0 = new_len_u32;
                list.unsized_list_len().0 = new_len_u32;

                let size_increase = to_add * T::INIT_BYTES;
                list.unsized_size.0 += u32::try_from(size_increase)?;
                list.adjust_offsets(index + to_add, size_increase.try_into()?)?;
            }
            {
                let mut new_data = unsafe {
                    slice::from_raw_parts_mut(
                        list.unsized_data_ptr_mut()
                            .wrapping_byte_add(insertion_offset),
                        T::INIT_BYTES * to_add,
                    )
                };

                // zip_eq to ensure ExactSizeIterator is telling the truth
                for ((item_index, (item_init, offset_init)), _) in
                    items.enumerate().zip_eq(0..to_add)
                {
                    T::init(&mut new_data, item_init)?;
                    let new_offset = insertion_offset + item_index * T::INIT_BYTES;
                    list.offset_list[index + item_index] =
                        C::from_usize_offset(new_offset, offset_init)?;
                }
            }
        }
        Ok(())
    }

    #[exclusive]
    pub fn pop(&mut self) -> Result<Option<()>> {
        if self.is_empty() {
            return Ok(None);
        }
        Some(self.remove(self.len() - 1)).transpose()
    }

    #[exclusive]
    pub fn remove(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..=index)
    }

    #[exclusive]
    pub fn remove_range(&mut self, indices: impl RangeBounds<usize>) -> Result<()> {
        let start = match indices.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match indices.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => self.len(),
        };

        if start == 0 && end == self.len() {
            return self.clear();
        }

        ensure!(start <= end);
        ensure!(end <= self.len());

        let (start_offset, offset_of_start_ptr, end_offset) = {
            let start_offset = self.get_offset(start);
            let offset_of_start_ptr = self.unsized_data_ptr_mut().wrapping_byte_add(start_offset);
            let end_offset = self.get_offset(end);
            (start_offset, offset_of_start_ptr, end_offset)
        };
        let to_remove = end - start;
        let unsized_bytes_removed = end_offset - start_offset;

        {
            let offset_list_ptr = &mut self.offset_list.as_mut_ptr();
            let start_offset_list = offset_list_ptr.wrapping_add(start).cast::<u8>(); // dst ptr
            let end_offset_list = offset_list_ptr.wrapping_add(end).cast::<u8>(); // src ptr
            let shift_amount = offset_of_start_ptr as usize - end_offset_list as usize;
            unsafe {
                // shift everything until the removed elements up to get rid of removed offsets
                sol_memmove(
                    start_offset_list.cast(),
                    end_offset_list.cast(),
                    // provenance.with_addr(start_offset_list as usize).cast(),
                    // provenance.with_addr(end_offset_list as usize).cast(),
                    shift_amount,
                );
            }
        }

        {
            // we shifted all the unsized bytes to be removed up by the offset chunk to remove, so the start pointer of
            // bytes to remove has to be shifted too
            let remove_start = offset_of_start_ptr
                .wrapping_byte_sub(size_of::<C>() * to_remove)
                .cast::<()>();
            let remove_end = self
                .unsized_data_ptr()
                .wrapping_byte_add(end_offset)
                .cast::<()>();
            let source_ptr = self.list_ptr.cast_const().cast::<()>();
            unsafe {
                ExclusiveRecurse::remove_bytes(
                    self,
                    source_ptr,
                    remove_start.cast_const()..remove_end,
                )?;
            }
            {
                let list = &mut **self;
                {
                    list.list_ptr = ptr_meta::from_raw_parts_mut(
                        list.list_ptr.cast::<()>(),
                        list.len() - to_remove,
                    );
                }
                {
                    let new_len_u32 = list.len().try_into()?;
                    list.len.0 = new_len_u32;
                    list.unsized_list_len().0 = new_len_u32;
                    list.unsized_size.0 -= u32::try_from(unsized_bytes_removed)?;
                    list.adjust_offsets(start, -isize::try_from(unsized_bytes_removed)?)?;
                }
            }
        }

        Ok(())
    }

    #[exclusive]
    pub fn clear(&mut self) -> Result<()> {
        {
            let start_ptr = self
                .offset_list
                .as_ptr()
                // Leave enough room for the unsized len copy
                .wrapping_byte_add(U32_SIZE)
                .cast::<()>();
            let end_ptr = self
                .unsized_data_ptr()
                .wrapping_byte_add(self.unsized_size.usize())
                .cast::<()>();

            let source_ptr = self.list_ptr.cast_const().cast::<()>();
            unsafe {
                ExclusiveRecurse::remove_bytes(self, source_ptr, start_ptr..end_ptr)?;
            };
        }

        self.list_ptr = ptr_meta::from_raw_parts_mut(self.list_ptr.cast::<()>(), 0);
        self.len.0 = 0;
        self.unsized_list_len().0 = 0;
        self.unsized_size.0 = 0;
        Ok(())
    }
}

unsafe impl<T, C> UnsizedInit<DefaultInit> for UnsizedList<T, C>
where
    T: UnsizedType + UnsizedInit<DefaultInit> + ?Sized,
    C: UnsizedListOffset,
{
    const INIT_BYTES: usize = U32_SIZE * 3;

    unsafe fn init(bytes: &mut &mut [u8], _init: DefaultInit) -> Result<()> {
        let unsized_size_bytes = bytes.try_advance_array::<U32_SIZE>().with_context(|| {
            format!(
                "Failed to read unsized size bytes during initialization of {}",
                std::any::type_name::<Self>()
            )
        })?;
        unsized_size_bytes.copy_from_slice(&<[u8; U32_SIZE]>::zeroed());

        let both_len_bytes = bytes
            .try_advance_array::<{ U32_SIZE * 2 }>()
            .with_context(|| {
                format!(
                    "Failed to read both length bytes during initialization of {}",
                    std::any::type_name::<Self>()
                )
            })?;
        both_len_bytes.copy_from_slice(&<[u8; U32_SIZE * 2]>::zeroed());
        Ok(())
    }
}

impl<T, C> UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    #[inline]
    fn init_list_header<'a, const N: usize, I>(bytes: &mut &'a mut [u8]) -> Result<&'a mut [C; N]>
    where
        T: UnsizedInit<I>,
    {
        let unsized_size_bytes = bytes.advance_array::<U32_SIZE>();
        let unsized_len = (T::INIT_BYTES * N)
            .to_u32()
            .context("Total init bytes must be less than u32::MAX")?;
        *unsized_size_bytes = unsized_len.to_le_bytes();

        let len_l: u32 = N.to_u32().context("N must be less than u32::MAX")?;
        let len_bytes = bytes.try_advance(U32_SIZE).with_context(|| {
            format!(
                "Failed to advance {} bytes for length in list header initialization of {}",
                U32_SIZE,
                std::any::type_name::<Self>()
            )
        })?;
        len_bytes.copy_from_slice(bytes_of(&len_l));

        let offset_slice_bytes = bytes.try_advance(N * size_of::<C>()).with_context(|| {
            format!(
                "Failed to advance {} bytes for offset slice in list header initialization of {}",
                N * size_of::<C>(),
                std::any::type_name::<Self>()
            )
        })?;
        let offset_slice: &mut [C] = cast_slice_mut(offset_slice_bytes);

        let offset_len_bytes = bytes.try_advance(U32_SIZE).with_context(|| {
            format!(
                "Failed to advance {} bytes for offset length in list header initialization of {}",
                U32_SIZE,
                std::any::type_name::<Self>()
            )
        })?;
        offset_len_bytes.copy_from_slice(bytes_of(&len_l));

        Ok(offset_slice.try_into()?)
    }

    #[inline]
    fn init_offset_slice<const N: usize, I, OI>(
        offset_slice: &mut [C; N],
        inits: [OI; N],
    ) -> Result<()>
    where
        T: UnsizedInit<I>,
        C: UnsizedListOffset<ListOffsetInit = OI>,
    {
        for (index, (item, init)) in offset_slice.iter_mut().zip(inits.into_iter()).enumerate() {
            *item = C::from_usize_offset(index * T::INIT_BYTES, init)?;
        }
        Ok(())
    }
}

unsafe impl<const N: usize, T, C, I> UnsizedInit<[I; N]> for UnsizedList<T, C>
where
    T: UnsizedType + UnsizedInit<I> + ?Sized,
    C: UnsizedListOffset<ListOffsetInit = ()>,
{
    const INIT_BYTES: usize = U32_SIZE * 3 + (N * size_of::<C>()) + T::INIT_BYTES * N;

    unsafe fn init(bytes: &mut &mut [u8], array: [I; N]) -> Result<()> {
        let offset_slice = Self::init_list_header::<N, _>(bytes)?;
        Self::init_offset_slice(offset_slice, [(); N])?;

        for item in array {
            unsafe { T::init(bytes, item)? };
        }
        Ok(())
    }
}

unsafe impl<const N: usize, T, C, I> UnsizedInit<&[I; N]> for UnsizedList<T, C>
where
    I: Clone,
    T: UnsizedType + UnsizedInit<I> + ?Sized,
    C: UnsizedListOffset<ListOffsetInit = ()>,
{
    const INIT_BYTES: usize = U32_SIZE * 3 + (N * size_of::<C>()) + T::INIT_BYTES * N;

    unsafe fn init(bytes: &mut &mut [u8], array: &[I; N]) -> Result<()> {
        let offset_slice = Self::init_list_header::<N, _>(bytes)?;
        Self::init_offset_slice(offset_slice, [(); N])?;

        for item in array {
            unsafe { T::init(bytes, item.clone())? };
        }
        Ok(())
    }
}

unsafe impl<const N: usize, T, C, I, OI> UnsizedInit<([I; N], [OI; N])> for UnsizedList<T, C>
where
    T: UnsizedType + UnsizedInit<I> + ?Sized,
    C: UnsizedListOffset<ListOffsetInit = OI>,
{
    const INIT_BYTES: usize = U32_SIZE * 3 + (N * size_of::<C>()) + T::INIT_BYTES * N;

    unsafe fn init(bytes: &mut &mut [u8], arrays: ([I; N], [OI; N])) -> Result<()> {
        let offset_slice = Self::init_list_header::<N, _>(bytes)?;
        Self::init_offset_slice(offset_slice, arrays.1)?;

        for item in arrays.0 {
            unsafe { T::init(bytes, item)? };
        }
        Ok(())
    }
}

unsafe impl<const N: usize, T, C, I, OI> UnsizedInit<(&[I; N], &[OI; N])> for UnsizedList<T, C>
where
    I: Clone,
    OI: Clone,
    T: UnsizedType + UnsizedInit<I> + ?Sized,
    C: UnsizedListOffset<ListOffsetInit = OI>,
{
    const INIT_BYTES: usize = U32_SIZE * 3 + (N * size_of::<C>()) + T::INIT_BYTES * N;

    unsafe fn init(bytes: &mut &mut [u8], arrays: (&[I; N], &[OI; N])) -> Result<()> {
        let offset_slice = Self::init_list_header::<N, _>(bytes)?;
        Self::init_offset_slice(offset_slice, arrays.1.clone())?;

        for item in arrays.0 {
            unsafe { T::init(bytes, item.clone())? };
        }
        Ok(())
    }
}

impl<'a, T, C> IntoIterator for &'a UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Item = Result<T::Ref<'a>>;
    type IntoIter = UnsizedListIter<'a, T, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, C> IntoIterator for &'a mut UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Item = Result<T::Mut<'a>>;
    type IntoIter = UnsizedListIterMut<'a, T, C>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[derive(Debug, Clone)]
pub struct UnsizedListWithOffsetIter<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    list: &'a UnsizedList<T, C>,
    index: usize,
}

#[derive(Debug)]
pub struct UnsizedListWithOffsetIterMut<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    list: &'a mut UnsizedList<T, C>,
    index: usize,
}

#[derive(Debug, Clone)]
pub struct UnsizedListIter<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    iter: UnsizedListWithOffsetIter<'a, T, C>,
}

#[derive(Debug)]
pub struct UnsizedListIterMut<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    iter: UnsizedListWithOffsetIterMut<'a, T, C>,
}

impl<'a, T, C> Iterator for UnsizedListWithOffsetIter<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Item = Result<(T::Ref<'a>, C)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.list.len() {
            return None;
        }
        let (start, end) = self
            .list
            .get_unsized_range(self.index)
            .expect("Index is in bounds");

        let mut item_data = unsafe {
            slice::from_raw_parts(
                self.list.unsized_data_ptr().wrapping_byte_add(start),
                end - start,
            )
        };
        let offset = self.list.offset_list[self.index];
        self.index += 1;
        Some(T::get_ref(&mut item_data).map(|item| (item, offset)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.list.len() - self.index;
        (len, Some(len))
    }
}

impl<'a, T, C> Iterator for UnsizedListWithOffsetIterMut<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Item = Result<(T::Mut<'a>, C)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.list.len() {
            return None;
        }
        let (start, end) = self
            .list
            .get_unsized_range(self.index)
            .expect("Index is in bounds");

        // let mut bytes = self.list.unsized_list_bytes();
        // bytes.try_advance(start)?;

        let mut item_data = core::ptr::slice_from_raw_parts_mut(
            self.list.unsized_data_ptr_mut().wrapping_byte_add(start),
            end - start,
        );
        let offset = self.list.offset_list[self.index];
        self.index += 1;
        Some(unsafe { T::get_mut(&mut item_data) }.map(|item| (item, offset)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.list.len() - self.index;
        (len, Some(len))
    }
}

macro_rules! fused_iter {
    ($name:ident) => {
        impl<'a, T, C> FusedIterator for $name<'a, T, C>
        where
            T: UnsizedType + ?Sized,
            C: UnsizedListOffset,
        {
        }
    };
}

macro_rules! base_iter_impls {
    ($($name:ident),*) => {
        $(
            impl<'a, T, C> ExactSizeIterator for $name<'a, T, C>
            where
                T: UnsizedType + ?Sized,
                C: UnsizedListOffset,
            {
                fn len(&self) -> usize {
                    self.list.len() - self.index
                }
            }

            fused_iter!($name);
        )*
    }
}

base_iter_impls!(UnsizedListWithOffsetIter, UnsizedListWithOffsetIterMut);

macro_rules! iter_impls {
    ($($name:ident: $item:ty),*) => {
        $(
            impl<'a, T, C> Iterator for $name<'a, T, C>
            where
                T: UnsizedType + ?Sized,
                C: UnsizedListOffset,
            {
                type Item = Result<$item>;

                fn next(&mut self) -> Option<Self::Item> {
                    self.iter.next().map(|item| item.map(|item| item.0))
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    self.iter.size_hint()
                }
            }

            impl<'a, T, C> ExactSizeIterator for $name<'a, T, C>
            where
                T: UnsizedType + ?Sized,
                C: UnsizedListOffset,
            {
                fn len(&self) -> usize {
                    self.iter.len()
                }
            }

            fused_iter!($name);
        )*
    }
}

iter_impls!(UnsizedListIter: T::Ref<'a>, UnsizedListIterMut: T::Mut<'a>);

#[cfg(all(test, feature = "test_helpers"))]
mod tests {
    use super::*;
    use crate::{
        prelude::*,
        unsize::{test_helpers::TestByteSet, NewByteSet},
    };
    use pretty_assertions::assert_eq;
    use star_frame_proc::unsized_type;

    // TODO: write better tests

    #[unsized_type(skip_idl)]
    struct TestStruct {
        sized: u8,
        #[unsized_start]
        list: List<u8>,
    }

    #[test]
    fn list_iters() -> Result<()> {
        type TestList = UnsizedList<List<u8>>;
        let byte_arrays = [[100u8, 101, 102], [150, 151, 152], [200, 201, 202]];
        let test_bytes = TestByteSet::<TestList>::new_from_init(byte_arrays)?;
        let mut owned = byte_arrays.map(|array| array.to_vec()).to_vec();
        let mut unsized_lists = test_bytes.data_mut()?;
        unsized_lists.push([1, 2, 3])?;
        owned.push(vec![1, 2, 3]);
        for (list, owned_list) in unsized_lists.iter_with_offsets_mut().zip(owned.iter_mut()) {
            let (mut list, _) = list?;
            assert_eq!(&**list, owned_list);
            for (item, owned_item) in list.iter_mut().zip(owned_list.iter_mut()) {
                *item += 1;
                *owned_item += 1;
            }
            for (item, owned_item) in list.iter().zip(owned_list.iter()) {
                assert_eq!(item, owned_item);
            }
            assert_eq!(&**list, owned_list);
        }

        for (list, owned_list) in unsized_lists.iter().zip(owned.iter()) {
            let list = list?;
            assert_eq!(&**list, owned_list);
        }

        unsized_lists.remove(0)?;
        let mut first = unsized_lists.index_exclusive(0)?;
        first.push(10)?;
        first.remove(0)?;

        owned.remove(0);
        owned[0].push(10);
        owned[0].remove(0);

        assert_eq!(unsized_lists.len(), owned.len());
        for (list, owned_list) in unsized_lists.iter_with_offsets_mut().zip(owned.iter_mut()) {
            let (list, _) = list?;
            assert_eq!(&**list, owned_list);
        }
        let to_owned = TestList::owned_from_ref(&TestList::mut_as_ref(&unsized_lists))?;
        assert_eq!(to_owned, owned);
        Ok(())
    }

    #[test]
    fn test_from_owned() -> Result<()> {
        let owned = vec![
            vec![<PackedValue<u32>>::from(1), 2.into(), 3.into()],
            vec![4.into(), 5.into()],
        ];
        let test_bytes = TestByteSet::<UnsizedList<List<PackedValue<u32>>>>::new(owned.clone())?;
        assert_eq!(test_bytes.owned()?, owned);
        Ok(())
    }

    #[test]
    fn test_unsized_list_crud() -> Result<()> {
        let mut owned_list = vec![vec![0u8, 1, 2], vec![10, 11, 12], vec![20, 21, 22]];
        let map = UnsizedList::<List<u8>>::new_byte_set(owned_list.clone())?;
        let mut data = map.data_mut()?;
        // insert a few elements
        data.push([0, 1, 2])?;
        owned_list.push(vec![0, 1, 2]);
        assert_eq!(data.len(), 4);
        data.push([10, 11, 12])?;
        owned_list.push(vec![10, 11, 12]);
        assert_eq!(data.len(), 5);
        data.push([20, 21, 22])?;
        owned_list.push(vec![20, 21, 22]);
        assert_eq!(data.len(), 6);
        for (item, owned_item) in data.iter().zip(owned_list.iter()) {
            let item = item?;
            assert_eq!(&**item, owned_item);
        }

        let mut second_item = data.get_exclusive(1)?.expect("Second item exists");
        let second_item_owned = &mut owned_list[1];

        second_item.push(13)?;
        second_item_owned.push(13);

        second_item.insert(0, 9)?;
        second_item_owned.insert(0, 9);

        second_item.remove(1)?;
        second_item_owned.remove(1);
        assert_eq!(&***second_item, &*second_item_owned);

        drop(data);
        let data_to_owned = map.owned()?;
        assert_eq!(owned_list, data_to_owned);
        let mut data_mut = map.data_mut()?;
        data_mut.clear()?;
        owned_list.clear();

        data_mut.push([0, 1, 2])?;
        owned_list.push(vec![0, 1, 2]);
        data_mut.push([10, 11, 12])?;
        owned_list.push(vec![10, 11, 12]);

        let slice1 = data_mut.get(0)?.unwrap();
        assert_eq!(&**slice1, &[0, 1, 2]);
        let mut slice2 = data_mut.get_mut(1)?.unwrap();
        slice2[0] = 100;
        owned_list[1][0] = 100;
        assert_eq!(&**slice2, &[100, 11, 12]);
        drop(data_mut);
        let data_to_owned = map.owned()?;
        assert_eq!(owned_list, data_to_owned);
        Ok(())
    }
}
