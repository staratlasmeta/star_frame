use crate::prelude::{AsBytes, FromBytesReturn, RefWrapper, RefWrapperMutExt, RefWrapperTypes};
use crate::unsize::{AsMutBytes, RefDeref, RefDerefMut, UnsizedType};
use star_frame_proc::Align1;
use typenum::True;

#[derive(Align1, Debug)]
#[repr(transparent)]
pub struct Bytes([u8]);
unsafe impl UnsizedType for Bytes {
    type RefMeta = ();
    type RefData = BytesRef;
    type Owned = Vec<u8>;
    type IsUnsized = True;

    fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        let bytes = super_ref.as_bytes()?;
        Ok(FromBytesReturn {
            meta: (),
            bytes_used: bytes.len(),
            ref_wrapper: unsafe { RefWrapper::new(super_ref, BytesRef) },
        })
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
        Ok(r.to_vec())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BytesRef;

impl<S> RefDeref<S> for BytesRef
where
    S: AsBytes,
{
    type Target = [u8];

    fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target {
        // sup is the underlying bytes. They have already been advanced to only cover this type (and potentially adjacent types if they existed)
        // because bytes just consumes the rest of the bytes, we just take as bytes
        wrapper.sup().as_bytes().expect("Invalid Bytes")
    }
}

impl<S> RefDerefMut<S> for BytesRef
where
    S: AsMutBytes,
{
    fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target {
        unsafe { wrapper.sup_mut() }
            .as_mut_bytes()
            .expect("Invalid bytes")
    }
}
