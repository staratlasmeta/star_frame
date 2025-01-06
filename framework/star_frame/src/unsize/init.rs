use crate::prelude::*;
use crate::unsize::ref_wrapper::{AsMutBytes, RefWrapper};

/// The return type of [`UnsizedInit::init`].
/// Index `0` is the ref wrapper for the new type, index `1` is the meta.
pub type UnsizedInitReturn<S, U> = (
    RefWrapper<S, <U as UnsizedType>::RefData>,
    <U as UnsizedType>::RefMeta,
);

/// An [`UnsizedType`] that can be initialized with an `InitArg`. Must have a statically known size
/// (for arg type) at initialization.
pub trait UnsizedInit<InitArg>: UnsizedType {
    /// Amount of zeroed bytes this type takes to initialize.
    const INIT_BYTES: usize;

    /// # Safety
    /// `super_ref` must have [`UnsizedInit::INIT_BYTES`] zeroed bytes.
    unsafe fn init<S: AsMutBytes>(super_ref: S, arg: InitArg)
        -> Result<UnsizedInitReturn<S, Self>>;
}

/// Argument for initializing a type to a default value
#[derive(Debug, Copy, Clone)]
pub struct DefaultInit;
