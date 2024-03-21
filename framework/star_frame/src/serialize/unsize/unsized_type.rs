use crate::align1::Align1;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use crate::Result;
pub use star_frame_proc::unsized_type;

pub trait UnsizedType: 'static + Align1 {
    type RefMeta: 'static + Copy;
    type RefData;

    fn from_bytes<S: AsBytes>(bytes: S) -> Result<FromBytesReturn<S, Self::RefData>>;
}

#[derive(Debug, Copy, Clone)]
pub struct FromBytesReturn<S, R> {
    pub bytes_used: usize,
    pub ref_wrapper: RefWrapper<S, R>,
}
