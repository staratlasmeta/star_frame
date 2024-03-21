use crate::prelude::UnsizedType;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use crate::Result;

/// TODO: Implement this with macro, maybe move into [`UnsizedType`]
pub trait UnsizedTypeToOwned: UnsizedType {
    type Owned;

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned>;
}
