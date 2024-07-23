use crate::prelude::UnsizedType;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapperTypes};
use bytemuck::{CheckedBitPattern, NoUninit};

pub trait UnsizedEnum: UnsizedType {
    type Discriminant: CheckedBitPattern + NoUninit;

    fn discriminant<S: AsBytes>(
        r: &impl RefWrapperTypes<Super = S, Ref = Self::RefData>,
    ) -> Self::Discriminant;
}
