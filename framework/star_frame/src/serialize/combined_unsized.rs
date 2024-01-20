use crate::serialize::pointer_breakup::{BuildPointer, BuildPointerMut, PointerBreakup};
use crate::serialize::serialize_with::SerializeWith;
use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut, ResizeFn};
use advance::Advance;
use solana_program::program_memory::sol_memmove;
use star_frame::serialize::FrameworkSerialize;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::NonNull;

pub struct CombinedUnsized<T: ?Sized, U: ?Sized> {
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
    _data: [u8],
}

impl<T, U> SerializeWith for CombinedUnsized<T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type RefMeta = CombinedUnsizedMetadata<T::RefMeta, U::RefMeta>;
    type Ref<'a> = CombinedUnsizedRef<'a, T, U> where Self: 'a;
    type RefMut<'a> = CombinedUnsizedRefMut<'a, T, U> where Self: 'a;
}

#[derive(Copy, Clone)]
pub struct CombinedUnsizedMetadata<TMeta, UMeta> {
    data_len: usize,
    t_meta: TMeta,
    u_meta: UMeta,
    t_len: usize,
}

pub struct CombinedUnsizedRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    phantom_ref: PhantomData<&'a ()>,
    pointer: NonNull<()>,
    meta: CombinedUnsizedMetadata<T::RefMeta, U::RefMeta>,
}
impl<'a, T, U> Deref for CombinedUnsizedRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Target = CombinedUnsized<T, U>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.pointer.as_ptr(), self.meta.data_len) }
    }
}
impl<'a, T, U> FrameworkSerialize for CombinedUnsizedRef<'a, T, U>
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
unsafe impl<'a, T, U> FrameworkFromBytes<'a> for CombinedUnsizedRef<'a, T, U>
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
            pointer: NonNull::from(bytes.try_advance(data_len)?).cast(),
            meta: CombinedUnsizedMetadata {
                data_len,
                t_meta: t.break_pointer().1,
                u_meta: u.break_pointer().1,
                t_len,
            },
        })
    }
}
impl<'a, T, U> PointerBreakup for CombinedUnsizedRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Metadata = CombinedUnsizedMetadata<
        <T::Ref<'static> as PointerBreakup>::Metadata,
        <U::Ref<'static> as PointerBreakup>::Metadata,
    >;

    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (self.pointer, self.meta)
    }
}
impl<'a, T, U> BuildPointer for CombinedUnsizedRef<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    unsafe fn build_pointer(pointee: NonNull<()>, metadata: Self::Metadata) -> Self {
        Self {
            phantom_ref: PhantomData,
            pointer: pointee,
            meta: metadata,
        }
    }
}

pub struct CombinedUnsizedRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    phantom_ref: PhantomData<&'a mut ()>,
    pointer: NonNull<()>,
    meta: CombinedUnsizedMetadata<T::RefMeta, U::RefMeta>,
    resize: Box<dyn ResizeFn<'a, <Self as PointerBreakup>::Metadata>>,
}
impl<'a, T, U> Deref for CombinedUnsizedRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Target = CombinedUnsized<T, U>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*ptr::from_raw_parts(self.pointer.as_ptr(), self.meta.data_len) }
    }
}
impl<'a, T, U> DerefMut for CombinedUnsizedRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *ptr::from_raw_parts_mut(self.pointer.as_ptr(), self.meta.data_len) }
    }
}
impl<'a, T, U> FrameworkSerialize for CombinedUnsizedRefMut<'a, T, U>
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
unsafe impl<'a, T, U> FrameworkFromBytesMut<'a> for CombinedUnsizedRefMut<'a, T, U>
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
        let t = T::RefMut::from_bytes_mut(&mut bytes_clone, |_, _| {
            panic!("Cannot resize during `from_bytes`")
        })?;
        let t_meta = t.break_pointer().1;
        let t_len = bytes_len - bytes_clone.len();
        drop(t);
        let u = U::RefMut::from_bytes_mut(&mut bytes_clone, |_, _| {
            panic!("Cannot resize during `from_bytes`")
        })?;
        let u_meta = u.break_pointer().1;
        drop(u);
        let data_len = bytes_len - bytes_clone.len();
        Ok(Self {
            phantom_ref: PhantomData,
            pointer: NonNull::from(bytes.try_advance(data_len)?).cast(),
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

impl<'a, T, U> PointerBreakup for CombinedUnsizedRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    type Metadata = CombinedUnsizedMetadata<T::RefMeta, U::RefMeta>;

    fn break_pointer(&self) -> (NonNull<()>, Self::Metadata) {
        (self.pointer, self.meta)
    }
}

impl<'a, T, U> BuildPointerMut<'a> for CombinedUnsizedRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    unsafe fn build_pointer_mut(
        pointee: NonNull<()>,
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
impl<'a, T, U> CombinedRefAccess<T, U> for CombinedUnsizedRef<'a, T, U>
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
        unsafe {
            U::Ref::build_pointer(
                NonNull::new(pointer.as_ptr().byte_add(meta.t_len)).unwrap(),
                meta.u_meta,
            )
        }
    }
}
impl<'a, T, U> CombinedRefAccess<T, U> for CombinedUnsizedRefMut<'a, T, U>
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
        unsafe {
            U::Ref::build_pointer(
                NonNull::new(pointer.as_ptr().byte_add(meta.t_len)).unwrap(),
                meta.u_meta,
            )
        }
    }
}

impl<'a, T, U> CombinedUnsizedRefMut<'a, T, U>
where
    T: ?Sized + SerializeWith,
    U: ?Sized + SerializeWith,
{
    pub fn t_mut(&mut self) -> T::RefMut<'_> {
        let (pointer, meta) = Self::break_pointer(self);
        unsafe {
            T::RefMut::build_pointer_mut(pointer, meta.t_meta, move |new_size, new_meta| {
                let old_t_len = self.meta.t_len;
                match old_t_len.cmp(&new_size) {
                    Ordering::Equal => {
                        self.meta.t_meta = new_meta;
                        let new_ptr = (self.resize)(self.meta.data_len, self.meta)?;
                        self.pointer = new_ptr;
                        Ok(new_ptr)
                    }
                    // Old size less than new size
                    Ordering::Less => {
                        self.meta.t_meta = new_meta;
                        self.meta.t_len = new_size;
                        self.meta.data_len += new_size - old_t_len;
                        let new_ptr = (self.resize)(self.meta.data_len, self.meta)?;
                        self.pointer = new_ptr;
                        sol_memmove(
                            self.pointer.as_ptr().byte_add(new_size).cast(),
                            self.pointer.as_ptr().byte_add(old_t_len).cast(),
                            self.meta.data_len - new_size,
                        );
                        Ok(new_ptr)
                    }
                    // Old size greater than new size
                    Ordering::Greater => {
                        sol_memmove(
                            self.pointer.as_ptr().byte_add(new_size).cast(),
                            self.pointer.as_ptr().byte_add(old_t_len).cast(),
                            self.meta.data_len - old_t_len,
                        );
                        self.meta.t_meta = new_meta;
                        self.meta.t_len = new_size;
                        self.meta.data_len -= old_t_len - new_size;
                        let new_ptr = (self.resize)(self.meta.data_len, self.meta)?;
                        self.pointer = new_ptr;
                        Ok(new_ptr)
                    }
                }
            })
        }
    }

    pub fn u_mut(&mut self) -> U::RefMut<'_> {
        let (pointer, meta) = Self::break_pointer(self);
        unsafe {
            U::RefMut::build_pointer_mut(
                NonNull::new(pointer.as_ptr().byte_add(meta.t_len)).unwrap(),
                meta.u_meta,
                move |new_size, new_meta| {
                    let new_data_len = new_size + meta.t_len;
                    self.meta.u_meta = new_meta;
                    self.meta.data_len = new_data_len;
                    let new_ptr = (self.resize)(new_data_len, self.meta)?;
                    self.pointer = new_ptr;
                    Ok(NonNull::new(new_ptr.as_ptr().byte_add(meta.t_len)).unwrap())
                },
            )
        }
    }

    pub fn split_mut(&mut self) -> (T::Ref<'_>, U::RefMut<'_>) {
        let (pointer, meta) = Self::break_pointer(self);
        (
            unsafe { T::Ref::build_pointer(pointer, meta.t_meta) },
            unsafe {
                U::RefMut::build_pointer_mut(
                    NonNull::new(pointer.as_ptr().byte_add(meta.t_len)).unwrap(),
                    meta.u_meta,
                    move |new_size, new_meta| {
                        let new_data_len = new_size + meta.t_len;
                        self.meta.u_meta = new_meta;
                        self.meta.data_len = new_data_len;
                        let new_ptr = (self.resize)(new_data_len, self.meta)?;
                        self.pointer = new_ptr;
                        Ok(NonNull::new(new_ptr.as_ptr().byte_add(meta.t_len)).unwrap())
                    },
                )
            },
        )
    }
}

#[cfg(test)]
mod test {
    use crate::packed_value::PackedValue;
    use crate::serialize::combined_unsized::{CombinedRefAccess, CombinedUnsized};
    use crate::serialize::list::test::Cool;
    use crate::serialize::list::List;
    use crate::Result;
    use star_frame::serialize::test::TestByteSet;
    use std::ops::Deref;

    #[test]
    fn test_combined() -> Result<()> {
        let mut test_bytes = TestByteSet::<CombinedUnsized<List<Cool>, List<u8>>>::new(8);
        assert_eq!(test_bytes.immut()?.t().deref().deref(), &[]);
        assert_eq!(test_bytes.immut()?.u().deref().deref(), &[] as &[u8]);

        let mut mutable = test_bytes.mutable()?;
        mutable.t_mut().push(Cool { a: 1, b: 1 })?;
        mutable.u_mut().push_all([1, 2, 3])?;
        assert_eq!(mutable.t().deref().deref(), &[Cool { a: 1, b: 1 }]);
        assert_eq!(mutable.u().deref().deref(), &[1, 2, 3]);
        drop(mutable);
        // println!("bytes: {:?}", test_bytes.bytes);
        // println!(
        //     "list1: {:#?}",
        //     test_bytes.immut()?.split().0.deref().deref()
        // );
        // println!(
        //     "list2: {:#?}",
        //     test_bytes.immut()?.split().1.deref().deref()
        // );
        Ok(())
    }

    #[test]
    fn test_combined_recursive() -> Result<()> {
        let mut test_bytes = TestByteSet::<
            CombinedUnsized<List<Cool>, CombinedUnsized<List<u8>, List<PackedValue<u16>>>>,
        >::new(12);
        assert_eq!(test_bytes.immut()?.t().deref().deref(), &[]);
        assert_eq!(test_bytes.immut()?.u().t().deref().deref(), &[] as &[u8]);
        assert_eq!(test_bytes.immut()?.u().u().deref().deref(), &[]);

        let mut mutable = test_bytes.mutable()?;
        mutable.t_mut().push(Cool { a: 1, b: 1 })?;
        mutable.u_mut().t_mut().push_all([1, 2, 3])?;
        mutable
            .u_mut()
            .u_mut()
            .push_all([PackedValue(1), PackedValue(2)])?;
        assert_eq!(mutable.t().deref().deref(), &[Cool { a: 1, b: 1 }]);
        assert_eq!(mutable.u().t().deref().deref(), &[1, 2, 3]);
        assert_eq!(
            mutable.u().u().deref().deref(),
            &[PackedValue(1), PackedValue(2)]
        );
        drop(mutable);
        // println!("bytes: {:?}", test_bytes.bytes);
        // println!(
        //     "list1: {:#?}",
        //     test_bytes.immut()?.split().0.deref().deref()
        // );
        // println!(
        //     "list2: {:#?}",
        //     test_bytes.immut()?.split().1.t().deref().deref()
        // );
        // println!(
        //     "list3: {:#?}",
        //     test_bytes.immut()?.split().1.u().deref().deref()
        // );
        Ok(())
    }
}
