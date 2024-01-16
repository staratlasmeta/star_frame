use crate::align1::Align1;
use crate::packed_value::PackedValue;
use crate::serialize::pointer_breakup::{
    BuildPointer, BuildPointerMut, PointerBreakup, PointerBreakupMut,
};
use crate::serialize::serialize_with::SerializeWith;
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut, FrameworkSerialize, ResizeFn};
use crate::Result;
use advance::Advance;
use bytemuck::{from_bytes, Pod};
use num_traits::{FromPrimitive, ToPrimitive};
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memmove;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr;

#[derive(Align1)]
#[repr(C)]
pub struct List<T, L = u32>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    len: PackedValue<L>,
    items: [PackedValue<T>],
}
impl<T, L> Deref for List<T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = [PackedValue<T>];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}
impl<T, L> DerefMut for List<T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T, L> SerializeWith for List<T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type RefMeta = L;
    type Ref<'a> = ListRef<'a, T, L> where Self: 'a;
    type RefMut<'a> = ListRefMut<'a, T, L> where Self: 'a;
}

pub struct ListRef<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    list: &'a List<T, L>,
}
impl<'a, T, L> Deref for ListRef<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        self.list
    }
}
impl<'a, T, L> FrameworkSerialize for ListRef<'a, T, L>
where
    T: Pod,
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
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self> {
        let len = from_bytes::<L>(&bytes[..size_of::<L>()])
            .to_usize()
            .unwrap();
        Ok(Self {
            list: unsafe {
                &*ptr::from_raw_parts(
                    bytes
                        .try_advance(size_of::<L>() + size_of::<PackedValue<T>>() * len)?
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
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Metadata = L;

    fn break_pointer(&self) -> (*const (), Self::Metadata) {
        ((self.list as *const List<T, L>).cast(), self.list.len.0)
    }
}
impl<'a, T, L> BuildPointer for ListRef<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    unsafe fn build_pointer(pointee: *const (), metadata: Self::Metadata) -> Self {
        Self {
            list: &*ptr::from_raw_parts(pointee.cast(), metadata.to_usize().unwrap()),
        }
    }
}

pub struct ListRefMut<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    phantom_ref: PhantomData<&'a mut [T]>,
    ptr: *mut (),
    metadata: L,
    resize: Box<dyn ResizeFn<'a, L>>,
}
impl<'a, T, L> Deref for ListRefMut<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Target = List<T, L>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.ptr.cast(), self.metadata.to_usize().unwrap()) }
    }
}
impl<'a, T, L> DerefMut for ListRefMut<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *ptr::from_raw_parts_mut(self.ptr.cast(), self.metadata.to_usize().unwrap()) }
    }
}
impl<'a, T, L> FrameworkSerialize for ListRefMut<'a, T, L>
where
    T: Pod,
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
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self> {
        let len = *from_bytes::<L>(&bytes[..size_of::<L>()]);
        let len_usize = len.to_usize().unwrap();
        let ptr = bytes
            .try_advance(size_of::<L>() + size_of::<PackedValue<T>>() * len_usize)?
            .as_mut_ptr()
            .cast();
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
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    type Metadata = L;

    fn break_pointer(&self) -> (*const (), Self::Metadata) {
        (self.ptr, self.metadata)
    }
}
impl<'a, T, L> PointerBreakupMut for ListRefMut<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    fn break_pointer_mut(&mut self) -> (*mut (), Self::Metadata) {
        (self.ptr, self.metadata)
    }
}
impl<'a, T, L> BuildPointerMut<'a> for ListRefMut<'a, T, L>
where
    T: Pod,
    L: Pod + ToPrimitive + FromPrimitive,
{
    unsafe fn build_pointer_mut(
        pointee: *mut (),
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
    T: Pod,
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
        (self.resize)(new_len * size_of::<PackedValue<T>>(), self.metadata)?;
        unsafe {
            sol_memmove(
                self.items.as_mut_ptr().add(index + iter.len()).cast(),
                self.items.as_mut_ptr().add(index).cast(),
                (old_len - index) * size_of::<PackedValue<T>>(),
            );
        }
        for (i, item) in iter.enumerate() {
            self.items[index + i] = PackedValue(item);
        }

        Ok(())
    }
}
