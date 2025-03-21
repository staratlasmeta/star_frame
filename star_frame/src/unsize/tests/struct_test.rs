use crate::prelude::*;
use crate::unsize::test_helpers::TestByteSet;
// use crate::unsize::tests::struct_test::many_unsized::{
//     ManyUnsized, ManyUnsizedExclusiveExt, ManyUnsizedOwned,
// };
use crate::unsize::tests::struct_test::many_unsized::{
    ManyUnsized, ManyUnsizedExclusiveExt, ManyUnsizedOwned,
};
use pretty_assertions::assert_eq;
use star_frame_proc::{derivative, unsized_impl};

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct UnsizedTest {
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>, u8>,
    pub unsized3: UnsizedTest3,
    pub unsized2: List<PackedValue<u16>, u8>,
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
pub struct UnsizedTest3 {
    #[unsized_start]
    pub unsized3: List<PackedValue<u16>, u8>,
}

#[unsized_impl]
impl UnsizedTest3 {
    #[exclusive]
    fn foo<'child>(
        &'child mut self,
    ) -> ExclusiveWrapperT<'child, 'ptr, 'top, 'info, List<PackedValue<u16>, u8>, O, A> {
        self.unsized3()
    }
}

#[test]
fn test_unsized_simple() -> Result<()> {
    TestByteSet::<UnsizedTest>::new(DefaultInit)?;
    let r = TestByteSet::<UnsizedTest>::new(UnsizedTestInit {
        unsized1: [100, 101, 102].map(Into::into),
        unsized2: [200, 201, 202].map(Into::into),
        unsized3: UnsizedTest3Init {
            unsized3: [150, 151, 152].map(Into::into),
        },
    })?;
    let mut data_mut = r.data_mut()?;

    let mut banana = data_mut.exclusive();
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

    drop(data_mut);

    let expected = UnsizedTestOwned {
        unsized1: [100, 101, 102, 103].map(Into::into).to_vec(),
        unsized2: [200, 201, 202, 203].map(Into::into).to_vec(),
        unsized3: UnsizedTest3Owned {
            unsized3: [150, 151, 152, 153].map(Into::into).to_vec(),
        },
    };
    let owned = UnsizedTest::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(owned, expected);
    Ok(())
}

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

#[test]
fn test_single_unsized() -> Result<()> {
    TestByteSet::<SingleUnsized>::new(DefaultInit)?;
    let r = TestByteSet::<SingleUnsized>::new(SingleUnsizedInit { unsized1: [1, 2] })?;
    r.data_mut()?.exclusive().unsized1().push(3)?;
    r.data_mut()?
        .exclusive()
        .unsized1()
        .insert_all(1, [10, 9, 8])?;
    let expected = vec![1, 10, 9, 8, 2, 3];
    assert_eq!(r.data_ref()?.unsized1.as_slice(), expected.as_slice());
    let owned = SingleUnsized::owned_from_ref(*r.data_ref()?)?;
    assert_eq!(owned, SingleUnsizedOwned { unsized1: expected });
    Ok(())
}

mod many_unsized {
    use super::*;
    #[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
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
    TestByteSet::<many_unsized::ManyUnsized>::new(DefaultInit)?;
    let r = TestByteSet::<many_unsized::ManyUnsized>::new(many_unsized::ManyUnsizedInit {
        sized: many_unsized::ManyUnsizedSized {
            sized1: 1,
            sized2: 2,
        },
        unsized1: [1.into()],
        unsized2: SingleUnsizedInit { unsized1: [2] },
        unsized3: 3,
        unsized4: [TestStruct { val1: 4, val2: 5 }],
        unsized5: [TestStruct { val1: 6, val2: 7 }],
    })?;
    r.data_mut()?.exclusive().foo()?;
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

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
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
    >::new(WithSizedAndUnsizedGenericsInit {
        sized: WithSizedAndUnsizedGenericsSized {
            sized1: TestStruct { val1: 1, val2: 2 },
            sized2: 3.into(),
            __generics: Default::default(),
        },
        unsized1: PackedValueChecked(3u16),
        unsized2: [5.into()],
    })?;
    r.data_mut()?.exclusive().thingy()?;
    let owned = WithSizedAndUnsizedGenerics::owned_from_ref(*r.data_ref()?)?;
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
