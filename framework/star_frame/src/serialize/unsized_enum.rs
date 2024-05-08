use crate::prelude::*;
use crate::serialize::ref_wrapper::{AsBytes, RefWrapper};
use bytemuck::CheckedBitPattern;
use std::fmt::Debug;

pub trait UnsizedEnum: UnsizedType {
    type Discriminant: 'static + Copy + Debug + CheckedBitPattern;

    fn discriminant<S: AsBytes>(r: &RefWrapper<S, Self::RefData>) -> Self::Discriminant;
}

// ---------------------------- Test Stuff ----------------------------

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use bytemuck::{Pod, Zeroable};
    use star_frame_proc::Align1;

    #[derive(Pod, Zeroable, Copy, Clone, Align1, Debug, PartialEq, Eq)]
    #[repr(C, packed)]
    pub struct TestStruct {
        val1: u32,
        val2: u64,
    }

    // enum TestEnum<T>
    // where
    //     T: UnsizedType,
    // {
    //     // A,
    //     A,
    //     // B(CombinedUnsized<TestStruct, List<u8, u8>>) = 4,
    //     B = 4,
    //     // C {
    //     //     list: List<PackedValue<u32>>,
    //     //     other: T,
    //     // },
    //     C,
    // }
}
