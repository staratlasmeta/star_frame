use crate::align1::Align1;
use crate::packed_value::PackedValue;
use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut, PointerBreakup};
use crate::serialize::unsized_type::UnsizedType;
use crate::serialize::{
    FrameworkFromBytes, FrameworkFromBytesMut, FrameworkInit, FrameworkSerialize, ResizeFn,
};
use crate::Result;
use advance::Advance;
use bytemuck::{from_bytes, Pod};
use derivative::Derivative;
use num_traits::{FromPrimitive, ToPrimitive, Zero};
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memmove;
use std::collections::Bound;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut, RangeBounds};
use std::ptr;
use std::ptr::NonNull;

#[derive(Align1, Debug)]
#[repr(C)]
pub struct List<T, L = u32>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    len: PackedValue<L>,
    items: [T],
}
unsafe impl<T, L> FrameworkInit<()> for List<T, L>
where
    T: Pod + Align1,
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
        &self.items
    }
}
impl<T, L> DerefMut for List<T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T, L> UnsizedType for List<T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type RefMeta = L;
    type Ref<'a> = ListRef<'a, T, L>;
    type RefMut<'a> = ListRefMut<'a, T, L>;
}

#[derive(Debug, Copy, Clone)]
pub struct ListRef<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    list: &'a List<T, L>,
}
impl<'a, T, L> Deref for ListRef<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        self.list
    }
}
impl<'a, T, L> FrameworkSerialize for ListRef<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        (&self.len).to_bytes(output)?;
        for item in self.items.iter() {
            item.to_bytes(output)?;
        }
        Ok(())
    }
}
unsafe impl<'a, T, L> FrameworkFromBytes<'a> for ListRef<'a, T, L>
where
    T: Pod + Align1,
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
                    len,
                )
            },
        })
    }
}
impl<'a, T, L> PointerBreakup for ListRef<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Metadata = L;

    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (NonNull::from(&self.list).cast(), self.list.len.0)
    }
}
impl<'a, T, L> BuildPointer for ListRef<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    unsafe fn build_pointer(pointee: NonNull<()>, metadata: Self::Metadata) -> Self {
        Self {
            list: unsafe { &*ptr::from_raw_parts(pointee.as_ptr(), metadata.to_usize().unwrap()) },
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ListRefMut<'a, T, L>
where
    T: Pod + Align1,
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
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.ptr.as_ptr(), self.metadata.to_usize().unwrap()) }
    }
}
impl<'a, T, L> DerefMut for ListRefMut<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *ptr::from_raw_parts_mut(self.ptr.as_ptr(), self.metadata.to_usize().unwrap())
        }
    }
}
impl<'a, T, L> FrameworkSerialize for ListRefMut<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        (&self.len).to_bytes(output)?;
        for item in self.items.iter() {
            item.to_bytes(output)?;
        }
        Ok(())
    }
}
unsafe impl<'a, T, L> FrameworkFromBytesMut<'a> for ListRefMut<'a, T, L>
where
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self> {
        let len = *from_bytes::<L>(&bytes[..size_of::<L>()]);
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
    T: Pod + Align1,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Metadata = L;

    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (self.ptr, self.metadata)
    }
}
impl<'a, T, L> BuildPointerMut<'a> for ListRefMut<'a, T, L>
where
    T: Pod + Align1,
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
    T: Pod + Align1,
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
            return Err(ProgramError::InvalidArgument);
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
                self.items.as_mut_ptr().add(index + iter.len()).cast(),
                self.items.as_mut_ptr().add(index).cast(),
                (old_len - index) * size_of::<T>(),
            );
        }
        for (i, item) in iter.enumerate() {
            self.items[index + i] = item;
        }

        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<()> {
        self.remove_range(index..index + 1)
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
            return Err(ProgramError::InvalidArgument);
        }

        unsafe {
            sol_memmove(
                self.items.as_mut_ptr().add(start).cast(),
                self.items.as_mut_ptr().add(end).cast(),
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
        assert_eq!(test_bytes.immut()?.deref().deref(), &[]);
        assert_eq!(test_bytes.mutable()?.deref().deref(), &[]);
        test_bytes.mutable()?.push(Cool { a: 1, b: 1 })?;
        assert_eq!(
            test_bytes.immut()?.deref().deref(),
            &[Cool { a: 1, b: 1 }],
            "bytes: {:?}",
            test_bytes.bytes
        );
        assert_eq!(
            test_bytes.mutable()?.deref().deref(),
            &[Cool { a: 1, b: 1 }]
        );

        let mut mutable = test_bytes.mutable()?;
        mutable.push(Cool { a: 2, b: 2 })?;
        mutable.push(Cool { a: 3, b: 3 })?;
        assert_eq!(
            mutable.deref().deref(),
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 2, b: 2 },
                Cool { a: 3, b: 3 },
            ]
        );
        mutable.push_all((4..=6).map(|x| Cool { a: x, b: x }))?;
        assert_eq!(
            mutable.deref().deref(),
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
            mutable.deref().deref(),
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 5, b: 5 },
                Cool { a: 6, b: 6 },
            ]
        );
        mutable.push_all((7..=9).map(|x| Cool { a: x, b: x + 1 }))?;
        assert_eq!(
            mutable.deref().deref(),
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
            mutable.deref().deref(),
            &[
                Cool { a: 1, b: 1 },
                Cool { a: 5, b: 5 },
                Cool { a: 6, b: 6 },
            ]
        );
        mutable.retain(|x| x.a % 2 == 0)?;
        assert_eq!(mutable.deref().deref(), &[Cool { a: 6, b: 6 }]);
        drop(mutable);

        assert_eq!(test_bytes.immut()?.deref().deref(), &[Cool { a: 6, b: 6 }]);

        Ok(())
    }
}
