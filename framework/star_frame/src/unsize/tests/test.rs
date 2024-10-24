use crate::prelude::*;
use pretty_assertions::assert_eq;

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
    pub unsized1: List<PackedValue<u16>>,
}

#[test]
fn test_single_unsized() -> Result<()> {
    TestByteSet::<SingleUnsized>::new(Zeroed)?;
    let r = &mut TestByteSet::<SingleUnsized>::new(SingleUnsizedInit {
        unsized1: [PackedValue(1)],
    })?;
    let owned = SingleUnsized::owned(r.immut()?)?;
    assert_eq!(
        owned,
        SingleUnsizedOwned {
            unsized1: vec![1.into()]
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
pub struct ManyUnsized {
    #[unsized_start]
    pub unsized1: List<PackedValue<u16>>,
    pub unsized2: SingleUnsized,
    pub unsized3: u8,
    pub unsized4: List<TestStruct>,
    pub unsized5: List<TestStruct>,
}

#[test]
fn test_many_unsized() -> Result<()> {
    TestByteSet::<ManyUnsized>::new(Zeroed)?;
    let r = TestByteSet::<ManyUnsized>::new(ManyUnsizedInit {
        unsized1: [PackedValue(1)],
        unsized2: SingleUnsizedInit {
            unsized1: [PackedValue(2)],
        },
        unsized3: 3,
        unsized4: [TestStruct { val1: 4, val2: 5 }],
        unsized5: [TestStruct { val1: 6, val2: 7 }],
    })?;

    let expected = ManyUnsizedOwned {
        unsized1: vec![PackedValue(1)],
        unsized2: SingleUnsizedOwned {
            unsized1: vec![PackedValue(2)],
        },
        unsized3: 3,
        unsized4: vec![TestStruct { val1: 4, val2: 5 }],
        unsized5: vec![TestStruct { val1: 6, val2: 7 }],
    };
    let owned = ManyUnsized::owned(r.immut()?)?;
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
    TestByteSet::<SingleUnsizedWithSized>::new(Zeroed)?;
    let r = TestByteSet::<SingleUnsizedWithSized>::new(SingleUnsizedWithSizedInit {
        sized1: true,
        unsized1: [PackedValue(1)],
    })?;
    let owned = SingleUnsizedWithSized::owned(r.immut()?)?;
    assert_eq!(
        owned,
        SingleUnsizedWithSizedOwned {
            sized1: true,
            unsized1: vec![PackedValue(1)],
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
    TestByteSet::<SizedAndUnsized>::new(Zeroed)?;
    let r = TestByteSet::<SizedAndUnsized>::new(SizedAndUnsizedInit {
        sized1: true,
        sized2: PackedValue(1),
        sized3: 2,
        sized4: [3; 10],
        unsized1: [PackedValue(4)],
        unsized2: [TestStruct { val1: 5, val2: 6 }],
        unsized3: 7,
    })?;
    let owned = SizedAndUnsized::owned(r.immut()?)?;
    assert_eq!(
        owned,
        SizedAndUnsizedOwned {
            sized1: true,
            sized2: PackedValue(1),
            sized3: 2,
            sized4: [3; 10],
            unsized1: vec![PackedValue(4)],
            unsized2: vec![TestStruct { val1: 5, val2: 6 }],
            unsized3: 7,
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
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
    TestByteSet::<WithSizedGenerics<TestStruct, bool>>::new(Zeroed)?;
    let r = TestByteSet::<WithSizedGenerics<TestStruct, bool>>::new(WithSizedGenericsInit {
        sized1: TestStruct { val1: 1, val2: 2 },
        sized2: true,
        sized3: 4,
        unsized1: [TestStruct { val1: 5, val2: 6 }],
        __phantom_generics: Default::default(),
    })?;
    let owned = WithSizedGenerics::owned(r.immut()?)?;
    assert_eq!(
        owned,
        WithSizedGenericsOwned {
            sized1: TestStruct { val1: 1, val2: 2 },
            sized2: true,
            sized3: 4,
            unsized1: vec![TestStruct { val1: 5, val2: 6 }],
            __phantom_generics: Default::default(),
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
    pub unsized2: CombinedUnsized<A, B>,
}

#[test]
fn test_with_unsized_generics() -> Result<()> {
    TestByteSet::<WithUnsizedGenerics<PackedValueChecked<u16>, TestStruct>>::new(Zeroed)?;
    let r = TestByteSet::<WithUnsizedGenerics<PackedValueChecked<u16>, TestStruct>>::new(
        WithUnsizedGenericsInit {
            sized1: 1,
            unsized1: [PackedValueChecked(2u16)],
            unsized2: (
                PackedValueChecked(u16::MAX),
                TestStruct { val1: 3, val2: 4 },
            ),
            __phantom_generics: Default::default(),
        },
    )?;
    let owned = WithUnsizedGenerics::owned(r.immut()?)?;
    assert_eq!(
        owned,
        WithUnsizedGenericsOwned {
            sized1: 1,
            unsized1: vec![PackedValueChecked(2u16)],
            unsized2: (
                PackedValueChecked(u16::MAX),
                TestStruct { val1: 3, val2: 4 }
            ),
            __phantom_generics: Default::default(),
        }
    );
    Ok(())
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct WithSizedAndUnsizedGenerics<A: UnsizedGenerics, B, C>
where
    B: UnsizedGenerics,
    C: CheckedBitPattern + Align1 + NoUninit + Zeroable,
{
    pub sized1: A,
    pub sized2: B,
    #[unsized_start]
    pub unsized1: List<A>,
    pub unsized2: CombinedUnsized<A, C>,
}

#[test]
fn test_with_sized_and_unsized_generics() -> Result<()> {
    TestByteSet::<
        WithSizedAndUnsizedGenerics<TestStruct, PackedValueChecked<u16>, PackedValueChecked<u32>>,
    >::new(Zeroed)?;
    let r = TestByteSet::<
        WithSizedAndUnsizedGenerics<TestStruct, PackedValueChecked<u16>, PackedValueChecked<u32>>,
    >::new(WithSizedAndUnsizedGenericsInit {
        sized1: TestStruct { val1: 1, val2: 2 },
        sized2: PackedValueChecked(3u16),
        unsized1: [TestStruct { val1: 4, val2: 5 }],
        unsized2: (
            TestStruct { val1: 6, val2: 7 },
            PackedValueChecked(u32::MAX / 4),
        ),
        __phantom_generics: Default::default(),
    })?;
    let owned = WithSizedAndUnsizedGenerics::owned(r.immut()?)?;
    assert_eq!(
        owned,
        WithSizedAndUnsizedGenericsOwned {
            sized1: TestStruct { val1: 1, val2: 2 },
            sized2: PackedValueChecked(3u16),
            unsized1: vec![TestStruct { val1: 4, val2: 5 }],
            unsized2: (
                TestStruct { val1: 6, val2: 7 },
                PackedValueChecked(u32::MAX / 4),
            ),
            __phantom_generics: Default::default(),
        }
    );
    Ok(())
}

//todo: make a single very complex struct and test it with a watcher on owned like list
