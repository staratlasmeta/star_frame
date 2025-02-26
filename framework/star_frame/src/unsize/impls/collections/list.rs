use std::any::type_name;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter::once;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut, RangeBounds};
use std::ptr;
use std::ptr::addr_of_mut;

use anyhow::{bail, ensure, Context};
use bytemuck::checked::{try_cast_slice, try_cast_slice_mut, try_from_bytes, try_from_bytes_mut};
use bytemuck::{
    bytes_of, cast_slice, cast_slice_mut, from_bytes, CheckedBitPattern, NoUninit, Pod,
};
use derivative::Derivative;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use solana_program::program_memory::sol_memmove;
use typenum::True;

use advance::{Advance, Length};
use star_frame::unsize::ref_wrapper::RefDeref;

use crate::align1::Align1;
use crate::data_types::PackedValue;
use crate::unsize::init::DefaultInit;
use crate::unsize::init::UnsizedInit;
use crate::unsize::ref_wrapper::{
    AsBytes, AsMutBytes, RefDerefMut, RefWrapper, RefWrapperMutExt, RefWrapperTypes,
};
use crate::unsize::resize::Resize;
use crate::unsize::FromBytesReturn;
use crate::unsize::UnsizedType;
use crate::util::uninit_array_bytes;
use crate::Result;

/// A marker trait for types that can be used as the length of a [`List<T>`].
pub trait ListLength: Pod + ToPrimitive + FromPrimitive {}
impl<T> ListLength for T where T: Pod + ToPrimitive + FromPrimitive {}

#[derive(Align1, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct List<T, L = u32>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    len: PackedValue<L>,
    phantom_t: PhantomData<T>,
    bytes: [u8],
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::idl::TypeToIdl;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<T, L> TypeToIdl for List<T, L>
    where
        T: CheckedBitPattern + NoUninit + Align1 + TypeToIdl,
        L: ListLength + TypeToIdl,
    {
        type AssociatedProgram = T::AssociatedProgram;
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
    pub fn len(&self) -> usize {
        self.len.to_usize().expect("Invalid length")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self[index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            Some(&mut self[index])
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &[T]
    where
        T: Pod,
    {
        cast_slice(&self.bytes)
    }

    pub fn as_mut_slice(&mut self) -> &mut [T]
    where
        T: Pod,
    {
        cast_slice_mut(&mut self.bytes)
    }

    pub fn as_checked_slice(&self) -> Result<&[T]> {
        try_cast_slice(&self.bytes).map_err(Into::into)
    }

    pub fn as_checked_mut_slice(&mut self) -> Result<&mut [T]> {
        try_cast_slice_mut(&mut self.bytes).map_err(Into::into)
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
    /// # use star_frame::unsize::{List, TestByteSet};
    /// let bytes: TestByteSet<List<u8>> = TestByteSet::new(&[0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]).unwrap();
    /// let s = bytes.immut().unwrap();
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
    T: Pod + Align1,
    L: ListLength,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T, L> DerefMut for List<T, L>
where
    T: Pod + Align1,
    L: ListLength,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
impl<T, L> Index<usize> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        try_from_bytes(&self.bytes[index * size_of::<T>()..][..size_of::<T>()])
            .expect("Invalid data for index")
    }
}
impl<T, L> IndexMut<usize> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        try_from_bytes_mut(&mut self.bytes[index * size_of::<T>()..][..size_of::<T>()])
            .expect("Invalid data for index")
    }
}
impl<T, L, R> Index<(R,)> for List<T, L>
where
    T: Pod + Align1,
    L: ListLength,
    R: RangeBounds<usize>,
{
    type Output = [T];

    fn index(&self, index: (R,)) -> &Self::Output {
        let start = match index.0.start_bound() {
            std::ops::Bound::Included(&start) => start * size_of::<T>(),
            std::ops::Bound::Excluded(&start) => (start + 1) * size_of::<T>(),
            std::ops::Bound::Unbounded => 0,
        };
        let end = match index.0.end_bound() {
            std::ops::Bound::Included(&end) => (end + 1) * size_of::<T>(),
            std::ops::Bound::Excluded(&end) => end * size_of::<T>(),
            std::ops::Bound::Unbounded => {
                self.len.to_usize().expect("Invalid length") * size_of::<T>()
            }
        };
        try_cast_slice(&self.bytes[start..end]).expect("Invalid data for range")
    }
}
impl<T, L, R> IndexMut<(R,)> for List<T, L>
where
    T: Pod + Align1,
    L: ListLength,
    R: RangeBounds<usize>,
{
    fn index_mut(&mut self, index: (R,)) -> &mut Self::Output {
        let start = match index.0.start_bound() {
            std::ops::Bound::Included(&start) => start * size_of::<T>(),
            std::ops::Bound::Excluded(&start) => (start + 1) * size_of::<T>(),
            std::ops::Bound::Unbounded => 0,
        };
        let end = match index.0.end_bound() {
            std::ops::Bound::Included(&end) => (end + 1) * size_of::<T>(),
            std::ops::Bound::Excluded(&end) => end * size_of::<T>(),
            std::ops::Bound::Unbounded => {
                self.len.to_usize().expect("Invalid length") * size_of::<T>()
            }
        };
        try_cast_slice_mut(&mut self.bytes[start..end]).expect("Invalid data for range")
    }
}
unsafe impl<T, L> UnsizedType for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type RefMeta = ();
    type RefData = ListRef<T, L>;
    type IsUnsized = True;
    type Owned = Vec<T>;

    fn from_bytes<S: AsBytes>(
        bytes: S,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        let mut bytes_slice = AsBytes::as_bytes(&bytes)?;
        let len_l = from_bytes::<PackedValue<L>>(bytes_slice.try_advance(size_of::<L>())?);
        let len = len_l
            .to_usize()
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size to usize"))?;
        if bytes_slice.len() < len * size_of::<T>() {
            bail!(
                "Bytes (len: {}) not long enough for list (list bytes len: {})",
                bytes_slice.len(),
                len * size_of::<T>()
            );
        }
        Ok(FromBytesReturn {
            bytes_used: size_of::<L>() + size_of::<T>() * len,
            meta: (),
            ref_wrapper: unsafe { RefWrapper::new(bytes, ListRef(PhantomData)) },
        })
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned> {
        Ok(r.as_checked_slice()?.to_vec())
    }
}
impl<T, L> UnsizedInit<DefaultInit> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength + Zero,
{
    const INIT_BYTES: usize = size_of::<L>();

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        _arg: DefaultInit,
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        let bytes = unsafe { AsMutBytes::as_mut_bytes(&mut super_ref) }?;
        bytes[0..size_of::<L>()].copy_from_slice(bytes_of(&L::zeroed()));
        Ok((
            unsafe { RefWrapper::new(super_ref, ListRef(PhantomData)) },
            (),
        ))
    }
}

impl<const N: usize, T, L> UnsizedInit<[T; N]> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength + Zero,
{
    const INIT_BYTES: usize = <Self as UnsizedInit<&[T; N]>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        array: [T; N],
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        unsafe { <Self as UnsizedInit<&[T; N]>>::init(super_ref, &array) }
    }
}

impl<const N: usize, T, L> UnsizedInit<&[T; N]> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength + Zero,
{
    const INIT_BYTES: usize = size_of::<L>() + size_of::<T>() * N;

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        array: &[T; N],
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        let bytes = unsafe { AsMutBytes::as_mut_bytes(&mut super_ref) }?;
        let len_bytes = L::from_usize(N).with_context(|| {
            format!(
                "Init array length larger than max size of List length {}",
                type_name::<L>()
            )
        })?;
        bytes[0..size_of::<L>()].copy_from_slice(bytes_of(&len_bytes));
        bytes[size_of::<L>()..].copy_from_slice(uninit_array_bytes(array));
        Ok((
            unsafe { RefWrapper::new(super_ref, ListRef(PhantomData)) },
            (),
        ))
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""), Copy(bound = ""))]
pub struct ListRef<T, L>(PhantomData<fn() -> (T, L)>);
impl<S, T, L> RefDeref<S> for ListRef<T, L>
where
    S: AsBytes,
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Target = List<T, L>;

    fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
        let sup_ref = RefWrapperTypes::sup(wrapper);
        let bytes = AsBytes::as_bytes(sup_ref).expect("Invalid bytes");
        unsafe { &*ptr::from_raw_parts(bytes.as_ptr().cast(), bytes.len() - size_of::<L>()) }
    }
}
impl<S, T, L> RefDerefMut<S> for ListRef<T, L>
where
    S: AsMutBytes,
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
        let bytes = unsafe {
            let sup_mut = RefWrapperMutExt::sup_mut(wrapper);
            AsMutBytes::as_mut_bytes(sup_mut).expect("Invalid bytes")
        };
        unsafe {
            &mut *ptr::from_raw_parts_mut(bytes.as_mut_ptr().cast(), bytes.len() - size_of::<L>())
        }
    }
}

pub trait ListExt: DerefMut<Target = List<Self::Item, Self::Len>> {
    type Item: CheckedBitPattern + NoUninit + Align1;
    type Len: ListLength;

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
        Some(self.index_mut(index))
    }

    fn push(&mut self, item: Self::Item) -> Result<()> {
        self.push_all(once(item))
    }
    fn push_all<I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Borrow<Self::Item>,
    {
        self.insert_all(self.len(), items)
    }
    fn insert(&mut self, index: usize, item: Self::Item) -> Result<()> {
        self.insert_all(index, once(item))
    }
    fn insert_all<I>(&mut self, index: usize, items: I) -> Result<()>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Borrow<Self::Item>;

    fn remove(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..=index)
    }
    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()>;
    fn pop(&mut self) -> Result<Option<Self::Item>> {
        let len = self.len();
        if len == 0 {
            return Ok(None);
        }
        let item = self[len - 1];
        self.remove(len - 1)?;
        Ok(Some(item))
    }
}
impl<R: ?Sized, T, L> ListExt for R
where
    R: DerefMut<Target = List<T, L>> + RefWrapperMutExt<Ref = ListRef<T, L>>,
    R::Super: Resize<()>,
    T: CheckedBitPattern + NoUninit + Align1,
    L: ListLength,
{
    type Item = T;
    type Len = L;

    fn insert_all<I>(&mut self, index: usize, items: I) -> Result<()>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Borrow<Self::Item>,
    {
        let items = items.into_iter();
        let old_len = self.len();
        ensure!(index <= old_len, "Index out of bounds");
        let item_count = items.len();
        let new_len = old_len + item_count;
        let new_len_l = L::from_usize(new_len)
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size {new_len} to L"))?;
        let new_byte_len = size_of::<L>() + size_of::<T>() * new_len;
        unsafe { Resize::resize(RefWrapperMutExt::sup_mut(self), new_byte_len, ())? }
        self.len = new_len_l.into();

        let start_byte_index = index * size_of::<T>();
        if index < old_len {
            let end_byte_index = start_byte_index + item_count * size_of::<T>();
            // just need to shift the displaced bytes
            let byte_count = (old_len - index) * size_of::<T>();

            let start_ptr = addr_of_mut!(self.bytes[start_byte_index]);
            let end_ptr = addr_of_mut!(self.bytes[end_byte_index]);
            unsafe { sol_memmove(end_ptr, start_ptr, byte_count) }
        }
        let bytes = &mut self.bytes;
        for (byte_index, item) in items
            .enumerate()
            .map(|(i, item)| (start_byte_index + i * size_of::<T>(), item))
        {
            bytes[byte_index..][..size_of::<T>()].copy_from_slice(bytes_of(item.borrow()));
        }

        Ok(())
    }

    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()> {
        let old_len = self.len();

        let start_index = match range.start_bound() {
            std::ops::Bound::Included(&start) => start,
            std::ops::Bound::Excluded(&start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end_index = match range.end_bound() {
            std::ops::Bound::Included(&end) => end + 1,
            std::ops::Bound::Excluded(&end) => end,
            std::ops::Bound::Unbounded => old_len,
        };
        if start_index == end_index {
            return Ok(());
        }
        ensure!(start_index <= end_index, "Invalid range");
        ensure!(end_index <= old_len, "Index out of bounds");

        // Would call drop, but copy requirement on `T` means drop is trivial

        let start_byte_index = start_index * size_of::<T>();
        let end_byte_index = end_index * size_of::<T>();
        let remaining_bytes = self.bytes.len() - end_byte_index;

        let start_byte_ptr = addr_of_mut!(self.bytes[start_byte_index]);
        if end_index != old_len {
            let end_byte_ptr = addr_of_mut!(self.bytes[end_byte_index]);
            unsafe { sol_memmove(start_byte_ptr, end_byte_ptr, remaining_bytes) }
        }

        let new_len = old_len - (end_index - start_index);
        let new_len_l = L::from_usize(new_len)
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size {new_len} to L"))?;
        let new_byte_len = size_of::<L>() + size_of::<T>() * new_len;

        unsafe { Resize::resize(RefWrapperMutExt::sup_mut(self), new_byte_len, ())? }

        self.len = new_len_l.into();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use bytemuck::{Pod, Zeroable};
    use pretty_assertions::assert_eq;
    use star_frame_proc::Align1;

    #[derive(Debug, PartialEq, Eq, Copy, Clone, Pod, Align1, Zeroable)]
    #[repr(C, packed)]
    struct TestStruct {
        val1: u32,
        val2: u32,
    }

    #[test]
    fn list_test() -> anyhow::Result<()> {
        let mut watcher = Vec::new();
        let mut test_set = TestByteSet::<List<TestStruct>>::new(DefaultInit)?;

        // test no-ops
        {
            let mut mutable = test_set.mutable()?;
            mutable.remove_range(..)?;
            mutable.remove_range(10..10)?;
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let struct1 = TestStruct { val1: 0, val2: 1 };
            let struct2 = TestStruct { val1: 2, val2: 3 };
            mutable.push_all([struct1, struct2])?;
            watcher.extend([struct1, struct2]);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let val = TestStruct { val1: 4, val2: 5 };
            mutable.push(val)?;
            watcher.push(val);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let val = TestStruct { val1: 6, val2: 7 };
            mutable.insert(0, val)?;
            watcher.insert(0, val);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let val = TestStruct { val1: 8, val2: 9 };
            mutable.insert(1, val)?;
            watcher.insert(1, val);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            mutable.remove_range(0..=1)?;
            watcher.drain(0..=1);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            mutable.remove(0)?;
            watcher.remove(0);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let byte_pop = mutable.pop()?;
            let watcher_pop = watcher.pop();
            assert!(byte_pop.is_some());
            assert_eq!(**mutable, watcher);
            assert_eq!(byte_pop, watcher_pop);
        }
        assert_eq!(**test_set.immut()?, watcher);

        Ok(())
    }
}
