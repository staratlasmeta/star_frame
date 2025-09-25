use crate::{
    prelude::*,
    unsize::{test_helpers::TestByteSet, TestUnderlyingData},
};

#[unsized_type(skip_idl)]
pub struct Unsized1 {
    pub sized: u8,
    #[unsized_start]
    pub list: List<u8, u8>,
}

#[unsized_type(skip_idl)]
pub struct Unsized2 {
    pub sized: PackedValue<u16>,
    #[unsized_start]
    pub list: List<PackedValue<u16>, u8>,
}

#[unsized_type(skip_idl)]
pub struct Unsized3 {
    pub sized: PackedValue<u32>,
    #[unsized_start]
    pub unsized1: Unsized2,
}

#[unsized_type(skip_idl)]
pub struct EnumTestStruct {
    #[unsized_start]
    pub list_before: List<u8>,
    pub enum_test: UnsizedEnumTest<Unsized1>,
    pub list_after: List<u8>,
}

#[unsized_type(skip_idl)]
#[repr(u16)]
pub enum UnsizedEnumTest<T: UnsizedType + Debug + ?Sized> {
    #[default_init]
    Unsized1(T),
    Unsized2(Unsized2) = 2,
    Unsized3(Unsized3),
    Unsized4,
}

fn compare_with_owned(
    owned: &EnumTestStructOwned,
    exclusive: &ExclusiveWrapperTop<EnumTestStruct, TestUnderlyingData>,
) {
    assert_eq!(**exclusive.list_before, owned.list_before);
    assert_eq!(**exclusive.list_after, owned.list_after);
    match &*exclusive.enum_test {
        UnsizedEnumTestMut::Unsized1(unsized1) => {
            let UnsizedEnumTestOwned::Unsized1(unsized1_owned) = &owned.enum_test else {
                panic!("Expected Unsized1");
            };
            assert_eq!(unsized1.sized, unsized1_owned.sized,);
            assert_eq!(**unsized1.list, unsized1_owned.list,);
        }
        UnsizedEnumTestMut::Unsized2(unsized2) => {
            let UnsizedEnumTestOwned::Unsized2(unsized2_owned) = &owned.enum_test else {
                panic!("Expected Unsized2");
            };
            assert_eq!(unsized2.sized, unsized2_owned.sized);
            assert_eq!(**unsized2.list, unsized2_owned.list);
        }
        UnsizedEnumTestMut::Unsized3(unsized3) => {
            let UnsizedEnumTestOwned::Unsized3(unsized3_owned) = &owned.enum_test else {
                panic!("Expected Unsized3");
            };
            assert_eq!(unsized3.sized, unsized3_owned.sized);
            assert_eq!(unsized3.unsized1.sized, unsized3_owned.unsized1.sized);
            assert_eq!(**unsized3.unsized1.list, unsized3_owned.unsized1.list);
        }
        UnsizedEnumTestMut::Unsized4 => {
            let UnsizedEnumTestOwned::Unsized4 = &owned.enum_test else {
                panic!("Expected Unsized4");
            };
        }
    }
}

#[test]
fn unsized_enum_test() -> Result<()> {
    let bytes = TestByteSet::<EnumTestStruct>::new_default()?;
    let mut owned = EnumTestStructOwned {
        list_before: vec![],
        enum_test: UnsizedEnumTestOwned::Unsized1(Unsized1Owned {
            sized: 0,
            list: vec![],
        }),
        list_after: vec![],
    };

    let mut mut_bytes = bytes.data_mut()?;

    mut_bytes.list_before().push(100)?;
    owned.list_before.push(100);
    compare_with_owned(&owned, &mut_bytes);
    if let UnsizedEnumTestExclusive::Unsized1(mut a) = mut_bytes.enum_test().get() {
        a.list().push(150)?;
        a.list().push(151)?;
        a.sized = 10;
    } else {
        panic!("Expected Unsized1");
    }
    if let UnsizedEnumTestOwned::Unsized1(unsized1) = &mut owned.enum_test {
        unsized1.list.push(150);
        unsized1.list.push(151);
        unsized1.sized = 10;
    } else {
        panic!("Expected Unsized1Owned");
    }
    compare_with_owned(&owned, &mut_bytes);

    mut_bytes.list_after().push(202)?;
    mut_bytes.list_after().push(203)?;
    mut_bytes.list_after().push(204)?;
    owned.list_after.push(202);
    owned.list_after.push(203);
    owned.list_after.push(204);
    compare_with_owned(&owned, &mut_bytes);

    let mut enum_test = mut_bytes.enum_test();
    let mut unsized3 = enum_test.set_unsized3(DefaultInit)?;
    owned.enum_test = UnsizedEnumTestOwned::Unsized3(Unsized3Owned {
        sized: 0.into(),
        unsized1: Unsized2Owned {
            sized: 0.into(),
            list: vec![],
        },
    });
    {
        unsized3.unsized1().list().push(151.into())?;
        unsized3.unsized1().list().insert(0, 150.into())?;
        unsized3.unsized1.sized = 30.into();
        if let UnsizedEnumTestOwned::Unsized3(unsized3) = &mut owned.enum_test {
            unsized3.unsized1.list.push(151.into());
            unsized3.unsized1.list.insert(0, 150.into());
            unsized3.unsized1.sized = 30.into();
        } else {
            panic!("Expected Unsized3Owned");
        }
        compare_with_owned(&owned, &mut_bytes);
    }
    mut_bytes.list_before().insert(0, 190)?;
    mut_bytes.list_after().insert(2, 200)?;
    owned.list_before.insert(0, 190);
    owned.list_after.insert(2, 200);
    compare_with_owned(&owned, &mut_bytes);

    mut_bytes.enum_test().set_unsized2(Unsized2Init {
        sized: Unsized2Sized { sized: 426.into() },
        list: [1, 2, 3, 4, 5].map(Into::into),
    })?;
    owned.enum_test = UnsizedEnumTestOwned::Unsized2(Unsized2Owned {
        sized: 426.into(),
        list: [1, 2, 3, 4, 5].map(Into::into).to_vec(),
    });
    compare_with_owned(&owned, &mut_bytes);

    if let UnsizedEnumTestExclusive::Unsized2(mut a) = mut_bytes.enum_test().get() {
        a.list().insert(0, 0.into())?;
    } else {
        panic!("Expected Unsized2");
    }
    if let UnsizedEnumTestOwned::Unsized2(unsized2) = &mut owned.enum_test {
        unsized2.list.insert(0, 0.into());
    } else {
        panic!("Expected Unsized2Owned");
    }
    compare_with_owned(&owned, &mut_bytes);

    let new_owned = EnumTestStruct::owned_from_ref(&EnumTestStruct::mut_as_ref(&mut_bytes))?;
    compare_with_owned(&new_owned, &mut_bytes);
    assert_eq!(owned, new_owned);
    Ok(())
}
