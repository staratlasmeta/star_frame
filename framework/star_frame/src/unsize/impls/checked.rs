use crate::align1::Align1;
use crate::unsize::*;
use crate::Result;
use advance::Advance;
use bytemuck::checked::try_from_bytes;
use bytemuck::{bytes_of, CheckedBitPattern, NoUninit, Zeroable};
use derivative::Derivative;
use std::marker::PhantomData;
use std::mem::size_of;
use typenum::False;

unsafe impl<T> UnsizedType for T
where
    T: Align1 + CheckedBitPattern + NoUninit,
{
    type RefMeta = ();
    type RefData = CheckRef<T>;
    type Owned = T;
    type IsUnsized = False;

    fn from_bytes<S: AsBytes>(
        bytes: S,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        try_from_bytes::<Self>(bytes.as_bytes()?.try_advance(size_of::<T>())?)?;
        Ok(FromBytesReturn {
            bytes_used: size_of::<T>(),
            meta: (),
            ref_wrapper: unsafe { RefWrapper::new(bytes, CheckRef(PhantomData)) },
        })
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned> {
        Ok(*r)
    }
}

/// A ref to a [`CheckedBitPattern`] value. Used in a [`RefWrapper`].
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""), Copy(bound = ""))]
pub struct CheckRef<T>(PhantomData<fn() -> T>);

impl<S, T> RefDeref<S> for CheckRef<T>
where
    S: AsBytes,
    T: Align1 + CheckedBitPattern + NoUninit,
{
    type Target = T;

    fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
        unsafe {
            &*wrapper
                .sup()
                .as_bytes()
                .expect("Invalid bytes")
                .as_ptr()
                .cast()
        }
    }
}

impl<S, T> RefDerefMut<S> for CheckRef<T>
where
    S: AsMutBytes,
    T: Align1 + CheckedBitPattern + NoUninit,
{
    fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
        unsafe {
            &mut *wrapper
                .sup_mut()
                .as_mut_bytes()
                .expect("Invalid bytes")
                .as_mut_ptr()
                .cast()
        }
    }
}

impl<T> UnsizedInit<T> for T
where
    T: Align1 + CheckedBitPattern + NoUninit,
{
    const INIT_BYTES: usize = size_of::<T>();

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        arg: T,
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        super_ref
            .as_mut_bytes()?
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytes_of(&arg));
        Ok((
            unsafe { RefWrapper::new(super_ref, CheckRef(PhantomData)) },
            (),
        ))
    }
}

impl<T> UnsizedInit<Zeroed> for T
where
    T: Align1 + CheckedBitPattern + NoUninit + Zeroable,
{
    const INIT_BYTES: usize = size_of::<T>();

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        _arg: Zeroed,
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        super_ref
            .as_mut_bytes()?
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytes_of(&T::zeroed()));
        Ok((
            unsafe { RefWrapper::new(super_ref, CheckRef(PhantomData)) },
            (),
        ))
    }
}
