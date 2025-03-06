use crate::prelude::*;
use crate::unsize::test_helpers::TestByteSet;
use pretty_assertions::assert_eq;
use star_frame_proc::derivative;

#[derive(Debug, Copy, Clone, Pod, Zeroable, Align1, PartialEq, Eq, TypeToIdl)]
#[repr(C, packed)]
pub struct TestStruct {
    pub val1: u32,
    pub val2: u64,
}

#[unsized_type(
    owned_attributes = [
        derive(PartialEq, Eq, Clone)
    ]
)]
pub struct SingleUnsized {
    #[unsized_start]
    pub unsized1: List<u8>,
}
//
// #[unsized_impl]
// impl SingleUnsized {
//     fn foo(&mut self) -> Result<u16> {
//         let list = &mut self.unsized1()?;
//         list.push(2u16.into())?;
//         Ok(10)
//     }
// }

#[test]
fn test_single_unsized() -> Result<()> {
    TestByteSet::<SingleUnsized>::new(DefaultInit)?;
    let r = &mut TestByteSet::<SingleUnsized>::new(SingleUnsizedInit { unsized1: [1, 2] })?;
    r.data_mut()?.unsized1_exclusive().push(3)?;
    r.data_mut()?
        .unsized1_exclusive()
        .insert_all(1, [10, 9, 8])?;
    let owned = SingleUnsized::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(
        owned,
        SingleUnsizedOwned {
            unsized1: vec![1, 10, 9, 8, 2, 3]
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
pub struct ManyUnsized {
    pub sized1: u8,
    pub sized2: u8,
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
    unsized2: SingleUnsized,
    pub unsized3: u8,
    pub unsized4: List<TestStruct>,
    unsized5: List<TestStruct>,
}
//
// #[unsized_impl]
// impl ManyUnsized {
//     fn foo(&mut self) -> Result<u16> {
//         let list = &mut self.unsized1()?;
//         list.push(2u16.into())?;
//         Ok(10)
//     }
//     pub fn bar(&self) -> Result<()> {
//         Ok(())
//     }
// }
//
// #[unsized_impl(tag = "1")]
// impl ManyUnsized {
//     fn foo1(&mut self) -> Result<u16> {
//         let list = &mut self.unsized1()?;
//         list.push(2u16.into())?;
//         Ok(10)
//     }
//     pub fn bar1(&self) -> Result<()> {
//         Ok(())
//     }
// }

#[test]
fn test_many_unsized() -> Result<()> {
    TestByteSet::<ManyUnsized>::new(DefaultInit)?;
    let r = TestByteSet::<ManyUnsized>::new(ManyUnsizedInit {
        sized: ManyUnsizedSized {
            sized1: 1,
            sized2: 2,
        },
        unsized1: [1.into()],
        unsized2: SingleUnsizedInit { unsized1: [2] },
        unsized3: 3,
        unsized4: [TestStruct { val1: 4, val2: 5 }],
        unsized5: [TestStruct { val1: 6, val2: 7 }],
    })?;

    let expected = ManyUnsizedOwned {
        sized1: 1,
        sized2: 2,
        unsized1: vec![1.into()],
        unsized2: SingleUnsizedOwned { unsized1: vec![2] },
        unsized3: 3,
        unsized4: vec![TestStruct { val1: 4, val2: 5 }],
        unsized5: vec![TestStruct { val1: 6, val2: 7 }],
    };
    let owned = ManyUnsized::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(owned, expected);
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
pub struct SingleUnsizedWithSized {
    pub sized1: bool,
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
}

#[test]
fn test_single_unsized_with_sized() -> Result<()> {
    TestByteSet::<SingleUnsizedWithSized>::new(DefaultInit)?;
    let r = TestByteSet::<SingleUnsizedWithSized>::new(SingleUnsizedWithSizedInit {
        sized: DefaultInit,
        unsized1: [1.into()],
    })?;
    let owned = SingleUnsizedWithSized::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(
        owned,
        SingleUnsizedWithSizedOwned {
            sized1: false,
            unsized1: vec![1.into()],
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
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

#[test]
fn test_sized_and_unsized() -> Result<()> {
    TestByteSet::<SizedAndUnsized>::new(DefaultInit)?;
    let r = TestByteSet::<SizedAndUnsized>::new(SizedAndUnsizedInit {
        sized: SizedAndUnsizedSized {
            sized1: true,
            sized2: 1.into(),
            sized3: 2,
            sized4: [3; 10],
        },
        unsized1: [4.into()],
        unsized2: [TestStruct { val1: 5, val2: 6 }],
        unsized3: 7,
    })?;
    let owned = SizedAndUnsized::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(
        owned,
        SizedAndUnsizedOwned {
            sized1: true,
            sized2: 1.into(),
            sized3: 2,
            sized4: [3; 10],
            unsized1: vec![4.into()],
            unsized2: vec![TestStruct { val1: 5, val2: 6 }],
            unsized3: 7,
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl, skip_phantom_generics)]
pub struct WithSizedGenerics<A: UnsizedGenerics, B>
where
    B: UnsizedGenerics,
{
    pub sized1: A,
    pub sized2: B,
    pub sized3: u8,
    #[unsized_start]
    pub unsized1: List<TestStruct>,
}

#[test]
fn test_with_sized_generics() -> Result<()> {
    TestByteSet::<WithSizedGenerics<TestStruct, bool>>::new(DefaultInit)?;
    let r = TestByteSet::<WithSizedGenerics<TestStruct, bool>>::new(WithSizedGenericsInit {
        sized: WithSizedGenericsSized {
            sized1: TestStruct { val1: 1, val2: 2 },
            sized2: true,
            sized3: 4,
        },
        unsized1: [TestStruct { val1: 5, val2: 6 }],
    })?;
    let owned = WithSizedGenerics::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(
        owned,
        WithSizedGenericsOwned {
            sized1: TestStruct { val1: 1, val2: 2 },
            sized2: true,
            sized3: 4,
            unsized1: vec![TestStruct { val1: 5, val2: 6 }],
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct WithUnsizedGenerics<A: UnsizedGenerics, B>
where
    B: UnsizedGenerics,
{
    pub sized1: u8,
    #[unsized_start]
    pub unsized1: List<A>,
    pub unsized2: List<B>,
}

#[test]
fn test_with_unsized_generics() -> Result<()> {
    TestByteSet::<WithUnsizedGenerics<PackedValueChecked<u16>, TestStruct>>::new(DefaultInit)?;
    let r = TestByteSet::<WithUnsizedGenerics<PackedValueChecked<u16>, TestStruct>>::new(
        WithUnsizedGenericsInit {
            sized: WithUnsizedGenericsSized {
                sized1: 1,
                __generics: Default::default(),
            },
            unsized1: [PackedValueChecked(u16::MAX)],
            unsized2: [TestStruct { val1: 3, val2: 4 }],
        },
    )?;
    let owned = WithUnsizedGenerics::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(
        owned,
        WithUnsizedGenericsOwned {
            sized1: 1,
            unsized1: vec![PackedValueChecked(u16::MAX)],
            unsized2: vec![TestStruct { val1: 3, val2: 4 }]
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct WithOnlyUnsizedGenerics<A: UnsizedGenerics, B: UnsizedType>
where
    B::Owned: PartialEq + Eq + Clone,
{
    #[unsized_start]
    pub unsized1: List<A>,
    pub unsized2: B,
}

#[test]
fn test_with_only_unsized_generics() -> Result<()> {
    TestByteSet::<WithOnlyUnsizedGenerics<TestStruct, PackedValueChecked<u16>>>::new(DefaultInit)?;
    let r = TestByteSet::<WithOnlyUnsizedGenerics<TestStruct, PackedValueChecked<u16>>>::new(
        WithOnlyUnsizedGenericsInit {
            unsized1: [TestStruct { val1: 4, val2: 5 }],
            unsized2: PackedValueChecked(10),
        },
    )?;
    let owned = WithOnlyUnsizedGenerics::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(
        owned,
        WithOnlyUnsizedGenericsOwned {
            unsized1: vec![TestStruct { val1: 4, val2: 5 }],
            unsized2: PackedValueChecked(10),
        }
    );
    Ok(())
}

// #[unsized_type(owned_attributes = [star_frame_proc::derivative(PartialEq, Eq, Clone)], skip_idl)]
// pub struct WithSizedAndUnsizedGenerics<A: UnsizedGenerics, B, C>
// where
//     B: UnsizedType<Owned: Clone + PartialEq + Eq> + ?Sized,
//     C: CheckedBitPattern + Align1 + NoUninit + Zeroable,
// {
//     pub sized1: A,
//     pub sized2: C,
//     #[unsized_start]
//     pub unsized1: B,
//     pub unsized2: List<C>,
// }
//
// #[unsized_impl]
// impl<A: UnsizedGenerics, B, C> WithSizedAndUnsizedGenerics<A, B, C>
// where
//     B: UnsizedGenerics,
//     C: CheckedBitPattern + Align1 + NoUninit + Zeroable,
// {
//     fn thingy(&mut self) -> Result<()> {
//         let item_to_push = self.sized1;
//         self.unsized1()?.push(item_to_push)?;
//         Ok(())
//     }
// }
//
// #[test]
// fn test_with_sized_and_unsized_generics() -> Result<()> {
//     TestByteSet::<WithSizedAndUnsizedGenerics<TestStruct, List<u8>, PackedValueChecked<u32>>>::new(
//         DefaultInit,
//     )?;
//     let mut r = TestByteSet::<
//         WithSizedAndUnsizedGenerics<TestStruct, PackedValueChecked<u16>, PackedValueChecked<u32>>,
//     >::new(WithSizedAndUnsizedGenericsInit {
//         sized: WithSizedAndUnsizedGenericsSized {
//             sized1: TestStruct { val1: 1, val2: 2 },
//             sized2: PackedValueChecked(3u16),
//             __generics: Default::default(),
//         },
//         unsized1: [1, 2, 3],
//         unsized2: [TestStruct { val1: 6, val2: 7 }],
//     })?;
//     // r.mutable()?.thingy()?;
//     let owned = WithSizedAndUnsizedGenerics::owned_from_ref(*r.data_ref()?)?;
//     assert_eq!(
//         owned,
//         WithSizedAndUnsizedGenericsOwned {
//             sized1: TestStruct { val1: 1, val2: 2 },
//             sized2: PackedValueChecked(3u16),
//             unsized1: vec![
//                 TestStruct { val1: 4, val2: 5 },
//                 TestStruct { val1: 1, val2: 2 }
//             ],
//             unsized2: (
//                 TestStruct { val1: 6, val2: 7 },
//                 PackedValueChecked(u32::MAX / 4),
//             ),
//             __generics: Default::default(),
//         }
//     );
//     Ok(())
// }

// #[unsized_type(skip_idl)]
// pub struct MapTest {
//     #[unsized_start]
//     pub map: Map<u8, TestStruct>,
//     pub map2: Map<u8, TestStruct>,
// }
//
// use crate::unsize::test_helpers::TestByteSet;
// use crate::unsize::CombinedExt;
//
// #[unsized_impl]
// impl MapTest {
//     fn get_first(&self) -> Result<usize> {
//         let size = self.map()?.capacity();
//         Ok(size)
//     }
// }
//
// #[test]
// fn test_map() -> Result<()> {
//     let mut r = TestByteSet::<MapTest>::new_default()?;
//     let mutable = r.mutable()?;
//     let map = &mut mutable.map()?;
//     map.insert(1, TestStruct { val1: 1, val2: 10 })?;
//     map.insert(3, TestStruct { val1: 3, val2: 30 })?;
//     map.insert(2, TestStruct { val1: 2, val2: 20 })?;
//     let first = map.get(&1);
//     println!("{first:?}");
//     Ok(())
// }

//todo: make a single very complex struct and test it with a watcher on owned like list
