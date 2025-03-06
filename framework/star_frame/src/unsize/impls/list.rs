use crate::align1::Align1;
use crate::data_types::PackedValue;
use crate::unsize::init::{DefaultInit, UnsizedInit};
use crate::unsize::wrapper::{ExclusiveWrapper, UnsizedTypeDataAccess};
use crate::unsize::UnsizedType;
use crate::unsize::{AsShared, ResizeOperation};
use crate::util::uninit_array_bytes;
use crate::Result;
use advance::Advance;
use anyhow::{bail, Context};
use bytemuck::{bytes_of, checked, from_bytes, CheckedBitPattern, NoUninit, Pod};
use bytemuck::{cast_slice, cast_slice_mut};
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use std::any::type_name;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, RangeBounds};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
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
    data: [u8],
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
        self.len
            .to_usize()
            .expect("Could not convert list size to usize")
    }

    pub fn is_empty(&self) -> bool {
        self.len
            .to_usize()
            .expect("Could not convert list size to usize")
            == 0
    }
}

impl<T, L> Deref for List<T, L>
where
    L: ListLength,
    T: Pod + Align1,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        cast_slice(&self.data)
    }
}
impl<T, L> DerefMut for List<T, L>
where
    L: ListLength,
    T: Pod + Align1,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        cast_slice_mut(&mut self.data)
    }
}
unsafe impl<T, L> Align1 for List<T, L>
where
    T: Align1 + CheckedBitPattern + NoUninit,
    L: ListLength,
{
}

#[derive(Copy, Clone, Debug)]
pub struct ListRef<'a, T, L>(*const List<T, L>, PhantomData<&'a ()>)
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
pub struct ListMut<'a, T, L>(*mut List<T, L>, PhantomData<&'a ()>)
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
impl<'a, T, L> AsShared<'a> for ListMut<'_, T, L>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Shared<'b> = ListRef<'b, T, L> where Self: 'a, Self: 'b;

    fn as_shared(&'a self) -> Self::Shared<'a> {
        ListRef(self.0, PhantomData)
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

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        let ptr = data.as_ptr();
        let length_bytes = data.try_advance(size_of::<L>())?;
        let len_l = from_bytes::<PackedValue<L>>(length_bytes);
        let length = len_l
            .to_usize()
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size to usize"))?;
        data.advance(size_of::<T>() * length);
        Ok(ListRef(
            unsafe { &*ptr::from_raw_parts(ptr.cast(), size_of::<T>() * length) },
            PhantomData,
        ))
    }

    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>> {
        // let data_ptr = data.as_mut_ptr();
        // let len = data.len();
        // let length_bytes = unsafe {
        //     let amount = size_of::<L>();
        //     let len = data.len();
        //     let ptr = data.as_mut_ptr();
        //     *data = &mut *slice_from_raw_parts_mut(ptr.add(amount), len - amount);
        //     &mut *slice_from_raw_parts_mut(ptr, amount)
        // };
        let length_bytes = data.try_advance(size_of::<L>())?;
        let len_l = from_bytes::<PackedValue<L>>(length_bytes);
        let length = len_l
            .to_usize()
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size to usize"))?;
        // unsafe {
        //     let amount = size_of::<T>() * length;
        //     let len = data.len();
        //     let ptr = data.as_mut_ptr();
        //     *data = &mut *slice_from_raw_parts_mut(ptr.add(amount), len - amount);
        //     &mut *slice_from_raw_parts_mut(ptr, amount)
        // };
        // Advance::advance(data, size_of::<T>() * length);
        let new_data = data.try_advance(size_of::<T>() * length)?;
        let total_advanced = size_of::<T>() * length;
        Ok(ListMut(
            unsafe {
                &mut *ptr::from_raw_parts_mut(
                    length_bytes.as_mut_ptr().cast(),
                    size_of::<T>() * length,
                )
            },
            PhantomData,
        ))
    }

    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned> {
        Ok(checked::try_cast_slice(&r.data)?.to_vec())
    }

    unsafe fn resize_notification(data: &mut &mut [u8], _operation: ResizeOperation) -> Result<()> {
        let length_bytes = data.try_advance(size_of::<L>())?;
        let len_l = from_bytes::<PackedValue<L>>(length_bytes);
        let length = len_l
            .to_usize()
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size to usize"))?;
        data.advance(size_of::<T>() * length);
        Ok(())
    }
}

pub trait ListExclusive<'a, T, L>: DerefMut<Target = ListMut<'a, T, L>>
where
    L: ListLength,
    T: Align1 + NoUninit + CheckedBitPattern,
{
    fn insert(&mut self, index: usize, item: T) -> Result<()> {
        self.insert_all(index, iter::once(item))
    }
    fn insert_all<I>(&mut self, index: usize, items: I) -> Result<()>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Borrow<T>;
    fn push_all<I>(&mut self, items: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.insert_all(self.len(), items)
    }
    fn push(&mut self, item: T) -> Result<()> {
        self.insert(self.len(), item)
    }
    fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()>;
    fn remove(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..=index)
    }
}

impl<'a, 'info, T, O: ?Sized, A, L> ListExclusive<'a, T, L>
    for ExclusiveWrapper<'a, 'info, <List<T, L> as UnsizedType>::Mut<'a>, O, A>
where
    T: Align1 + NoUninit + CheckedBitPattern,
    L: ListLength,
    O: UnsizedType,
    A: UnsizedTypeDataAccess<'info>,
{
    fn insert_all<I>(&mut self, index: usize, items: I) -> Result<()>
    where
        I: IntoIterator,
        I::IntoIter: ExactSizeIterator,
        I::Item: Borrow<T>,
    {
        let list: &mut List<T, L> = self;
        let iter = items.into_iter();
        let to_add = iter.len();
        let old_len = list.len();
        if index > old_len {
            bail!("Index {index} is out of bounds for list of length {old_len}",);
        }
        let new_len = L::from_usize(old_len + to_add).context("Failed to convert new len to L")?;
        unsafe {
            let end_ptr = list.data.as_mut_ptr().add(index).cast();
            ExclusiveWrapper::add_bytes(self, end_ptr, size_of::<T>() * to_add, |l| {
                l.len = PackedValue(new_len);
                Ok(())
            })?;
            ExclusiveWrapper::set_inner(self, |list| {
                list.0 = &mut *ptr::from_raw_parts_mut(
                    list.0.cast::<()>(),
                    (old_len + to_add) * size_of::<T>(),
                );
            });
        }
        for (i, value) in iter.enumerate() {
            self.data[index + i * size_of::<T>()..][..size_of::<T>()]
                .copy_from_slice(bytes_of(value.borrow()));
        }
        Ok(())
    }

    fn remove_range(&mut self, indexes: impl RangeBounds<usize>) -> Result<()> {
        let start = match indexes.start_bound() {
            std::ops::Bound::Included(start) => *start,
            std::ops::Bound::Excluded(start) => start + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match indexes.end_bound() {
            std::ops::Bound::Included(end) => *end + 1,
            std::ops::Bound::Excluded(end) => *end,
            std::ops::Bound::Unbounded => self.len(),
        };

        assert!(start <= end);
        assert!(end <= self.len());

        let to_remove = end - start;
        let old_len = self.len();
        let new_len = old_len - to_remove;

        unsafe {
            let start_ptr = self.data.as_ptr().add(start).cast();
            let end_ptr = self.data.as_ptr().add(end).cast();
            ExclusiveWrapper::remove_bytes(self, start_ptr..end_ptr, |l| {
                l.len = PackedValue(
                    L::from_usize(new_len).context("Failed to convert new list len to L")?,
                );
                Ok(())
            })?;
            ExclusiveWrapper::set_inner(self, |list| {
                list.0 =
                    &mut *ptr::from_raw_parts_mut(list.0.cast::<()>(), new_len * size_of::<T>());
            });
        }
        Ok(())
    }
}

impl<T, L> UnsizedInit<DefaultInit> for List<T, L>
where
    L: ListLength,
    T: CheckedBitPattern + NoUninit + Align1,
{
    const INIT_BYTES: usize = size_of::<L>();

    unsafe fn init(bytes: &mut &mut [u8], _arg: DefaultInit) -> Result<()> {
        bytes.advance(<Self as UnsizedInit<DefaultInit>>::INIT_BYTES);
        Ok(())
    }
}

impl<const N: usize, T, L> UnsizedInit<&[T; N]> for List<T, L>
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
        let array_bytes = bytes.advance(<Self as UnsizedInit<&[T; N]>>::INIT_BYTES);
        array_bytes[0..size_of::<L>()].copy_from_slice(bytes_of(&len_bytes));
        array_bytes[size_of::<L>()..].copy_from_slice(uninit_array_bytes(array));
        Ok(())
    }
}

impl<const N: usize, T, L> UnsizedInit<[T; N]> for List<T, L>
where
    L: ListLength + Zero,
    T: CheckedBitPattern + NoUninit + Align1,
{
    const INIT_BYTES: usize = <Self as UnsizedInit<&[T; N]>>::INIT_BYTES;

    unsafe fn init(bytes: &mut &mut [u8], array: [T; N]) -> Result<()> {
        unsafe { <Self as UnsizedInit<&[T; N]>>::init(bytes, &array) }
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
        bytes.push_all([10, 11, 12])?;
        vec.extend_from_slice(&[10, 11, 12]);
        let list_bytes = &***bytes;
        println!("{:?}", list_bytes);
        assert_eq!(list_bytes, vec.as_slice());
        Ok(())
    }
}
