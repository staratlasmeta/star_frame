use crate::prelude::*;
use crate::unsize::test_helpers::TestByteSet;
use crate::unsize::tests::struct_test::many_unsized::{ManyUnsizedExclusiveExt, ManyUnsizedOwned};
use crate::unsize::ModifyOwned;
use pretty_assertions::assert_eq;
use star_frame_proc::unsized_impl;

#[unsized_type(skip_idl)]
pub struct UnsizedTest {
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>, u8>,
    pub unsized3: UnsizedTest3,
    pub unsized2: List<PackedValue<u16>, u8>,
    pub map: Map<u8, PackedValue<u16>>,
    pub map2: UnsizedMap<u8, UnsizedTest3>,
}

#[unsized_type]
pub struct UnsizedTest3 {
    #[unsized_start]
    pub unsized3: List<PackedValue<u16>, u8>,
    pub unsized_map: UnsizedMap<u8, List<u8>>,
}

#[unsized_impl]
impl UnsizedTest3 {
    #[exclusive]
    fn foo<'child>(
        &'child mut self,
    ) -> ExclusiveWrapper<'child, 'top, ListMut<'top, PackedValue<u16>, u8>, Self> {
        self.unsized3()
    }
}

#[test]
fn test_unsized_test() -> Result<()> {
    TestByteSet::<UnsizedTest>::new_default()?;
    let r = TestByteSet::<UnsizedTest>::new(UnsizedTestOwned {
        unsized1: [100, 101, 102].map(Into::into).to_vec(),
        unsized2: [200, 201, 202].map(Into::into).to_vec(),
        unsized3: UnsizedTest3Owned {
            unsized3: [150, 151, 152].map(Into::into).to_vec(),
            unsized_map: Default::default(),
        },
        map: Default::default(),
        map2: Default::default(),
    })?;

    let mut banana = r.data_mut()?;
    banana.unsized1().push(103.into())?;
    assert_eq!(&**banana.unsized1, [100, 101, 102, 103]);
    assert_eq!(&**banana.unsized2, [200, 201, 202]);
    assert_eq!(&**banana.unsized3.unsized3, [150, 151, 152]);
    banana.unsized2().push(203.into())?;
    assert_eq!(&**banana.unsized1, [100, 101, 102, 103]);
    assert_eq!(&**banana.unsized2, [200, 201, 202, 203]);
    assert_eq!(&**banana.unsized3.unsized3, [150, 151, 152]);

    banana.unsized3().unsized3().push(153.into())?;
    assert_eq!(&**banana.unsized1, [100, 101, 102, 103]);
    assert_eq!(&**banana.unsized2, [200, 201, 202, 203]);
    assert_eq!(&**banana.unsized3.unsized3, [150, 151, 152, 153]);
    for (key, value) in &mut banana.map {
        println!("{key:?}: {value:?}");
    }

    let mut map2 = banana.map2();
    let unsized3_arr = [1, 2, 3, 4, 5];
    map2.insert(
        1,
        UnsizedTest3Init {
            unsized3: unsized3_arr.map(Into::into),
            unsized_map: DefaultInit,
        },
    )?;
    let mut item = map2.get_exclusive(&1)?.expect("Item exsts");
    assert_eq!(&**item.unsized3, unsized3_arr);
    item.unsized3().push(6.into())?;
    item.unsized_map().insert(1, [1, 2, 3])?;
    drop(item); // ensure drop works properly still
    let mut some_item = banana.unsized2();
    some_item.push(204.into())?;
    drop(banana);

    let expected = UnsizedTestOwned {
        unsized1: [100, 101, 102, 103].map(Into::into).to_vec(),
        unsized2: [200, 201, 202, 203, 204].map(Into::into).to_vec(),
        unsized3: UnsizedTest3Owned {
            unsized3: [150, 151, 152, 153].map(Into::into).to_vec(),
            unsized_map: Default::default(),
        },
        map: Default::default(),
        map2: std::iter::once((
            1,
            UnsizedTest3Owned {
                unsized3: [1, 2, 3, 4, 5, 6].map(Into::into).to_vec(),
                unsized_map: [(1u8, vec![1, 2, 3])].into_iter().collect(),
            },
        ))
        .collect(),
    };
    let owned = r.owned()?;
    assert_eq!(owned, expected);
    Ok(())
}

#[test]
fn test_modify_owned() -> Result<()> {
    let mut my_vec = vec![1u8, 2, 3];
    my_vec.modify_owned::<List<u8>>(|a| {
        a.push(4)?;
        Ok(())
    })?;
    assert_eq!(my_vec, vec![1, 2, 3, 4]);
    Ok(())
}

#[derive(Debug, Copy, Clone, Pod, Zeroable, Align1, PartialEq, Eq, TypeToIdl)]
#[repr(C, packed)]
pub struct TestStruct {
    pub val1: u32,
    pub val2: u64,
}

#[unsized_type]
pub struct SingleUnsized {
    #[unsized_start]
    pub unsized1: List<u8>,
}

#[test]
fn test_single_unsized() -> Result<()> {
    TestByteSet::<SingleUnsized>::new_default()?;
    let r = TestByteSet::<SingleUnsized>::new(SingleUnsizedOwned {
        unsized1: vec![1, 2],
    })?;
    r.data_mut()?.unsized1().push(3)?;
    r.data_mut()?.unsized1().insert_all(1, [10, 9, 8])?;
    let expected = vec![1, 10, 9, 8, 2, 3];
    assert_eq!(r.data()?.unsized1.as_slice(), expected.as_slice());
    let owned = r.owned()?;
    assert_eq!(owned, SingleUnsizedOwned { unsized1: expected });
    Ok(())
}

mod many_unsized {
    use super::*;
    #[unsized_type()]
    pub struct ManyUnsized {
        pub sized1: u8,
        pub sized2: u8,
        #[unsized_start]
        pub unsized1: List<PackedValue<u16>>,
        pub unsized3: u8,
        pub unsized2: SingleUnsized,
        pub unsized4: List<TestStruct>,
        pub unsized5: List<TestStruct>,
    }
}

#[unsized_impl]
impl many_unsized::ManyUnsized {
    #[exclusive]
    fn foo(&mut self) -> Result<u16> {
        let list = &mut self.unsized1();
        list.push(2u16.into())?;
        self.unsized5().push(TestStruct { val1: 8, val2: 9 })?;
        Ok(10)
    }
}

#[unsized_impl(tag = "1")]
impl many_unsized::ManyUnsized {
    #[exclusive]
    fn bar(&mut self) -> Result<u16> {
        let mut list = self.unsized1();
        list.push(426u16.into())?;
        Ok(10)
    }
}

#[test]
fn test_many_unsized() -> Result<()> {
    TestByteSet::<many_unsized::ManyUnsized>::new_default()?;
    let r = TestByteSet::<many_unsized::ManyUnsized>::new(many_unsized::ManyUnsizedOwned {
        sized1: 1,
        sized2: 2,
        unsized1: vec![1.into()],
        unsized2: SingleUnsizedOwned { unsized1: vec![2] },
        unsized3: 3,
        unsized4: vec![TestStruct { val1: 4, val2: 5 }],
        unsized5: vec![TestStruct { val1: 6, val2: 7 }],
    })?;
    r.data_mut()?.foo()?;
    let expected = ManyUnsizedOwned {
        sized1: 1,
        sized2: 2,
        unsized1: vec![1.into(), 2.into()],
        unsized2: SingleUnsizedOwned { unsized1: vec![2] },
        unsized3: 3,
        unsized4: vec![TestStruct { val1: 4, val2: 5 }],
        unsized5: vec![
            TestStruct { val1: 6, val2: 7 },
            TestStruct { val1: 8, val2: 9 },
        ],
    };
    let owned = r.owned()?;
    assert_eq!(owned, expected);
    Ok(())
}

#[unsized_type()]
pub struct SingleUnsizedWithSized {
    pub sized1: bool,
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
}

#[test]
fn test_single_unsized_with_sized() -> Result<()> {
    TestByteSet::<SingleUnsizedWithSized>::new_default()?;
    let r = TestByteSet::<SingleUnsizedWithSized>::new(SingleUnsizedWithSizedOwned {
        sized1: false,
        unsized1: vec![1.into()],
    })?;
    let owned = r.owned()?;
    assert_eq!(
        owned,
        SingleUnsizedWithSizedOwned {
            sized1: false,
            unsized1: vec![1.into()],
        }
    );
    Ok(())
}

#[unsized_type()]
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
    TestByteSet::<SizedAndUnsized>::new_default()?;
    let r = TestByteSet::<SizedAndUnsized>::new(SizedAndUnsizedOwned {
        sized1: true,
        sized2: 1.into(),
        sized3: 2,
        sized4: [3; 10],
        unsized1: vec![4.into()],
        unsized2: vec![TestStruct { val1: 5, val2: 6 }],
        unsized3: 7,
    })?;
    let owned = r.owned()?;
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

#[unsized_type(skip_idl, skip_phantom_generics)]
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
    TestByteSet::<WithSizedGenerics<TestStruct, bool>>::new_default()?;
    let r = TestByteSet::<WithSizedGenerics<TestStruct, bool>>::new(WithSizedGenericsOwned {
        sized1: TestStruct { val1: 1, val2: 2 },
        sized2: true,
        sized3: 4,
        unsized1: vec![TestStruct { val1: 5, val2: 6 }],
    })?;
    let owned = r.owned()?;
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

#[unsized_type(skip_idl)]
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
    TestByteSet::<WithUnsizedGenerics<PackedValueChecked<u16>, TestStruct>>::new_default()?;
    let r = TestByteSet::<WithUnsizedGenerics<PackedValueChecked<u16>, TestStruct>>::new(
        WithUnsizedGenericsOwned {
            sized1: 1,
            unsized1: vec![PackedValueChecked(u16::MAX)],
            unsized2: vec![TestStruct { val1: 3, val2: 4 }],
        },
    )?;
    let owned = r.owned()?;
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

#[unsized_type(skip_idl)]
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
    TestByteSet::<WithOnlyUnsizedGenerics<TestStruct, PackedValueChecked<u16>>>::new_default()?;
    let r = TestByteSet::<WithOnlyUnsizedGenerics<TestStruct, PackedValueChecked<u16>>>::new(
        WithOnlyUnsizedGenericsOwned {
            unsized1: vec![TestStruct { val1: 4, val2: 5 }],
            unsized2: PackedValueChecked(10),
        },
    )?;
    let owned = r.owned()?;
    assert_eq!(
        owned,
        WithOnlyUnsizedGenericsOwned {
            unsized1: vec![TestStruct { val1: 4, val2: 5 }],
            unsized2: PackedValueChecked(10),
        }
    );
    Ok(())
}

#[unsized_type(skip_idl)]
pub struct WithSizedAndUnsizedGenerics<A: UnsizedGenerics, B, C>
where
    B: UnsizedType<Owned: Clone + PartialEq + Eq> + ?Sized,
    C: CheckedBitPattern + Align1 + NoUninit + Zeroable,
{
    pub sized1: A,
    pub sized2: C,
    #[unsized_start]
    pub unsized1: B,
    pub unsized2: List<C>,
}

#[unsized_impl]
impl<A: UnsizedGenerics, B, C> WithSizedAndUnsizedGenerics<A, B, C>
where
    B: UnsizedType<Owned: Clone + PartialEq + Eq> + ?Sized,
    C: CheckedBitPattern + Align1 + NoUninit + Zeroable,
{
    #[exclusive]
    fn thingy(&mut self) -> Result<()> {
        let item_to_push = self.sized2;
        self.unsized2().push(item_to_push)?;
        Ok(())
    }
}

#[test]
fn test_with_sized_and_unsized_generics() -> Result<()> {
    let r = TestByteSet::<
        WithSizedAndUnsizedGenerics<TestStruct, PackedValueChecked<u16>, PackedValueChecked<u32>>,
    >::new(WithSizedAndUnsizedGenericsOwned {
        sized1: TestStruct { val1: 1, val2: 2 },
        sized2: 3.into(),
        unsized1: PackedValueChecked(3u16),
        unsized2: vec![5.into()],
    })?;
    r.data_mut()?.thingy()?;
    let owned = r.owned()?;
    assert_eq!(
        owned,
        WithSizedAndUnsizedGenericsOwned {
            sized1: TestStruct { val1: 1, val2: 2 },
            sized2: 3.into(),
            unsized1: 3.into(),
            unsized2: vec![5.into(), 3.into()],
        }
    );
    Ok(())
}

//todo: make a single very complex struct and test it with a watcher on owned like list
