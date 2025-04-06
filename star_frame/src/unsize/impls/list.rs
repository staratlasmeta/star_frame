use crate::align1::Align1;
use crate::data_types::PackedValue;
use crate::prelude::UnsizedTypeDataAccess;
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::ExclusiveWrapper;
use crate::unsize::{AsShared, FromOwned, UnsizedType};
use crate::util::uninit_array_bytes;
use crate::Result;
use advancer::Advance;
use anyhow::{bail, ensure, Context};
use bytemuck::{bytes_of, checked, from_bytes, CheckedBitPattern, NoUninit, Pod, Zeroable};
use bytemuck::{cast_slice, cast_slice_mut};
use itertools::Itertools;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use std::any::type_name;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut, RangeBounds};
use std::{iter, ptr};

/// A marker trait for types that can be used as the length of a [`List<T> `].
pub trait ListLength: Pod + ToPrimitive + FromPrimitive {}
impl<T> ListLength for T where T: Pod + ToPrimitive + FromPrimitive {}

#[derive(Debug)]
#[repr(C)]
pub struct List<T, L = u32>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    len: PackedValue<L>,
    phantom_t: PhantomData<fn() -> T>,
    bytes: [u8],
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use crate::prelude::System;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<T, L> TypeToIdl for List<T, L>
    where
        T: CheckedBitPattern + NoUninit + Align1 + TypeToIdl,
        L: ListLength + TypeToIdl,
    {
        type AssociatedProgram = System;
        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            let inner_type = T::type_to_idl(idl_definition)?;
            Ok(IdlTypeDef::List {
                item_ty: Box::new(inner_type),
                len_ty: Box::new(L::type_to_idl(idl_definition)?),
            })
        }
    }
}

impl<T, L> List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    #[inline]
    pub fn len(&self) -> usize {
        self.len
            .to_usize()
            .expect("Could not convert list size to usize")
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            Some(&mut self[index])
        } else {
            None
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[T]
    where
        T: Pod,
    {
        cast_slice(&self.bytes)
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T]
    where
        T: Pod,
    {
        cast_slice_mut(&mut self.bytes)
    }

    #[inline]
    pub fn as_checked_slice(&self) -> Result<&[T]> {
        checked::try_cast_slice(&self.bytes).map_err(Into::into)
    }

    #[inline]
    pub fn as_checked_mut_slice(&mut self) -> Result<&mut [T]> {
        checked::try_cast_slice_mut(&mut self.bytes).map_err(Into::into)
    }

    pub fn iter(&self) -> ListIter<'_, T, L> {
        ListIter {
            list: self,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> ListIterMut<'_, T, L> {
        ListIterMut {
            list_bytes_ptr: &mut self.bytes,
            remaining: self.len(),
            phantom_data: Default::default(),
        }
    }

    /// See [`<[T]>::binary_search`]
    pub fn binary_search(&self, x: &T) -> std::result::Result<usize, usize>
    where
        T: Ord,
    {
        Self::binary_search_by(self, |p| p.cmp(x))
    }

    /// See [`<[T]>::binary_search_by`]
    /// ```
    /// # use star_frame::unsize::{impls::List, TestByteSet};
    /// let bytes: TestByteSet<List<u8>> = TestByteSet::new(&[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]).unwrap();
    /// let s = bytes.data_mut().unwrap();
    /// let seek = 13;
    /// assert_eq!(s.binary_search_by(|probe| probe.cmp(&seek)), Ok(9));
    /// let seek = 4;
    /// assert_eq!(s.binary_search_by(|probe| probe.cmp(&seek)), Err(7));
    /// let seek = 100;
    /// assert_eq!(s.binary_search_by(|probe| probe.cmp(&seek)), Err(13));
    /// let seek = 1;
    /// let r = s.binary_search_by(|probe| probe.cmp(&seek));
    /// assert!(match r { Ok(1..=4) => true, _ => false, });
    /// ```
    pub fn binary_search_by<F>(&self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&T) -> Ordering,
    {
        let size = self.len();
        let mut left = 0;
        let mut right = size;
        while left < right {
            let mid = (left + right) / 2;
            match f(&self[mid]) {
                Ordering::Less => left = mid + 1,
                Ordering::Equal => return Ok(mid),
                Ordering::Greater => right = mid,
            }
        }
        Err(left)
    }
}

impl<T, L> Deref for List<T, L>
where
    L: ListLength,
    T: Pod + Align1,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        cast_slice(&self.bytes)
    }
}

impl<T, L> DerefMut for List<T, L>
where
    L: ListLength,
    T: Pod + Align1,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        cast_slice_mut(&mut self.bytes)
    }
}
unsafe impl<T, L> Align1 for List<T, L>
where
    T: Align1 + CheckedBitPattern + NoUninit,
    L: ListLength,
{
}

impl<T, L> Index<usize> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        checked::try_from_bytes(&self.bytes[index * size_of::<T>()..][..size_of::<T>()])
            .expect("Invalid data for index")
    }
}
impl<T, L> IndexMut<usize> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        checked::try_from_bytes_mut(&mut self.bytes[index * size_of::<T>()..][..size_of::<T>()])
            .expect("Invalid data for index")
    }
}

fn get_bounds<T, L>(list: &List<T, L>, range: impl RangeBounds<usize>) -> (usize, usize)
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    let start = match range.start_bound() {
        std::ops::Bound::Included(&start) => start * size_of::<T>(),
        std::ops::Bound::Excluded(&start) => (start + 1) * size_of::<T>(),
        std::ops::Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        std::ops::Bound::Included(&end) => (end + 1) * size_of::<T>(),
        std::ops::Bound::Excluded(&end) => end * size_of::<T>(),
        std::ops::Bound::Unbounded => list.len.to_usize().expect("Invalid length") * size_of::<T>(),
    };
    (start, end)
}
impl<T, L, R> Index<(R,)> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
    R: RangeBounds<usize>,
{
    type Output = [T];

    fn index(&self, index: (R,)) -> &Self::Output {
        let (start, end) = get_bounds(self, index.0);
        checked::try_cast_slice(&self.bytes[start..end]).expect("Invalid data for range")
    }
}
impl<T, L, R> IndexMut<(R,)> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
    R: RangeBounds<usize>,
{
    fn index_mut(&mut self, index: (R,)) -> &mut Self::Output {
        let (start, end) = get_bounds(self, index.0);
        checked::try_cast_slice_mut(&mut self.bytes[start..end]).expect("Invalid data for range")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ListRef<'a, T, L = u32>(*const List<T, L>, PhantomData<&'a ()>)
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1;

impl<'a, T, L> Deref for ListRef<'a, T, L>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
#[derive(Debug)]
pub struct ListMut<'a, T, L = u32>(*mut List<T, L>, PhantomData<&'a ()>)
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1;

impl<'a, T, L> Deref for ListMut<'a, T, L>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<'a, T, L> DerefMut for ListMut<'a, T, L>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl<'a, T, L> AsShared<'a> for ListMut<'a, T, L>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Ref = ListRef<'a, T, L>;
    fn as_shared(&'a self) -> Self::Ref {
        List::mut_as_ref(self)
    }
}

unsafe impl<T, L> UnsizedType for List<T, L>
where
    L: ListLength,
    T: Align1 + CheckedBitPattern + NoUninit,
{
    type Ref<'a> = ListRef<'a, T, L>;
    type Mut<'a> = ListMut<'a, T, L>;
    type Owned = Vec<T>;
    const ZST_STATUS: bool = { size_of::<L>() != 0 };

    fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a> {
        ListRef(m.0, PhantomData)
    }

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        let ptr = data.as_ptr();
        let length_bytes = data.try_advance(size_of::<L>())?;
        let len_l = from_bytes::<PackedValue<L>>(length_bytes);
        let length = len_l
            .to_usize()
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size to usize"))?;
        data.try_advance(size_of::<T>() * length)?;
        Ok(ListRef(
            unsafe { &*ptr::from_raw_parts(ptr.cast::<()>(), size_of::<T>() * length) },
            PhantomData,
        ))
    }

    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>> {
        let length_bytes = data.try_advance(size_of::<L>())?;
        let len_l = from_bytes::<PackedValue<L>>(length_bytes);
        let length = len_l
            .to_usize()
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size to usize"))?;
        data.try_advance(size_of::<T>() * length)?;
        let list_ptr = ptr::from_mut(unsafe {
            &mut *ptr::from_raw_parts_mut(
                length_bytes.as_mut_ptr().cast::<()>(),
                size_of::<T>() * length,
            )
        });
        Ok(ListMut(list_ptr, PhantomData))
    }

    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned> {
        Ok(checked::try_cast_slice(&r.bytes)?.to_vec())
    }

    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()> {
        let self_ptr = self_mut.0;
        if source_ptr < self_ptr.cast_const().cast() {
            self_mut.0 = unsafe { self_ptr.byte_offset(change) };
        }
        Ok(())
    }
}

impl<T, L> FromOwned for List<T, L>
where
    L: ListLength,
    T: Align1 + CheckedBitPattern + NoUninit,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        size_of::<L>() + size_of::<T>() * owned.len()
    }

    fn from_owned(owned: Self::Owned, out: &mut [u8]) -> Result<usize> {
        out[..size_of::<L>()].copy_from_slice(bytes_of(&L::from_usize(owned.len()).unwrap()));
        let mut offset = size_of::<L>();

        for item in &owned {
            out[offset..][..size_of::<T>()].copy_from_slice(bytes_of(item));
            offset += size_of::<T>();
        }

        Ok(size_of::<L>() + size_of::<T>() * owned.len())
    }
}

impl<'parent, 'ptr, 'top, 'info, T, L, O, A>
    ExclusiveWrapper<'parent, 'top, 'info, ListMut<'ptr, T, L>, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
    T: Align1 + NoUninit + CheckedBitPattern,
    L: ListLength,
{
    #[inline]
    pub fn push(&mut self, item: T) -> Result<()> {
        let len = self.len();
        self.insert(len, item)
    }
    #[inline]
    pub fn push_all<I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.insert_all(self.len(), items)
    }
    #[inline]
    pub fn insert(&mut self, index: usize, item: T) -> Result<()> {
        self.insert_all(index, iter::once(item))
    }

    pub fn insert_all<I>(&mut self, index: usize, items: I) -> Result<()>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Borrow<T>,
    {
        let iter = items.into_iter();
        let to_add = iter.len();
        let byte_index = index * size_of::<T>();

        let (end_ptr, old_len, new_len, source_ptr) = {
            let list: &mut List<T, L> = self;
            let old_len = list.len();
            if index > old_len {
                bail!("Index {index} is out of bounds for list of length {old_len}",);
            }
            let new_len =
                L::from_usize(old_len + to_add).context("Failed to convert new len to L")?;
            let end_ptr = unsafe { list.bytes.as_mut_ptr().add(byte_index).cast() };
            (end_ptr, old_len, new_len, self.0.cast_const().cast::<()>())
        };

        unsafe {
            ExclusiveWrapper::add_bytes(
                self,
                source_ptr,
                end_ptr,
                size_of::<T>() * to_add,
                |list| {
                    list.len = PackedValue(new_len);
                    list.0 = &mut *ptr::from_raw_parts_mut(
                        list.0.cast::<()>(),
                        (old_len + to_add) * size_of::<T>(),
                    );
                    Ok(())
                },
            )?;
        };
        // zip_eq to ensure ExactSizeIterator is telling the truth
        for ((i, value), _) in iter.enumerate().zip_eq(0..to_add) {
            let bytes = &mut self.bytes;
            bytes[byte_index + i * size_of::<T>()..][..size_of::<T>()]
                .copy_from_slice(bytes_of(value.borrow()));
        }
        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> Result<Option<()>> {
        if self.len() == 0 {
            return Ok(None);
        }
        self.remove(self.len() - 1).map(Some)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..=index)
    }

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

        ensure!(start <= end);
        ensure!(end <= self.len());

        let to_remove = end - start;
        let old_len = self.len();
        let new_len = old_len - to_remove;
        let source_ptr: *const () = self.0.cast_const().cast();

        unsafe {
            let start_ptr = self.bytes.as_ptr().add(start * size_of::<T>()).cast();
            let end_ptr = self.bytes.as_ptr().add(end * size_of::<T>()).cast();
            ExclusiveWrapper::remove_bytes(self, source_ptr, start_ptr..end_ptr, |list| {
                list.len = PackedValue(
                    L::from_usize(new_len).context("Failed to convert new list len to L")?,
                );
                list.0 =
                    &mut *ptr::from_raw_parts_mut(list.0.cast::<()>(), new_len * size_of::<T>());
                Ok(())
            })?;
        };
        Ok(())
    }
}

unsafe impl<T, L> UnsizedInit<DefaultInit> for List<T, L>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    const INIT_BYTES: usize = size_of::<L>();

    unsafe fn init(bytes: &mut &mut [u8], _arg: DefaultInit) -> Result<()> {
        bytes
            .try_advance(<Self as UnsizedInit<DefaultInit>>::INIT_BYTES)?
            .copy_from_slice(bytes_of(&<PackedValue<L>>::zeroed()));
        Ok(())
    }
}

unsafe impl<const N: usize, T, L> UnsizedInit<&[T; N]> for List<T, L>
where
    L: ListLength + Zero,
    T: CheckedBitPattern + NoUninit + Align1,
{
    const INIT_BYTES: usize = size_of::<L>() + size_of::<T>() * N;

    unsafe fn init(bytes: &mut &mut [u8], array: &[T; N]) -> Result<()> {
        let len_bytes = L::from_usize(N).with_context(|| {
            format!(
                "Init array length larger than max size of List length {}",
                type_name::<L>()
            )
        })?;
        let array_bytes = bytes.try_advance(<Self as UnsizedInit<&[T; N]>>::INIT_BYTES)?;
        array_bytes[0..size_of::<L>()].copy_from_slice(bytes_of(&len_bytes));
        array_bytes[size_of::<L>()..].copy_from_slice(uninit_array_bytes(array));
        Ok(())
    }
}

unsafe impl<const N: usize, T, L> UnsizedInit<[T; N]> for List<T, L>
where
    L: ListLength + Zero,
    T: CheckedBitPattern + NoUninit + Align1,
{
    const INIT_BYTES: usize = <Self as UnsizedInit<&[T; N]>>::INIT_BYTES;

    #[inline]
    unsafe fn init(bytes: &mut &mut [u8], array: [T; N]) -> Result<()> {
        unsafe { <Self as UnsizedInit<&[T; N]>>::init(bytes, &array) }
    }
}

#[derive(Debug, Clone)]
pub struct ListIter<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    list: &'a List<T, L>,
    index: usize,
}

#[derive(Debug)]
pub struct ListIterMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    list_bytes_ptr: *mut [u8],
    remaining: usize,
    phantom_data: PhantomData<&'a mut (T, L)>,
}

impl<'a, T, L> Iterator for ListIter<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.list.len() {
            return None;
        }
        let item = &self.list[self.index];
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.len() - self.index;
        (remaining, Some(remaining))
    }
}
impl<T, L> ExactSizeIterator for ListIter<'_, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    fn len(&self) -> usize {
        self.list.len() - self.index
    }
}

impl<T, L> FusedIterator for ListIter<'_, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
}

impl<'a, T, L> IntoIterator for &'a List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Item = &'a T;
    type IntoIter = ListIter<'a, T, L>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, L> Iterator for ListIterMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let mut list_bytes = unsafe { &mut *self.list_bytes_ptr };
        let item_data = list_bytes.advance(size_of::<T>());
        let item = checked::from_bytes_mut(item_data);

        self.remaining -= 1;
        self.list_bytes_ptr = list_bytes;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T, L> ExactSizeIterator for ListIterMut<'_, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<T, L> FusedIterator for ListIterMut<'_, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
}

impl<'a, T, L> IntoIterator for &'a mut List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Item = &'a mut T;
    type IntoIter = ListIterMut<'a, T, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unsize::test_helpers::TestByteSet;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_list() -> Result<()> {
        let byte_array = [1, 2, 3, 4, 5];
        let mut vec = byte_array.to_vec();
        let test_bytes = TestByteSet::<List<u8>>::new(&byte_array)?;
        let mut bytes = test_bytes.data_mut()?;
        let _ = bytes.exclusive();
        bytes.exclusive().push_all([10, 11, 12])?;
        vec.extend_from_slice(&[10, 11, 12]);

        for (list_item, owned_item) in bytes.iter_mut().zip_eq(vec.iter_mut()) {
            *list_item += 1;
            *owned_item += 1;
        }

        let list_bytes = &***bytes;
        println!("{list_bytes:?}");
        assert_eq!(list_bytes, vec.as_slice());
        Ok(())
    }
}
