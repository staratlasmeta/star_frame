use crate::prelude::*;
use crate::serialize::ref_wrapper::{AsMutBytes, RefWrapper};

// TODO: Remove this
#[allow(clippy::type_complexity)]
pub trait UnsizedInit<InitArg>: UnsizedType {
    const INIT_BYTES: usize;

    /// # Safety
    /// `super_ref` must have [`UnsizedInit::INIT_BYTES`] zeroed bytes.
    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: InitArg,
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)>;
}
