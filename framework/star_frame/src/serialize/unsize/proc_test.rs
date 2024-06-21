use crate::prelude::*;
use crate::serialize::unsize::test::{CombinedTest, TestStruct};
use star_frame_proc::unsized_type;

#[unsized_type]
pub struct SomeUnsizedType {
    pub sized1: bool,
    pub sized2: PackedValue<u16>,
    pub sized3: u8,
    #[unsized_start]
    pub list1: List<u8>,
    pub list2: List<TestStruct>,
    pub other: CombinedTest,
}
