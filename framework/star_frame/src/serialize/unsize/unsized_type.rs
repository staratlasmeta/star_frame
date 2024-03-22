use crate::align1::Align1;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use crate::Result;
pub use star_frame_proc::unsized_type;

/// # Safety
/// [`UnsizedType::from_bytes`] must return correct values.
pub unsafe trait UnsizedType: 'static + Align1 {
    type RefMeta: 'static + Copy;
    type RefData;

    fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>>;
}

#[derive(Debug, Copy, Clone)]
pub struct FromBytesReturn<S, R, M> {
    pub bytes_used: usize,
    pub meta: M,
    pub ref_wrapper: RefWrapper<S, R>,
}
