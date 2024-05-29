use crate::prelude::UnsizedType;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use bytemuck::{CheckedBitPattern, NoUninit};

pub trait UnsizedEnum: UnsizedType {
    type Discriminant: CheckedBitPattern + NoUninit;

    fn discriminant<S: AsBytes>(r: &RefWrapper<S, Self::RefData>) -> Self::Discriminant;
}
