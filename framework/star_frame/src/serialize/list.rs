use crate::align1::Align1;
use crate::packed_value::PackedValue;
use crate::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefDerefMut, RefWrapper, RefWrapperMutExt, RefWrapperTypes,
};
use crate::serialize::unsize::init::UnsizedInit;
use crate::serialize::unsize::resize::Resize;
use crate::serialize::unsize::FromBytesReturn;
use crate::serialize::unsize::UnsizedType;
use crate::Result;
use advance::{Advance, Length};
use anyhow::ensure;
use bytemuck::checked::{try_cast_slice, try_cast_slice_mut, try_from_bytes, try_from_bytes_mut};
use bytemuck::{
    bytes_of, cast_slice, cast_slice_mut, from_bytes, CheckedBitPattern, NoUninit, Pod,
};
use derivative::Derivative;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use solana_program::program_memory::sol_memmove;
use star_frame::serialize::ref_wrapper::RefDeref;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::iter::once;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut, RangeBounds};
use std::ptr;
use std::ptr::addr_of_mut;
use typenum::True;

#[derive(Align1, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct List<T, L = u32>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    len: PackedValue<L>,
    phantom_t: PhantomData<T>,
    bytes: [u8],
}
impl<T, L> List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    pub fn len(&self) -> usize {
        self.len.to_usize().expect("Invalid length")
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
}
impl<T, L> Deref for List<T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T, L> DerefMut for List<T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
impl<T, L> Index<usize> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
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
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        try_from_bytes_mut(&mut self.bytes[index * size_of::<T>()..][..size_of::<T>()])
            .expect("Invalid data for index")
    }
}
impl<T, L, R> Index<(R,)> for List<T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
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
    L: Pod + ToPrimitive + FromPrimitive,
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
    L: Pod + ToPrimitive + FromPrimitive,
{
    type RefMeta = ();
    type RefData = ListRef<T, L>;
    type IsUnsized = True;
    type Owned = Vec<T>;

    unsafe fn from_bytes<S: AsBytes>(
        bytes: S,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        let mut bytes_slice = bytes.as_bytes()?;
        let len_l = from_bytes::<PackedValue<L>>(bytes_slice.try_advance(size_of::<L>())?);
        let len = len_l
            .to_usize()
            .ok_or_else(|| anyhow::anyhow!("Could not convert list size to usize"))?;
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
impl<T, L> UnsizedInit<()> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive + Zero,
{
    const INIT_BYTES: usize = size_of::<L>();

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        _arg: (),
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        let bytes = super_ref.as_mut_bytes()?;
        bytes[0..size_of::<L>()].copy_from_slice(bytes_of(&L::zeroed()));
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
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = List<T, L>;

    fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
        let bytes = wrapper.sup().as_bytes().expect("Invalid bytes");
        unsafe { &*ptr::from_raw_parts(bytes.as_ptr().cast(), bytes.len() - size_of::<L>()) }
    }
}
impl<S, T, L> RefDerefMut<S> for ListRef<T, L>
where
    S: AsMutBytes,
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
        let bytes = unsafe { wrapper.sup_mut().as_mut_bytes().expect("Invalid bytes") };
        unsafe {
            &mut *ptr::from_raw_parts_mut(bytes.as_mut_ptr().cast(), bytes.len() - size_of::<L>())
        }
    }
}

pub trait ListExt: DerefMut<Target = List<Self::Item, Self::Len>> {
    type Item: CheckedBitPattern + NoUninit + Align1;
    type Len: Pod + ToPrimitive + FromPrimitive;

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
}
impl<R: ?Sized, T, L> ListExt for R
where
    R: DerefMut<Target = List<T, L>> + RefWrapperMutExt<Ref = ListRef<T, L>>,
    R::Super: Resize<()>,
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
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
        unsafe { self.sup_mut().resize(new_byte_len, ())? }
        self.len = new_len_l.into();

        let start_byte_index = index * size_of::<T>();
        if index < old_len {
            let byte_count = item_count * size_of::<T>();
            let end_byte_index = start_byte_index + byte_count;

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

        unsafe { self.sup_mut().resize(new_byte_len, ())? }

        self.len = new_len_l.into();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::List;
    use crate::serialize::list::ListExt;
    use crate::serialize::test::TestByteSet;
    use bytemuck::{Pod, Zeroable};
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
        let mut test_set = TestByteSet::<List<TestStruct>>::new(())?;

        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let val = TestStruct { val1: 1, val2: 2 };
            mutable.push(val)?;
            watcher.push(val);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let val = TestStruct { val1: 3, val2: 4 };
            mutable.insert(0, val)?;
            watcher.insert(0, val);
            assert_eq!(**mutable, watcher);
        }
        assert_eq!(**test_set.immut()?, watcher);
        {
            let mut mutable = test_set.mutable()?;
            let val = TestStruct { val1: 5, val2: 6 };
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

        Ok(())
    }
}
