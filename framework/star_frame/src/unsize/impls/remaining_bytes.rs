use crate::prelude::{
    AsBytes, FromBytesReturn, RefWrapper, RefWrapperMutExt, RefWrapperTypes, RemainingData,
};
use crate::unsize::{AsMutBytes, RefDeref, RefDerefMut, UnsizedType};
use star_frame_proc::Align1;
use typenum::True;

#[derive(Align1, Debug)]
#[repr(transparent)]
pub struct RemainingBytes([u8]);
unsafe impl UnsizedType for RemainingBytes {
    type RefMeta = ();
    type RefData = RemainingBytesRef;
    type Owned = RemainingData;
    type IsUnsized = True;

    fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        let bytes = AsBytes::as_bytes(&super_ref)?;
        Ok(FromBytesReturn {
            meta: (),
            bytes_used: bytes.len(),
            ref_wrapper: unsafe { RefWrapper::new(super_ref, RemainingBytesRef) },
        })
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
        Ok(r.to_vec().into())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RemainingBytesRef;

impl<S> RefDeref<S> for RemainingBytesRef
where
    S: AsBytes,
{
    type Target = [u8];

    fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
        // sup is the underlying bytes. They have already been advanced to only cover this type (and potentially adjacent types if they existed)
        // because bytes just consumes the rest of the bytes, we just take as bytes
        AsBytes::as_bytes(RefWrapper::sup(wrapper)).expect("Invalid Bytes")
    }
}

impl<S> RefDerefMut<S> for RemainingBytesRef
where
    S: AsMutBytes,
{
    fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
        unsafe { AsMutBytes::as_mut_bytes(RefWrapperMutExt::sup_mut(wrapper)) }
            .expect("Invalid bytes")
    }
}
