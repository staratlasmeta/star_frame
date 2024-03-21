use crate::align1::Align1;
use crate::prelude::UnsizedType;
use crate::serialize::ref_wrapper::{AsBytes, AsMutBytes, RefDeref, RefDerefMut, RefWrapper};
use crate::serialize::unsize::owned::UnsizedTypeToOwned;
use crate::serialize::unsize::unsized_type::FromBytesReturn;
use crate::Result;
use advance::Advance;
use bytemuck::checked::try_from_bytes;
use bytemuck::{CheckedBitPattern, NoUninit};
use derivative::Derivative;
use std::marker::PhantomData;
use std::mem::size_of;

impl<T> UnsizedType for T
where
    T: Align1 + CheckedBitPattern + NoUninit,
{
    type RefMeta = ();
    type RefData = CheckRef<T>;

    fn from_bytes<S: AsBytes>(bytes: S) -> Result<FromBytesReturn<S, Self::RefData>> {
        try_from_bytes::<Self>(bytes.as_bytes().try_advance(size_of::<T>())?)?;
        Ok(FromBytesReturn {
            bytes_used: size_of::<T>(),
            ref_wrapper: RefWrapper::new(bytes, CheckRef(PhantomData)),
        })
    }
}

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
        unsafe { &*wrapper.sup().as_bytes().as_ptr().cast() }
    }
}

impl<S, T> RefDerefMut<S> for CheckRef<T>
where
    S: AsMutBytes,
    T: Align1 + CheckedBitPattern + NoUninit,
{
    fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
        unsafe { &mut *wrapper.sup_mut().as_mut_bytes().as_mut_ptr().cast() }
    }
}

impl<T> UnsizedTypeToOwned for T
where
    T: Align1 + CheckedBitPattern + NoUninit,
{
    type Owned = T;

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Self::Owned {
        *r
    }
}
