use crate::serialize::pointer_breakup::{
    BuildPointer, BuildPointerMut, PointerBreakup, PointerBreakupMut,
};
use crate::serialize::serialize_with::SerializeWith;
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut, ResizeFn};
use advance::Advance;
use solana_program::program_memory::sol_memmove;
use star_frame::serialize::FrameworkSerialize;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;

pub struct CombinedUnsizedData<T: ?Sized, U: ?Sized> {
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
    _data: [u8],
}

impl<T, U> SerializeWith for CombinedUnsizedData<T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type RefMeta = <Self::Ref<'static> as PointerBreakup>::Metadata;
    type Ref<'a> = CombinedUnsizedDataRef<'a, T, U>;
    type RefMut<'a> = CombinedUnsizedDataRefMut<'a, T, U>;
}

#[derive(Copy, Clone)]
pub struct CombinedUnsizedMetadata<TMeta, UMeta> {
    data_len: usize,
    t_meta: TMeta,
    u_meta: UMeta,
    t_len: usize,
}

pub struct CombinedUnsizedDataRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    phantom_ref: PhantomData<&'a ()>,
    pointer: *const (),
    meta: CombinedUnsizedMetadata<
        <T::Ref<'static> as PointerBreakup>::Metadata,
        <U::Ref<'static> as PointerBreakup>::Metadata,
    >,
}
impl<'a, T, U> Deref for CombinedUnsizedDataRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Target = CombinedUnsizedData<T, U>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.pointer, self.meta.data_len) }
    }
}
impl<'a, T, U> FrameworkSerialize for CombinedUnsizedDataRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        let (t, u) = self.split();
        t.to_bytes(output)?;
        u.to_bytes(output)
    }
}
unsafe impl<'a, T, U> FrameworkFromBytes<'a> for CombinedUnsizedDataRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn from_bytes(bytes: &mut &'a [u8]) -> crate::Result<Self> {
        let mut bytes_clone = &**bytes;
        let t = T::Ref::from_bytes(&mut bytes_clone)?;
        let t_len = bytes.len() - bytes_clone.len();
        let u = U::Ref::from_bytes(&mut bytes_clone)?;
        let data_len = bytes.len() - bytes_clone.len();
        Ok(Self {
            phantom_ref: PhantomData,
            pointer: bytes.try_advance(data_len)?.as_ptr().cast(),
            meta: CombinedUnsizedMetadata {
                data_len,
                t_meta: t.break_pointer().1,
                u_meta: u.break_pointer().1,
                t_len,
            },
        })
    }
}
impl<'a, T, U> PointerBreakup for CombinedUnsizedDataRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Metadata = CombinedUnsizedMetadata<
        <T::Ref<'static> as PointerBreakup>::Metadata,
        <U::Ref<'static> as PointerBreakup>::Metadata,
    >;

    fn break_pointer(&self) -> (*const (), Self::Metadata) {
        (self.pointer, self.meta)
    }
}
impl<'a, T, U> BuildPointer for CombinedUnsizedDataRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    unsafe fn build_pointer(pointee: *const (), metadata: Self::Metadata) -> Self {
        Self {
            phantom_ref: PhantomData,
            pointer: pointee,
            meta: metadata,
        }
    }
}

pub struct CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    phantom_ref: PhantomData<&'a mut ()>,
    pointer: *mut (),
    meta: CombinedUnsizedMetadata<T::RefMeta, U::RefMeta>,
    resize: Box<dyn ResizeFn<'a, <Self as PointerBreakup>::Metadata>>,
}
impl<'a, T, U> Deref for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Target = CombinedUnsizedData<T, U>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.pointer, self.meta.data_len) }
    }
}
impl<'a, T, U> DerefMut for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *ptr::from_raw_parts_mut(self.pointer, self.meta.data_len) }
    }
}
impl<'a, T, U> FrameworkSerialize for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> crate::Result<()> {
        let (t, u) = self.split();
        t.to_bytes(output)?;
        u.to_bytes(output)
    }
}
unsafe impl<'a, T, U> FrameworkFromBytesMut<'a> for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> crate::Result<Self> {
        let bytes_len = bytes.len();
        let mut bytes_clone = &mut **bytes;
        let mut t = T::RefMut::from_bytes_mut(&mut bytes_clone, |_, _| {
            panic!("Cannot resize during `from_bytes`")
        })?;
        let t_meta = t.break_pointer_mut().1;
        let t_len = bytes_len - bytes_clone.len();
        drop(t);
        let mut u = U::RefMut::from_bytes_mut(&mut bytes_clone, |_, _| {
            panic!("Cannot resize during `from_bytes`")
        })?;
        let u_meta = u.break_pointer_mut().1;
        drop(u);
        let data_len = bytes_len - bytes_clone.len();
        Ok(Self {
            phantom_ref: PhantomData,
            pointer: bytes.try_advance(data_len)?.as_mut_ptr().cast(),
            meta: CombinedUnsizedMetadata {
                data_len,
                t_meta,
                u_meta,
                t_len,
            },
            resize: Box::new(resize),
        })
    }
}

impl<'a, T, U> PointerBreakup for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Metadata = CombinedUnsizedMetadata<T::RefMeta, U::RefMeta>;

    fn break_pointer(&self) -> (*const (), Self::Metadata) {
        (self.pointer, self.meta)
    }
}

impl<'a, T, U> PointerBreakupMut for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn break_pointer_mut(&mut self) -> (*mut (), Self::Metadata) {
        (self.pointer, self.meta)
    }
}

impl<'a, T, U> BuildPointerMut<'a> for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    unsafe fn build_pointer_mut(
        pointee: *mut (),
        metadata: Self::Metadata,
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Self {
        Self {
            phantom_ref: PhantomData,
            pointer: pointee,
            meta: metadata,
            resize: Box::new(resize),
        }
    }
}

pub trait CombinedRefAccess<T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn t(&self) -> T::Ref<'_>;
    fn u(&self) -> U::Ref<'_>;
    fn split(&self) -> (T::Ref<'_>, U::Ref<'_>) {
        (self.t(), self.u())
    }
}
impl<'a, T, U> CombinedRefAccess<T, U> for CombinedUnsizedDataRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn t(&self) -> T::Ref<'_> {
        let (pointer, meta) = self.break_pointer();
        unsafe { T::Ref::build_pointer(pointer, meta.t_meta) }
    }

    fn u(&self) -> U::Ref<'_> {
        let (pointer, meta) = self.break_pointer();
        unsafe { U::Ref::build_pointer(pointer.byte_add(meta.t_len), meta.u_meta) }
    }
}
impl<'a, T, U> CombinedRefAccess<T, U> for CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn t(&self) -> T::Ref<'_> {
        let (pointer, meta) = self.break_pointer();
        unsafe { T::Ref::build_pointer(pointer, meta.t_meta) }
    }

    fn u(&self) -> U::Ref<'_> {
        let (pointer, meta) = self.break_pointer();
        unsafe { U::Ref::build_pointer(pointer.byte_add(meta.t_len), meta.u_meta) }
    }
}

impl<'a, T, U> CombinedUnsizedDataRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    pub fn t_mut(&mut self) -> T::RefMut<'_> {
        let (pointer, meta) = self.break_pointer_mut();
        unsafe {
            T::RefMut::build_pointer_mut(pointer, meta.t_meta, move |new_size, new_meta| {
                let old_t_len = self.meta.t_len;
                match old_t_len.cmp(&new_size) {
                    Ordering::Equal => {
                        self.meta.t_meta = new_meta;
                        (self.resize)(self.meta.data_len, self.meta)
                    }
                    // Old size less than new size
                    Ordering::Less => {
                        self.meta.t_meta = new_meta;
                        self.meta.t_len = new_size;
                        self.meta.data_len += new_size - old_t_len;
                        (self.resize)(self.meta.data_len, self.meta)?;
                        sol_memmove(
                            self.pointer.byte_add(new_size).cast(),
                            self.pointer.byte_add(old_t_len).cast(),
                            self.meta.data_len - new_size,
                        );
                        Ok(())
                    }
                    // Old size greater than new size
                    Ordering::Greater => {
                        sol_memmove(
                            self.pointer.byte_add(new_size).cast(),
                            self.pointer.byte_add(old_t_len).cast(),
                            self.meta.data_len - old_t_len,
                        );
                        self.meta.t_meta = new_meta;
                        self.meta.t_len = new_size;
                        self.meta.data_len -= old_t_len - new_size;
                        (self.resize)(self.meta.data_len, self.meta)
                    }
                }
            })
        }
    }

    pub fn u_mut(&mut self) -> U::RefMut<'_> {
        let (pointer, meta) = self.break_pointer_mut();
        unsafe {
            U::RefMut::build_pointer_mut(
                pointer.byte_add(meta.t_len),
                meta.u_meta,
                move |new_size, new_meta| {
                    let new_data_len = new_size + meta.t_len;
                    self.meta.u_meta = new_meta;
                    self.meta.data_len = new_data_len;
                    (self.resize)(new_data_len, self.meta)
                },
            )
        }
    }

    pub fn split_mut(&mut self) -> (T::Ref<'_>, U::RefMut<'_>) {
        let (pointer, meta) = self.break_pointer_mut();
        (
            unsafe { T::Ref::build_pointer(pointer, meta.t_meta) },
            unsafe {
                U::RefMut::build_pointer_mut(
                    pointer.byte_add(meta.t_len),
                    meta.u_meta,
                    move |new_size, new_meta| {
                        let new_data_len = new_size + meta.t_len;
                        self.meta.u_meta = new_meta;
                        self.meta.data_len = new_data_len;
                        (self.resize)(new_data_len, self.meta)
                    },
                )
            },
        )
    }
}
