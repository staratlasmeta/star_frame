use crate::prelude::*;
use crate::serialize::ref_wrapper::{AsMutBytes, RefWrapper};

pub type UnsizedInitReturn<S, U> = (
    RefWrapper<S, <U as UnsizedType>::RefData>,
    <U as UnsizedType>::RefMeta,
);

pub trait UnsizedInit<InitArg>: UnsizedType {
    const INIT_BYTES: usize;

    /// # Safety
    /// `super_ref` must have [`UnsizedInit::INIT_BYTES`] zeroed bytes.
    unsafe fn init<S: AsMutBytes>(super_ref: S, arg: InitArg)
        -> Result<UnsizedInitReturn<S, Self>>;
}

#[derive(Debug, Copy, Clone)]
pub struct Zeroed;
