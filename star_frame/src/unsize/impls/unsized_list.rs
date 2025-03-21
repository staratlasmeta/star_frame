use crate::align1::Align1;
use crate::data_types::PackedValue;
use crate::prelude::{List, UnsizedTypeDataAccess};
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::unsized_impl;
use crate::unsize::wrapper::ExclusiveWrapper;
use crate::unsize::UnsizedType;
use crate::Result;
use advancer::{Advance, AdvanceArray};
use anyhow::{bail, Context};
use bytemuck::cast_slice_mut;
use bytemuck::{bytes_of, from_bytes, Pod, Zeroable};
use core::slice;
use derivative::Derivative;
use num_traits::ToPrimitive;
use star_frame_proc::unsized_type;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr;

#[derive(Align1, Zeroable, Debug, Copy, Clone)]
#[repr(C)]
pub struct OrdOffset<K>
where
    K: Pod + Ord,
{
    offset: PackedU32,
    key: K,
}

#[unsized_type(skip_idl)]
struct InnerProblem {
    #[unsized_start]
    list1: List<u8, u8>,
    list2: List<u8, u8>,
}

// Add 1 byte at ptr + 1 + 4*2 + 1
// Problematic { sized, UnsizedList { [0u32, 2u32], [InnerProblem(List { len, [] }, list2), InnerProblem(list1, list2)] }
// Add 1 byte at ptr + 4*2 + 1
// Add 1 byte at offset 1

// #[unsized_type(skip_idl)]
// struct Problematic {
//     sized: u8,
//     #[unsized_start]
//     list: UnsizedList<InnerProblem>,
// }
//
// #[unsized_type(skip_idl)]
// struct InnerProblem2 {
//     #[unsized_start]
//     list1: InnerProblem,
//     list2: InnerProblem,
// }
//
// #[unsized_impl]
// impl Problematic {
//     #[exclusive]
//     fn modify_inner_list(&mut self) -> Result<()> {
//         let mut list = self.list();
//         // let mut first_element = list.get_exclusive(1)?;
//         // {
//         //    ListMut: ptr,
//         //    ListMut: ptr + 1,
//         // }
//         // let mut list1 = first_element.list1();
//         // list1.push(10)?;
//         // let mut second_list = first_element.list2();
//         // second_list.push(20)?;
//         Ok(())
//     }
// }

unsafe impl<K> Pod for OrdOffset<K> where K: Pod + Ord + Align1 {}

// impl<K> UnsizedListOffset for OrdOffset<K>
// where
//     K: Pod + Ord + Align1,
// {
//     type ListOffsetInit = K;
//
//     #[inline]
//     fn as_list_offset(&self) -> usize {
//         self.offset.as_list_offset()
//     }
//
//     // #[inline]
//     // fn adjust_offset(&mut self, offset: isize) -> Result<()> {
//     //     self.offset.adjust_offset(offset)
//     // }
//
//     #[inline]
//     fn from_offset(offset: usize, init: Self::ListOffsetInit) -> Result<Self> {
//         Ok(Self {
//             offset: UnsizedListOffset::from_offset(offset, ())?,
//             key: init,
//         })
//     }
// }

impl UnsizedListOffset for PackedValue<u32> {
    type ListOffsetInit = ();
    #[inline]
    fn as_list_offset(&self) -> usize {
        self.0 as usize
    }
    #[inline]
    fn as_mut_offset(&mut self) -> &mut PackedU32 {
        self
    }

    #[inline]
    fn from_offset(offset: usize, _init: Self::ListOffsetInit) -> Result<Self> {
        Ok(PackedValue(offset.try_into()?))
    }
}

type PackedU32 = PackedValue<u32>;
impl PackedU32 {
    fn usize(self) -> usize {
        self.0 as usize
    }
}
const U32_SIZE: usize = size_of::<u32>();

pub trait UnsizedListOffset: Pod + Align1 {
    type ListOffsetInit;
    fn as_list_offset(&self) -> usize;
    fn as_mut_offset(&mut self) -> &mut PackedU32;
    fn from_offset(offset: usize, init: Self::ListOffsetInit) -> Result<Self>;
}

#[derive(Debug, Align1)]
#[repr(C)]
pub struct UnsizedList<T, C = PackedU32>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    len: PackedU32,
    unsized_size: PackedU32,
    phantom_t: PhantomData<fn() -> T>,
    offset_list: [C], // Turn this into some generic that impls Align1 + some ToUsize trait or something
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use crate::prelude::System;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<T, C> TypeToIdl for UnsizedList<T, C>
    where
        T: UnsizedType + TypeToIdl,
        C: UnsizedListOffset,
    {
        type AssociatedProgram = System;
        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            todo!()
        }
    }
}

impl<T, C> UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    pub fn len(&self) -> usize {
        self.len.0 as usize
    }
}

#[derive(Derivative)]
#[derivative(Copy(bound = ""), Clone(bound = ""))]
#[derive(Debug)]
pub struct UnsizedListRef<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    list_ptr: *const UnsizedList<T, C>,
    unsized_data_ptr: *const u8,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T, C> Deref for UnsizedListRef<'a, T, C>
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
    unsized_data_ptr: *mut u8, // should be 'a
    inner_exclusive: Option<T::Mut<'a>>,
    phantom: PhantomData<&'a ()>,
}

impl<'a, T, C> Deref for UnsizedListMut<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Target = UnsizedList<T, C>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.list_ptr }
    }
}

impl<'a, T, C> DerefMut for UnsizedListMut<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.list_ptr }
    }
}

impl<'a, T, C> UnsizedListMut<'a, T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    fn fix_offsets(&mut self, source_ptr: *const (), change: isize) -> Result<()> {
        let adjusted_source =
            unsafe { source_ptr.byte_sub(self.unsized_data_ptr as usize) } as usize;
        let index = match self
            .offset_list
            .binary_search_by(|offset| offset.as_list_offset().cmp(&adjusted_source))
        {
            Ok(index) => index + 1,
            Err(index) => index,
        };
        match change.cmp(&0) {
            Ordering::Less => {
                let change: u32 = (-change).try_into()?;
                self.offset_list[index..].iter_mut().for_each(|item| {
                    item.as_mut_offset().0 =
                        unsafe { item.as_mut_offset().0.unchecked_sub(change * 100) }
                });
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                let change: u32 = change.try_into()?;
                self.offset_list[index..].iter_mut().for_each(|item| {
                    item.as_mut_offset().0 = unsafe { item.as_mut_offset().0.unchecked_add(change) }
                });
            }
        }

        Ok(())
    }
}

#[unsized_impl]
impl<T, C> UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    unsafe fn unsized_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.unsized_data_ptr, self.unsized_size.usize()) }
    }
    unsafe fn unsized_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.unsized_data_ptr, self.unsized_size.usize()) }
    }

    #[must_use]
    pub fn total_byte_size(&self) -> usize {
        self.unsized_size.usize() + self.len() * size_of::<C>() + U32_SIZE * 2
    }

    // #[inline]
    // fn fix_offsets(&mut self, source_ptr: *const (), change: isize) -> Result<()> {
    //     let adjusted_source = unsafe { source_ptr.byte_sub(self.unsized_data as usize) } as usize;
    //     let index = match self
    //         .offset_list
    //         .binary_search_by(|offset| offset.as_list_offset().cmp(&adjusted_source))
    //     {
    //         Ok(index) => index + 1,
    //         Err(index) => index,
    //     };
    //     self.offset_list[index..]
    //         .iter_mut()
    //         .try_for_each(|offset| offset.adjust_offset(change))?;
    //     Ok(())
    // }

    pub fn get(&self, index: usize) -> Result<T::Ref<'_>> {
        let offset = self.offset_list[index].as_list_offset();
        let unsized_bytes = unsafe { self.unsized_bytes() };
        T::get_ref(&mut &unsized_bytes[offset..])
    }

    pub fn get_mut(&mut self, index: usize) -> Result<T::Mut<'_>> {
        let offset = self.offset_list[index].as_list_offset();
        let unsized_bytes = unsafe { self.unsized_bytes_mut() };
        T::get_mut(&mut &mut unsized_bytes[offset..])
    }
}

unsafe impl<T, C> UnsizedType for UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    type Ref<'a> = UnsizedListRef<'a, T, C>;
    type Mut<'a> = UnsizedListMut<'a, T, C>;
    type Owned = Vec<T::Owned>;
    // type Owned = Vec<(C, T::Owned)>;
    const ZST_STATUS: bool = {
        assert!(
            T::ZST_STATUS,
            "T cannot be a zero sized type in UnsizedList<T>"
        );
        true
    };

    fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a> {
        UnsizedListRef {
            list_ptr: m.list_ptr,
            unsized_data_ptr: m.unsized_data_ptr,
            phantom: Default::default(),
        }
    }

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        let ptr = data.as_ptr();
        let length_bytes = data.try_advance(U32_SIZE)?;
        let length = from_bytes::<PackedValue<u32>>(length_bytes).usize();

        let unsized_size_bytes = data.try_advance_array::<U32_SIZE>()?;
        let unsized_size = u32::from_le_bytes(*unsized_size_bytes) as usize;

        let _offset_list = data.try_advance(length * U32_SIZE)?;

        let unsized_data = data.try_advance(unsized_size)?;

        Ok(UnsizedListRef {
            list_ptr: unsafe { &*ptr::from_raw_parts(ptr.cast::<()>(), length) },
            unsized_data_ptr: unsized_data.as_ptr(),
            phantom: PhantomData,
        })
    }

    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>> {
        let ptr = data.as_mut_ptr();

        let length_bytes = data.try_advance(U32_SIZE)?;
        let length = from_bytes::<PackedValue<u32>>(length_bytes).usize();

        let unsized_size_bytes = data.try_advance_array::<U32_SIZE>()?;
        let unsized_size = u32::from_le_bytes(*unsized_size_bytes) as usize;

        let _offset_list = data.try_advance(length * U32_SIZE)?;

        let unsized_data = data.try_advance(unsized_size)?;

        Ok(UnsizedListMut {
            list_ptr: unsafe { &mut *ptr::from_raw_parts_mut(ptr.cast::<()>(), length) },
            unsized_data_ptr: unsized_data.as_mut_ptr(),
            inner_exclusive: None,
            phantom: PhantomData,
        })
    }

    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned> {
        let mut owned = Vec::with_capacity(r.len());
        let unsized_bytes = unsafe { r.unsized_bytes() };
        for offset in &r.offset_list {
            let t_ref = T::get_ref(&mut &unsized_bytes[offset.as_list_offset()..])?;
            // owned.push((offset, T::owned_from_ref(t_ref)?));
            owned.push(T::owned_from_ref(t_ref)?);
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
            self_mut.list_ptr = unsafe { self_ptr.byte_offset(change) };
            self_mut.unsized_data_ptr = unsafe { self_mut.unsized_data_ptr.byte_offset(change) };
            // I was not exclusively borrowed at the time, so it's not possible to be currently borrowing an inner element
            self_mut.inner_exclusive = None;
        } else if source_ptr == self_ptr.cast_const().cast() {
            // I am adding or removing elements to myself
            // I must have an exclusive wrapper to self, so it's not possible to be currently borrowing an inner element
            self_mut.inner_exclusive = None;
            // updating offset list should be handled by UnsizedList directly
        } else if source_ptr
            < unsafe {
                self_ptr
                    .cast_const()
                    .cast::<()>()
                    .byte_add(self_mut.total_byte_size())
            }
        {
            // An element in me is changing its size!!
            if let Some(inner) = &mut self_mut.inner_exclusive {
                unsafe { T::resize_notification(inner, source_ptr, change) }?;
            } else {
                bail!("My inner element was not initialized but it thinks it should be resized. This is a bug")
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
            self_mut.fix_offsets(source_ptr, change)?;
        } else {
            // The change happened after me. I must not have exclusive access
            self_mut.inner_exclusive = None;
        }
        Ok(())
    }
}

// insert:
/*

*/

impl<'parent, 'ptr, 'top, 'info, T, C, O, A>
    ExclusiveWrapper<'parent, 'top, 'info, UnsizedListMut<'ptr, T, C>, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    pub fn get_exclusive<'child>(
        &'child mut self,
        index: usize,
    ) -> Result<ExclusiveWrapper<'child, 'top, 'info, T::Mut<'ptr>, O, A>> {
        //todo: are these lifetimes correct?
        unsafe {
            ExclusiveWrapper::try_map_ref(self, |data| {
                let offset = data.offset_list[index].as_list_offset();
                let unsized_data_slice/* '1 */ =
                    slice::from_raw_parts_mut(data.unsized_data_ptr, data.unsized_size.usize());
                let ended_offset = offset
                    + data
                        .offset_list
                        .get(index + 1)
                        .map_or(data.unsized_size.usize(), UnsizedListOffset::as_list_offset);
                let t = T::get_mut(&mut &mut unsized_data_slice[offset..][..ended_offset])?;
                data.inner_exclusive = Some(t);
                Ok(data.inner_exclusive.as_mut().unwrap())
            })
        }
    }
    //
    // pub fn push<Init>(&mut self, item: Init) -> Result<()>
    // where
    //     T: UnsizedInit<Init>,
    // {
    //     let len = self.len();
    //     self.insert(len, item)
    // }
    // pub fn push_all<I, Init>(&mut self, items: I) -> Result<()>
    // where
    //     T: UnsizedInit<Init>,
    //     I: IntoIterator<Item = Init>,
    //     I::IntoIter: ExactSizeIterator,
    // {
    //     self.insert_all(self.len(), items)
    // }
    // pub fn insert<I>(&mut self, index: usize, item: I) -> Result<()>
    // where
    //     T: UnsizedInit<I>,
    // {
    //     // let to_add = T::INIT_BYTES;
    //     //
    //     // let (end_ptr, old_len, new_len, source_ptr) = {
    //     //     let list: &mut UnsizedList<T, C> = self;
    //     //     let old_len = list.len();
    //     //     if index > old_len {
    //     //         bail!("Index {index} is out of bounds for list of length {old_len}",);
    //     //     }
    //     //     let end_ptr = unsafe { list.uns.as_mut_ptr().add(byte_index).cast() };
    //     //     (end_ptr, old_len, new_len, self.0.cast_const().cast::<()>())
    //     // };
    //     // self.insert_all(index, iter::once(item))
    // }
    //
    // pub fn insert_all<I, Init>(&mut self, index: usize, items: I) -> Result<()>
    // where
    //     T: UnsizedInit<Init>,
    //     I: IntoIterator<Item = Init>,
    //     I::IntoIter: ExactSizeIterator,
    // {
    //     // let iter = items.into_iter();
    //     // let to_add = iter.len();
    //     // let byte_index = index * size_of::<T>();
    //     //
    //     // let (end_ptr, old_len, new_len, source_ptr) = {
    //     //     let list: &mut List<T, L> = self;
    //     //     let old_len = list.len();
    //     //     if index > old_len {
    //     //         bail!("Index {index} is out of bounds for list of length {old_len}",);
    //     //     }
    //     //     let new_len =
    //     //         L::from_usize(old_len + to_add).context("Failed to convert new len to L")?;
    //     //     let end_ptr = unsafe { list.bytes.as_mut_ptr().add(byte_index).cast() };
    //     //     (end_ptr, old_len, new_len, self.0.cast_const().cast::<()>())
    //     // };
    //     //
    //     // unsafe {
    //     //     ExclusiveWrapper::add_bytes(
    //     //         self,
    //     //         source_ptr,
    //     //         end_ptr,
    //     //         size_of::<T>() * to_add,
    //     //         |list| {
    //     //             list.len = PackedValue(new_len);
    //     //             list.0 = &mut *ptr::from_raw_parts_mut(
    //     //                 list.0.cast::<()>(),
    //     //                 (old_len + to_add) * size_of::<T>(),
    //     //             );
    //     //             Ok(())
    //     //         },
    //     //     )?;
    //     // };
    //     // for (i, value) in iter.enumerate() {
    //     //     let bytes = &mut self.bytes;
    //     //     bytes[byte_index + i * size_of::<T>()..][..size_of::<T>()]
    //     //         .copy_from_slice(bytes_of(value.borrow()));
    //     // }
    //     Ok(())
    // }
    //
    // pub fn remove(&mut self, index: usize) -> Result<()> {
    //     self.remove_range(index..=index)
    // }
    //
    // pub fn remove_range(&mut self, indexes: impl RangeBounds<usize>) -> Result<()> {
    //     // let start = match indexes.start_bound() {
    //     //     std::ops::Bound::Included(start) => *start,
    //     //     std::ops::Bound::Excluded(start) => start + 1,
    //     //     std::ops::Bound::Unbounded => 0,
    //     // };
    //     // let end = match indexes.end_bound() {
    //     //     std::ops::Bound::Included(end) => *end + 1,
    //     //     std::ops::Bound::Excluded(end) => *end,
    //     //     std::ops::Bound::Unbounded => self.len(),
    //     // };
    //     //
    //     // ensure!(start <= end);
    //     // ensure!(end <= self.len());
    //     //
    //     // let to_remove = end - start;
    //     // let old_len = self.len();
    //     // let new_len = old_len - to_remove;
    //     // let source_ptr: *const () = self.0.cast_const().cast();
    //     //
    //     // unsafe {
    //     //     let start_ptr = self.bytes.as_ptr().add(start * size_of::<T>()).cast();
    //     //     let end_ptr = self.bytes.as_ptr().add(end * size_of::<T>()).cast();
    //     //     ExclusiveWrapper::remove_bytes(self, source_ptr, start_ptr..end_ptr, |list| {
    //     //         list.len = PackedValue(
    //     //             L::from_usize(new_len).context("Failed to convert new list len to L")?,
    //     //         );
    //     //         list.0 =
    //     //             &mut *ptr::from_raw_parts_mut(list.0.cast::<()>(), new_len * size_of::<T>());
    //     //         Ok(())
    //     //     })?;
    //     // };
    //     Ok(())
    // }
}

unsafe impl<T, C> UnsizedInit<DefaultInit> for UnsizedList<T, C>
where
    T: UnsizedType + UnsizedInit<DefaultInit> + ?Sized,
    C: UnsizedListOffset,
{
    const INIT_BYTES: usize = U32_SIZE + U32_SIZE;

    unsafe fn init(bytes: &mut &mut [u8], _init: DefaultInit) -> Result<()> {
        let len_bytes = bytes.advance(U32_SIZE);
        len_bytes[0..U32_SIZE].copy_from_slice(bytes_of(&0u32));

        let unsized_len_bytes = bytes.advance_array::<U32_SIZE>();
        *unsized_len_bytes = 0u32.to_le_bytes();
        Ok(())
    }
}

impl<T, C> UnsizedList<T, C>
where
    T: UnsizedType + ?Sized,
    C: UnsizedListOffset,
{
    #[inline]
    fn init_list_header<'a, const N: usize, I>(bytes: &mut &'a mut [u8]) -> Result<&'a mut [C]>
    where
        T: UnsizedInit<I>,
    {
        let len_l: u32 = N.to_u32().context("N must be less than u32::MAX")?;

        let unsized_len = (T::INIT_BYTES * N)
            .to_u32()
            .context("Total init bytes must be less than u32::MAX")?;

        let len_bytes = bytes.advance(U32_SIZE);
        len_bytes[0..U32_SIZE].copy_from_slice(bytes_of(&len_l));

        let unsized_len_bytes = bytes.advance_array::<U32_SIZE>();
        *unsized_len_bytes = unsized_len.to_le_bytes();

        let offset_slice_bytes = bytes.advance(N * size_of::<C>());
        let offset_slice: &mut [C] = cast_slice_mut(offset_slice_bytes);
        Ok(offset_slice)
    }

    #[inline]
    fn init_offset_slice<I>(offset_slice: &mut [C]) -> Result<()>
    where
        T: UnsizedInit<I>,
        C: UnsizedListOffset<ListOffsetInit = ()>,
    {
        for (index, item) in offset_slice.iter_mut().enumerate() {
            *item = C::from_offset(index * T::INIT_BYTES, ())?;
        }
        Ok(())
    }
}

unsafe impl<const N: usize, T, C, I> UnsizedInit<[I; N]> for UnsizedList<T, C>
where
    T: UnsizedType + UnsizedInit<I> + ?Sized,
    C: UnsizedListOffset<ListOffsetInit = ()>,
{
    const INIT_BYTES: usize = U32_SIZE + U32_SIZE + (N * size_of::<C>()) + T::INIT_BYTES * N;

    unsafe fn init(bytes: &mut &mut [u8], array: [I; N]) -> Result<()> {
        let offset_slice = Self::init_list_header::<N, _>(bytes)?;
        Self::init_offset_slice(offset_slice)?;

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
    const INIT_BYTES: usize = U32_SIZE + U32_SIZE + (N * size_of::<C>()) + T::INIT_BYTES * N;

    unsafe fn init(bytes: &mut &mut [u8], array: &[I; N]) -> Result<()> {
        let offset_slice = Self::init_list_header::<N, _>(bytes)?;
        Self::init_offset_slice(offset_slice)?;

        for item in array {
            unsafe { T::init(bytes, item.clone())? };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::List;
    use crate::unsize::test_helpers::TestByteSet;

    #[test]
    #[allow(unused)]
    fn test_unsized_list() -> Result<()> {
        let byte_array = [
            [[1u8, 2, 3], [10u8, 20u8, 30]],
            [[100u8, 101, 102], [10u8, 20u8, 30]],
        ];
        // let mut vec = byte_array.to_vec();
        let test_bytes = TestByteSet::<UnsizedList<UnsizedList<List<u8, u8>>>>::new(byte_array)?;
        let mut bytes = test_bytes.data_mut()?;
        let mut exclusive = bytes.exclusive();
        let mut first_list = exclusive.get_mut(0)?;
        let mut first_first_list = first_list.get_mut(0)?;
        println!("{:?}", &**first_first_list);

        let mut first_exclusive = exclusive.get_exclusive(0)?;
        println!("{:?}", &**first_exclusive.get_mut(0)?);
        println!("{:?}", &first_exclusive.offset_list);
        let mut exclusive_exclusive = first_exclusive.get_exclusive(1)?;
        println!("{:?}", exclusive_exclusive.as_slice());
        exclusive_exclusive.push(2)?;
        //
        // drop(bytes);
        println!("List: {:?}", test_bytes.owned()?);
        // let mut first_list2 = exclusive.get_mut(0)?;
        // let inner1 = first_list.get_mut(1)?;
        // let inner22 = first_list.get_mut(2)?;
        // {
        //     let list = bytes.get_mut(0)?;
        //     println!("{:?}", &**list);
        //     let list = bytes.get_mut(1)?;
        //     println!("{:?}", &**list);
        // }
        //
        // // drop(exclusive);
        //
        // println!("Bytes {:?}", &**test_bytes.data_mut()?.get_mut(1)?);
        //
        // bytes.exclusive().push_all([10, 11, 12])?;
        // vec.extend_from_slice(&[10, 11, 12]);
        // let list_bytes = &***bytes;
        // println!("{list_bytes:?}");
        // assert_eq!(list_bytes, vec.as_slice());
        Ok(())
    }
}
