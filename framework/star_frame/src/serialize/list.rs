use crate::align1::Align1;
use crate::packed_value::PackedValue;
use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut, PointerBreakup};
use crate::serialize::unsized_type::UnsizedType;
use crate::serialize::{
    FrameworkFromBytes, FrameworkFromBytesMut, FrameworkInit, FrameworkSerialize, ResizeFn,
};
use crate::Result;
use advance::Advance;
use anyhow::bail;
use bytemuck::{
    bytes_of, cast_slice, cast_slice_mut, checked, from_bytes, CheckedBitPattern, NoUninit, Pod,
};
use derivative::Derivative;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memmove;
use std::collections::Bound;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, Index, IndexMut, RangeBounds};
use std::ptr;
use std::ptr::NonNull;

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
        { self.len.0 }.to_usize().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.len().is_zero()
    }
}

impl<T, L> Index<usize> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        checked::try_from_bytes(&self.bytes[index * size_of::<T>()..][..size_of::<T>()]).unwrap()
    }
}

impl<T, L> IndexMut<usize> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        checked::try_from_bytes_mut(&mut self.bytes[index * size_of::<T>()..][..size_of::<T>()])
            .unwrap()
    }
}

unsafe impl<T, L> FrameworkInit<()> for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive + Zero,
{
    const INIT_LENGTH: usize = size_of::<L>();

    unsafe fn init<'a>(
        bytes: &'a mut [u8],
        _arg: (),
        resize: impl ResizeFn<'a, Self::RefMeta>,
    ) -> Result<Self::RefMut<'a>> {
        debug_assert_eq!(bytes.len(), <Self as FrameworkInit<()>>::INIT_LENGTH);
        debug_assert!(bytes.iter().all(|b| *b == 0));
        let len = L::zero();
        bytes.copy_from_slice(bytemuck::bytes_of(&len));
        Ok(ListRefMut {
            phantom_ref: PhantomData,
            ptr: NonNull::from(bytes).cast(),
            metadata: L::zeroed(),
            resize: Box::new(resize),
        })
    }
}

impl<T, L> Deref for List<T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        cast_slice(&self.bytes)
    }
}

impl<T, L> DerefMut for List<T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        cast_slice_mut(&mut self.bytes)
    }
}

impl<T, L> UnsizedType for List<T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type RefMeta = L;
    type Ref<'a> = ListRef<'a, T, L>;
    type RefMut<'a> = ListRefMut<'a, T, L>;
}

#[derive(Debug, Copy, Clone)]
pub struct ListRef<'a, T, L = u32>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    list: &'a List<T, L>,
}

impl<'a, T, L> Deref for ListRef<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        self.list
    }
}

impl<'a, T, L> ListRef<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    #[must_use]
    pub fn inner(&self) -> &'a List<T, L> {
        self.list
    }
}

impl<'a, T, L> FrameworkSerialize for ListRef<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        (&self.len).to_bytes(output)?;
        for item in checked::cast_slice::<_, T>(&self.bytes) {
            item.to_bytes(output)?;
        }
        Ok(())
    }
}

unsafe impl<'a, T, L> FrameworkFromBytes<'a> for ListRef<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self> {
        let len = { from_bytes::<PackedValue<L>>(&bytes[..size_of::<L>()]).0 }
            .to_usize()
            .unwrap();
        Ok(Self {
            list: unsafe {
                &*ptr::from_raw_parts(
                    bytes
                        .try_advance(size_of::<L>() + size_of::<T>() * len)?
                        .as_ptr()
                        .cast(),
                    len * size_of::<T>(),
                )
            },
        })
    }
}

impl<'a, T, L> PointerBreakup for ListRef<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Metadata = L;

    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (NonNull::from(&self.list).cast(), self.list.len.0)
    }
}

impl<'a, T, L> BuildPointer for ListRef<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    unsafe fn build_pointer(pointee: NonNull<()>, metadata: Self::Metadata) -> Self {
        Self {
            list: unsafe {
                &*ptr::from_raw_parts(
                    pointee.as_ptr(),
                    metadata.to_usize().unwrap() * size_of::<T>(),
                )
            },
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ListRefMut<'a, T, L = u32>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    phantom_ref: PhantomData<&'a mut [T]>,
    ptr: NonNull<()>,
    metadata: L,
    #[derivative(Debug = "ignore")]
    resize: Box<dyn ResizeFn<'a, <Self as PointerBreakup>::Metadata>>,
}

impl<'a, T, L> Deref for ListRefMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &*ptr::from_raw_parts(
                self.ptr.as_ptr(),
                self.metadata.to_usize().unwrap() * size_of::<T>(),
            )
        }
    }
}

impl<'a, T, L> DerefMut for ListRefMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *ptr::from_raw_parts_mut(
                self.ptr.as_ptr(),
                self.metadata.to_usize().unwrap() * size_of::<T>(),
            )
        }
    }
}

impl<'a, T, L> FrameworkSerialize for ListRefMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        (&self.len).to_bytes(output)?;
        for item in checked::cast_slice::<_, T>(&self.bytes) {
            item.to_bytes(output)?;
        }
        Ok(())
    }
}

unsafe impl<'a, T, L> FrameworkFromBytesMut<'a> for ListRefMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self> {
        let len = from_bytes::<PackedValue<L>>(&bytes[..size_of::<L>()]).0;
        let len_usize = len.to_usize().unwrap();
        let ptr =
            NonNull::from(bytes.try_advance(size_of::<L>() + size_of::<T>() * len_usize)?).cast();
        Ok(Self {
            phantom_ref: PhantomData,
            ptr,
            metadata: len,
            resize: Box::new(resize),
        })
    }
}

impl<'a, T, L> PointerBreakup for ListRefMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Metadata = L;

    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (self.ptr, self.metadata)
    }
}

impl<'a, T, L> BuildPointerMut<'a> for ListRefMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    unsafe fn build_pointer_mut(
        pointee: NonNull<()>,
        metadata: Self::Metadata,
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Self {
        Self {
            phantom_ref: PhantomData,
            ptr: pointee,
            metadata,
            resize: Box::new(resize),
        }
    }
}

impl<'a, T, L> ListRefMut<'a, T, L>
where
    T: CheckedBitPattern + NoUninit + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    pub fn push(&mut self, item: T) -> Result<()> {
        self.push_all([item])
    }

    pub fn push_all<I>(&mut self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.insert_all(self.len(), iter)
    }

    pub fn insert(&mut self, index: usize, item: T) -> Result<()> {
        self.insert_all(index, [item])
    }

    pub fn insert_all<I>(&mut self, index: usize, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let old_len = self.len();
        if index > old_len {
            // TODO: Better errors
            bail!(ProgramError::InvalidArgument);
        }
        let iter = iter.into_iter();
        let new_len = self
            .metadata
            .to_usize()
            .unwrap()
            .checked_add(iter.len())
            .ok_or(ProgramError::InvalidArgument)?;
        self.len.0 = L::from_usize(new_len).unwrap();
        self.metadata = self.len.0;
        let new_ptr = (self.resize)(size_of::<L>() + new_len * size_of::<T>(), self.metadata)?;
        self.ptr = new_ptr;
        unsafe {
            sol_memmove(
                self.bytes
                    .as_mut_ptr()
                    .add((index + iter.len()) * size_of::<T>()),
                self.bytes.as_mut_ptr().add(index * size_of::<T>()),
                (old_len - index) * size_of::<T>(),
            );
        }
        for (i, item) in iter.enumerate() {
            let slot = index + i;
            self.bytes[slot * size_of::<T>()..][..size_of::<T>()].copy_from_slice(bytes_of(&item));
        }

        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..=index)
    }

    pub fn remove_range(&mut self, range: impl RangeBounds<usize>) -> Result<()> {
        let old_len = self.len();
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start.checked_add(1).ok_or(ProgramError::InvalidArgument)?,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => end.checked_add(1).ok_or(ProgramError::InvalidArgument)?,
            Bound::Excluded(end) => *end,
            Bound::Unbounded => old_len,
        };
        if start > end || end > old_len {
            bail!(ProgramError::InvalidArgument);
        }

        unsafe {
            sol_memmove(
                self.bytes.as_mut_ptr().add(start * size_of::<T>()).cast(),
                self.bytes.as_mut_ptr().add(end * size_of::<T>()).cast(),
                (old_len - end) * size_of::<T>(),
            );
        }
        let new_len = self.metadata.to_usize().unwrap() - (end - start);
        self.len.0 = L::from_usize(new_len).unwrap();
        self.metadata = self.len.0;
        let new_ptr = (self.resize)(size_of::<L>() + new_len * size_of::<T>(), self.metadata)?;
        self.ptr = new_ptr;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.remove_range(..)
    }

    pub fn retain(&mut self, mut op: impl FnMut(&mut T) -> bool) -> Result<()> {
        self.try_retain(|item| Ok(op(item)))
    }

    /// Retains any elements for which `op` returns `true`.
    /// Elements that return `false` will be removed.
    pub fn try_retain(&mut self, mut op: impl FnMut(&mut T) -> Result<bool>) -> Result<()> {
        let mut removal_start = 0;
        let mut index = 0;
        while index < self.len() {
            let result = op(&mut self[index])?;
            if result {
                if removal_start != index {
                    self.remove_range(removal_start..index)?;
                }
                removal_start = index + 1;
            }
            index += 1;
        }
        if removal_start < self.len() {
            self.remove_range(removal_start..)?;
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use crate::align1::Align1;
    use crate::serialize::list::List;
    use crate::serialize::test::TestByteSet;
    use crate::Result;
    use bytemuck::{Pod, Zeroable};
    use std::ops::Deref;

    #[derive(Pod, Zeroable, Copy, Clone, Eq, PartialEq, Debug, Align1)]
    #[repr(C, packed)]
    pub struct Cool {
        pub a: u8,
        pub b: u8,
    }

    #[test]
    fn test_stuff() -> Result<()> {
        let mut test_bytes = TestByteSet::<List<Cool>>::new(())?;
        assert_eq!(*test_bytes.immut()?, &[]);
        assert_eq!(*test_bytes.mutable()?, &[]);
        test_bytes.mutable()?.push(Cool { a: 1, b: 1 })?;
        assert_eq!(
            *test_bytes.immut()?,
            &[Cool { a: 1, b: 1 }],
            "bytes: {:?}",
            test_bytes.bytes
        );
        assert_eq!(*test_bytes.mutable()?, &[Cool { a: 1, b: 1 }]);

        let first = test_bytes.immut()?[0];
        println!("Cool: {first:#?}");

        let mut mutable = test_bytes.mutable()?;
        mutable.push(Cool { a: 2, b: 2 })?;
        mutable.push(Cool { a: 3, b: 3 })?;
        assert_eq!(
            *mutable.deref(),
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 2, b: 2 },
                Cool { a: 3, b: 3 },
            ]
        );
        mutable.push_all((4..=6).map(|x| Cool { a: x, b: x }))?;
        assert_eq!(
            *mutable,
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 2, b: 2 },
                Cool { a: 3, b: 3 },
                Cool { a: 4, b: 4 },
                Cool { a: 5, b: 5 },
                Cool { a: 6, b: 6 },
            ]
        );
        mutable.remove_range(1..4)?;
        assert_eq!(
            *mutable,
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 5, b: 5 },
                Cool { a: 6, b: 6 },
            ]
        );
        mutable.push_all((7..=9).map(|x| Cool { a: x, b: x + 1 }))?;
        assert_eq!(
            *mutable,
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 5, b: 5 },
                Cool { a: 6, b: 6 },
                Cool { a: 7, b: 8 },
                Cool { a: 8, b: 9 },
                Cool { a: 9, b: 10 },
            ]
        );
        mutable.retain(|x| x.a == x.b)?;
        assert_eq!(
            *mutable,
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 5, b: 5 },
                Cool { a: 6, b: 6 },
            ]
        );
        mutable.retain(|x| x.a % 2 == 0)?;
        assert_eq!(*mutable, &[Cool { a: 6, b: 6 }]);
        mutable.insert(1, Cool { a: 1, b: 1 })?;
        mutable.insert(1, Cool { a: 2, b: 2 })?;
        mutable.insert(1, Cool { a: 3, b: 3 })?;

        drop(mutable);

        assert_eq!(
            *test_bytes.immut()?,
            &[
                Cool { a: 6, b: 6 },
                Cool { a: 3, b: 3 },
                Cool { a: 2, b: 2 },
                Cool { a: 1, b: 1 }
            ]
        );

        Ok(())
    }
}

pub mod eq_impls {
    use super::*;

    impl<T, L> PartialEq<[T]> for List<T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &[T]) -> bool {
            self.deref().eq(other)
        }
    }

    impl<'a, T, L> PartialEq<&'a [T]> for List<T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &&'a [T]) -> bool {
            self.deref().eq(other)
        }
    }

    impl<T, L, const N: usize> PartialEq<[T; N]> for List<T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &[T; N]) -> bool {
            self.deref().eq(other)
        }
    }

    impl<'a, T, L, const N: usize> PartialEq<&'a [T; N]> for List<T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &&'a [T; N]) -> bool {
            self.deref().eq(other.as_slice())
        }
    }

    impl<T, L, const N: usize> PartialEq<List<T, L>> for [T; N]
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &List<T, L>) -> bool {
            self.as_slice().eq(other)
        }
    }

    impl<'a, T, L, const N: usize> PartialEq<List<T, L>> for &'a [T; N]
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &List<T, L>) -> bool {
            self.as_slice().eq(other)
        }
    }

    // Ref
    impl<T, L> PartialEq<[T]> for ListRef<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &[T]) -> bool {
            self.deref().eq(&other)
        }
    }

    impl<'a, T, L> PartialEq<&'a [T]> for ListRef<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &&'a [T]) -> bool {
            self.deref().eq(other)
        }
    }

    impl<T, L, const N: usize> PartialEq<[T; N]> for ListRef<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &[T; N]) -> bool {
            self.deref().eq(&other)
        }
    }

    impl<'a, T, L, const N: usize> PartialEq<&'a [T; N]> for ListRef<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &&'a [T; N]) -> bool {
            self.deref().eq(&other.as_slice())
        }
    }

    impl<T, L, const N: usize> PartialEq<ListRef<'_, T, L>> for [T; N]
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &ListRef<'_, T, L>) -> bool {
            self.as_slice().eq(other)
        }
    }

    impl<'a, T, L, const N: usize> PartialEq<ListRef<'_, T, L>> for &'a [T; N]
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &ListRef<'_, T, L>) -> bool {
            self.as_slice().eq(other)
        }
    }

    // RefMut
    impl<T, L> PartialEq<[T]> for ListRefMut<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &[T]) -> bool {
            self.deref().eq(other)
        }
    }

    impl<'a, T, L> PartialEq<&'a [T]> for ListRefMut<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &&'a [T]) -> bool {
            self.deref().eq(other)
        }
    }

    impl<T, L, const N: usize> PartialEq<[T; N]> for ListRefMut<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &[T; N]) -> bool {
            self.deref().eq(other)
        }
    }

    impl<'a, T, L, const N: usize> PartialEq<&'a [T; N]> for ListRefMut<'_, T, L>
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &&'a [T; N]) -> bool {
            self.deref().eq(other.as_slice())
        }
    }

    impl<T, L, const N: usize> PartialEq<ListRefMut<'_, T, L>> for [T; N]
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &ListRefMut<'_, T, L>) -> bool {
            self.as_slice().eq(other)
        }
    }

    impl<'a, T, L, const N: usize> PartialEq<ListRefMut<'_, T, L>> for &'a [T; N]
    where
        T: Pod + Align1,
        L: Pod + ToPrimitive + FromPrimitive,
        [T]: PartialEq,
    {
        fn eq(&self, other: &ListRefMut<'_, T, L>) -> bool {
            self.as_slice().eq(other)
        }
    }
}
