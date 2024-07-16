use crate::prelude::*;

#[derive(Debug, Copy, Clone, Pod, Zeroable, Align1, PartialEq, Eq)]
#[repr(C, packed)]
pub struct TestStruct {
    pub val1: u32,
    pub val2: u64,
}

#[unsized_type]
pub struct CombinedTest {
    #[unsized_start]
    pub list1: List<u8>,
    pub list2: List<TestStruct>,
}

// TODO: More fields, generics, enums, tuple structs, unit structs, macro it all
#[unsized_type]
pub struct SingleUnsized {
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
}

#[unsized_type]
pub struct ManyUnsized {
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
    pub unsized2: SingleUnsized,
    pub unsized3: u8,
    pub unsized4: List<TestStruct>,
    pub unsized5: List<TestStruct>,
}

#[unsized_type]
pub struct SingleUnsizedWithSized {
    pub sized1: bool,
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
}

#[unsized_type]
pub struct SizedAndUnsized {
    pub sized1: bool,
    pub sized2: PackedValue<u16>,
    pub sized3: u8,
    pub sized4: [u8; 10],
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
    pub unsized2: List<TestStruct>,
    pub unsized3: u8,
}

#[unsized_type]
pub struct WithSizedGenerics<A, B, C>
where
    A: UnsizedGenerics,
    B: UnsizedGenerics,
    C: UnsizedGenerics,
{
    pub sized1: A,
    pub sized2: B,
    // pub sized3: C,
    pub sized4: u8,
    #[unsized_start]
    pub unsized1: C,
    pub unsized2: List<TestStruct>,
}

//
// #[unsized_type]
// pub struct SizedAndUnsized {
//     pub sized1: bool,
//     pub sized2: PackedValue<u16>,
//     pub sized3: u8,
//     #[unsized_start]
//     pub list1: List<u8>,
//     pub list2: List<bool>,
//     pub other: CombinedTest,
// }
//
// #[unsized_type]
// pub struct OnlyUnsized {
//     #[unsized_start]
//     pub list1: List<u8>,
//     pub list2: List<bool>,
//     pub other: CombinedTest,
//     pub thing1: List<PackedValue<u16>>,
// }
//
// #[unsized_type]
// pub struct BigBoi
// // pub struct BigBoi<T0, T1, T2>
// // where
// //     T0: CheckedBitPattern + Zeroable + Align1 + Copy,
// //     T1: CheckedBitPattern + Zeroable + Align1 + Copy,
// //     T2: CheckedBitPattern + Zeroable + Align1 + Copy,
// {
//     pub sized00: u8,
//     // pub sized00: T0,
//     pub sized01: u8,
//     // pub sized01: T1,
//     pub sized02: u8,
//     // pub sized02: T2,
//     #[unsized_start]
//     pub unsized00: List<u8>,
//     // pub unsized01: List<u8>,
//     // pub unsized02: List<u8>,
//     // pub unsized03: List<u8>,
//     // pub unsized04: List<u8>,
//     // pub unsized05: List<u8>,
//     // pub unsized06: List<u8>,
//     // pub unsized07: List<u8>,
//     // pub unsized08: List<u8>,
//     // pub unsized09: List<u8>,
//     // pub unsized10: List<u8>,
//     // pub unsized11: List<u8>,
//     // pub unsized12: List<u8>,
//     // pub unsized13: List<u8>,
//     // pub unsized14: List<u8>,
//     // pub unsized15: List<u8>,
//     // pub unsized16: List<u8>,
//     // pub unsized17: List<u8>,
//     // pub unsized18: List<u8>,
//     // pub unsized19: List<u8>,
//     // pub unsized20: List<u8>,
// }
//
// #[unsized_type]
// pub struct BigBoi<T0, T1, T2>
// where
//     T0: CheckedBitPattern + Zeroable + Align1,
//     T1: CheckedBitPattern + Zeroable + Align1,
//     T2: CheckedBitPattern + Zeroable + Align1,
// {
//     pub sized00: T0,
//     pub sized01: T1,
//     pub sized02: T2,
//     #[unsized_start]
//     pub unsized00: List<u8>,
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::Zeroed;
    use crate::serialize::test_helpers::TestByteSet;
    use advance::Length;
    //
    // #[test]
    // fn test_minimum() -> anyhow::Result<()> {
    //     type Thingy = CombinedUnsized<CombinedUnsized<u8, u8>, u8>;
    //     let mut bytes = TestByteSet::<BigBoi>::new(Zeroed)?;
    //     let r = bytes.immut()?;
    //     assert_eq!(r.field10()?.len(), 0);
    //     assert_eq!(r.field20()?.len(), 0);
    //
    //     let owned = BigBoi::owned(r)?;
    //     println!("Owned: {:#?}", owned);
    //
    //     Ok(())
    // }
    //
    // fn cool(
    //     r: &mut RefWrapper<impl Resize<CombinedTestMeta>, CombinedTestRef>,
    //     val: u32,
    // ) -> anyhow::Result<()> {
    //     r.list1()?.push(0)?;
    //     r.list2()?.insert(0, TestStruct { val1: val, val2: 0 })?;
    //     Ok(())
    // }
    //
    // #[test]
    // fn test() -> anyhow::Result<()> {
    //     let mut bytes = TestByteSet::<CombinedTest>::new(Zeroed)?;
    //     let mut r = bytes.mutable()?;
    //     assert_eq!(&**(&r).list1()?, &[] as &[u8]);
    //     assert_eq!(&**(&r).list2()?, &[]);
    //     cool(&mut r, 1)?;
    //     assert_eq!(&**(&r).list1()?, &[0]);
    //     assert_eq!(&**(&r).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
    //     cool(&mut r, 2)?;
    //     let r = bytes.immut()?;
    //     assert_eq!(&**r.list1()?, &[0, 0]);
    //     assert_eq!(
    //         &**r.list2()?,
    //         &[
    //             TestStruct { val1: 2, val2: 0 },
    //             TestStruct { val1: 1, val2: 0 }
    //         ]
    //     );
    //     Ok(())
    // }
    //
    // type CombinedTestRefWrapper<S> = RefWrapper<S, CombinedTestRef>;
    // #[test]
    // fn test_stuff() -> anyhow::Result<()> {
    //     let bytes = vec![0u8; 100];
    //     let combined: CombinedTestRefWrapper<_> =
    //         unsafe { CombinedTest::from_bytes(bytes).unwrap() }.ref_wrapper;
    //     println!("{combined:?}");
    //     let mut list = combined.list1().unwrap();
    //     list.push(1)?;
    //     list.insert(0, 2)?;
    //     println!("{:?}", list.len());
    //     println!("{:?}", list.as_slice());
    //     Ok(())
    // }
}
