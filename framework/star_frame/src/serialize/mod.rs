pub mod borsh;
pub mod combined_unsized;
pub mod list;
pub mod pointer_breakup;
pub mod unsized_enum;
pub mod unsized_type;

use crate::align1::Align1;
use crate::Result;
use advance::Advance;
use bytemuck::{from_bytes, from_bytes_mut, Pod};
use star_frame::serialize::pointer_breakup::PointerBreakup;
use std::mem::size_of;
use std::ptr::NonNull;

pub trait ResizeFn<'a, M>: FnMut(usize, M) -> Result<NonNull<()>> + 'a {}
impl<'a, T, M> ResizeFn<'a, M> for T where T: FnMut(usize, M) -> Result<NonNull<()>> + 'a {}

pub trait FrameworkSerialize {
    /// Writes this type to a set of bytes.
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()>;
}
impl<'a, T> FrameworkSerialize for &'a T
where
    T: Align1 + Pod,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        output
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(*self));
        Ok(())
    }
}
impl<'a, T> FrameworkSerialize for &'a mut T
where
    T: Align1 + Pod,
{
    fn to_bytes(&self, output: &mut &mut [u8]) -> Result<()> {
        output
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(*self));
        Ok(())
    }
}

/// Writes this type to a set of bytes and reads this type from bytes.
///
/// # Safety
/// If `Self` is pointer type [`from_bytes`](FrameworkFromBytes::from_bytes) must return the same pointer that was
/// passed in. Metadata may be different.
pub unsafe trait FrameworkFromBytes<'a>: Sized + FrameworkSerialize {
    /// Deserializes this type from a set of bytes.
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self>;
}

unsafe impl<'a, T> FrameworkFromBytes<'a> for &'a T
where
    T: Align1 + Pod,
{
    fn from_bytes(bytes: &mut &'a [u8]) -> Result<Self> {
        Ok(from_bytes(bytes.try_advance(size_of::<T>())?))
    }
}

/// Allows this type to be referenced mutably from a set of bytes.
///
/// # Safety
/// If `Self` is pointer type [`from_bytes_mut`](FrameworkFromBytesMut::from_bytes_mut) must return the same pointer that
/// was passed in. Metadata may be different.
pub unsafe trait FrameworkFromBytesMut<'a>:
    Sized + FrameworkSerialize + PointerBreakup
{
    /// Deserializes this type from a set of bytes mutably.
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self>;
}
unsafe impl<'a, T> FrameworkFromBytesMut<'a> for &'a mut T
where
    T: Align1 + Pod,
{
    fn from_bytes_mut(
        bytes: &mut &'a mut [u8],
        _resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self> {
        Ok(from_bytes_mut(bytes.try_advance(size_of::<T>())?))
    }
}

/// # Safety
/// [`init`](FrameworkInit::init) must properly initialize the bytes.
pub unsafe trait FrameworkInit<'a, A>: FrameworkFromBytesMut<'a> {
    /// Length of bytes required to initialize this type.
    const INIT_LENGTH: usize;
    /// Initializes this type with the given arguments.
    /// # Safety
    /// `bytes` must be zeroed and length [`INIT_LENGTH`](FrameworkInit::INIT_LENGTH).
    unsafe fn init(
        bytes: &'a mut [u8],
        arg: A,
        resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self>;
}
unsafe impl<'a, T> FrameworkInit<'a, ()> for &'a mut T
where
    T: Align1 + Pod,
{
    const INIT_LENGTH: usize = size_of::<T>();

    unsafe fn init(
        bytes: &'a mut [u8],
        _arg: (),
        _resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self> {
        debug_assert_eq!(bytes.len(), <Self as FrameworkInit<()>>::INIT_LENGTH);
        Ok(from_bytes_mut(bytes))
    }
}
unsafe impl<'a, T> FrameworkInit<'a, (T,)> for &'a mut T
where
    T: Align1 + Pod,
{
    const INIT_LENGTH: usize = size_of::<T>();

    unsafe fn init(
        bytes: &'a mut [u8],
        arg: (T,),
        _resize: impl ResizeFn<'a, Self::Metadata>,
    ) -> Result<Self> {
        debug_assert_eq!(bytes.len(), <Self as FrameworkInit<(T,)>>::INIT_LENGTH);
        let out = from_bytes_mut(bytes);
        *out = arg.0;
        Ok(out)
    }
}

#[cfg(test)]
pub mod test {
    use crate::serialize::unsized_type::UnsizedType;
    use crate::serialize::{FrameworkFromBytes, FrameworkFromBytesMut};
    use std::marker::PhantomData;
    use std::ptr::NonNull;

    #[derive(Debug)]
    pub struct TestByteSet<T: ?Sized> {
        pub bytes: Vec<u8>,
        pub phantom_t: PhantomData<T>,
    }
    impl<T: ?Sized> TestByteSet<T> {
        pub fn new(len: usize) -> Self {
            Self {
                bytes: vec![0; len],
                phantom_t: PhantomData,
            }
        }
    }
    impl<T> TestByteSet<T>
    where
        T: ?Sized + UnsizedType,
    {
        pub fn immut(&self) -> crate::Result<T::Ref<'_>> {
            T::Ref::from_bytes(&mut &self.bytes[..])
        }

        pub fn mutable(&mut self) -> crate::Result<T::RefMut<'_>> {
            let bytes = &mut self.bytes as *mut Vec<u8>;
            T::RefMut::from_bytes_mut(&mut &mut self.bytes[..], move |len, _| {
                let bytes = unsafe { &mut *bytes };
                bytes.resize(len, 0);
                Ok(NonNull::<[u8]>::from(bytes.as_slice()).cast())
            })
        }
    }
}
