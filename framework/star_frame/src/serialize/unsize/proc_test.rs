use crate::prelude::*;
use crate::serialize::unsize::test::CombinedTest;

#[unsized_type]
pub struct SizedAndUnsized {
    pub sized1: bool,
    pub sized2: PackedValue<u16>,
    pub sized3: u8,
    #[unsized_start]
    pub list1: List<u8>,
    pub list2: List<bool>,
    pub other: CombinedTest,
}

#[unsized_type]
pub struct OnlyUnsized {
    #[unsized_start]
    pub list1: List<u8>,
    pub list2: List<bool>,
    pub other: CombinedTest,
    pub thing1: List<PackedValue<u16>>,
}
