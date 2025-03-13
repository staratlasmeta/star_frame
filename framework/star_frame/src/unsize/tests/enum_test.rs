use crate::prelude::*;
use crate::unsize::test_helpers::TestByteSet;
use crate::unsize::wrapper::StartPointer;
use advance::{Advance, AdvanceArray};
use anyhow::bail;
use bytemuck::bytes_of;
use star_frame_proc::derivative;
use std::mem::size_of;

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct Unsized1 {
    pub sized: u8,
    #[unsized_start]
    pub list: List<u8, u8>,
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct Unsized2 {
    pub sized: u16,
    #[unsized_start]
    pub list: List<PackedValue<u16>, u8>,
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct Unsized3 {
    pub sized: u32,
    #[unsized_start]
    pub unsized1: Unsized2,
}

// #[unsized_type]
// #[repr(u8)]
// pub enum UnsizedEnumTest {
//     #[default_init]
//     Unsized1(Unsized1),
//     Unsized2(Unsized2) = 2,
//     Unsized3(Unsized3),
// }

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)], skip_idl)]
pub struct EnumTestStruct {
    #[unsized_start]
    pub list_before: List<u8>,
    pub enum_test: UnsizedEnumTest,
    pub list_after: List<u8>,
}

#[derive(Debug)]
pub struct UnsizedEnumTest;

#[derive(Debug)]
#[repr(u8)]
pub enum UnsizedEnumTestDiscriminants {
    Unsized1,
    Unsized2 = 2,
    Unsized3,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum UnsizedEnumTestRef<'a> {
    Unsized1(<Unsized1 as UnsizedType>::Ref<'a>),
    Unsized2(<Unsized2 as UnsizedType>::Ref<'a>) = 2,
    Unsized3(<Unsized3 as UnsizedType>::Ref<'a>),
}

#[derive(Debug)]
#[repr(u8)]
pub enum UnsizedEnumTestMut<'a> {
    Unsized1(<Unsized1 as UnsizedType>::Mut<'a>),
    Unsized2(<Unsized2 as UnsizedType>::Mut<'a>) = 2,
    Unsized3(<Unsized3 as UnsizedType>::Mut<'a>),
}

impl<'l, 'as_shared> AsShared<'as_shared> for UnsizedEnumTestMut<'l>
where
    'l: 'as_shared,
{
    type Shared<'shared> = UnsizedEnumTestRef<'shared>
    where
        Self: 'shared;

    fn as_shared(&'as_shared self) -> Self::Shared<'as_shared> {
        match self {
            UnsizedEnumTestMut::Unsized1(unsized1) => {
                UnsizedEnumTestRef::Unsized1(unsized1.as_shared())
            }
            UnsizedEnumTestMut::Unsized2(unsized2) => {
                UnsizedEnumTestRef::Unsized2(unsized2.as_shared())
            }
            UnsizedEnumTestMut::Unsized3(unsized3) => {
                UnsizedEnumTestRef::Unsized3(unsized3.as_shared())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum UnsizedEnumTestOwned {
    Unsized1(<Unsized1 as UnsizedType>::Owned),
    Unsized2(<Unsized2 as UnsizedType>::Owned) = 2,
    Unsized3(<Unsized3 as UnsizedType>::Owned),
}

unsafe impl UnsizedType for UnsizedEnumTest {
    type Ref<'a> = UnsizedEnumTestRef<'a>;
    type Mut<'a> = StartPointer<UnsizedEnumTestMut<'a>>;
    type Owned = UnsizedEnumTestOwned;
    const ZST_STATUS: bool = {
        assert!(!(size_of::<u8>() == 0), "Zero sized types are not allowed as UnsizedEnum repr. Found ZST repr for `UnsizedEnumTest`");
        let _ = <Unsized1 as UnsizedType>::ZST_STATUS;
        let _ = <Unsized2 as UnsizedType>::ZST_STATUS;
        let _ = <Unsized3 as UnsizedType>::ZST_STATUS;
        true
    };

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        const UNSIZED_1_DISCRIMINANT: u8 = UnsizedEnumTestDiscriminants::Unsized1 as u8;
        const UNSIZED_2_DISCRIMINANT: u8 = UnsizedEnumTestDiscriminants::Unsized2 as u8;
        const UNSIZED_3_DISCRIMINANT: u8 = UnsizedEnumTestDiscriminants::Unsized3 as u8;
        let repr: u8 = <u8>::from_le_bytes(*data.try_advance_array()?);
        match repr {
            UNSIZED_1_DISCRIMINANT => Ok(UnsizedEnumTestRef::Unsized1(
                <Unsized1 as UnsizedType>::get_ref(data)?,
            )),
            UNSIZED_2_DISCRIMINANT => Ok(UnsizedEnumTestRef::Unsized2(
                <Unsized2 as UnsizedType>::get_ref(data)?,
            )),
            UNSIZED_3_DISCRIMINANT => Ok(UnsizedEnumTestRef::Unsized3(
                <Unsized3 as UnsizedType>::get_ref(data)?,
            )),
            _ => bail!("Invalid UnsizedEnum repr",),
        }
    }

    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>> {
        const UNSIZED_2_DISCRIMINANT: u8 = UnsizedEnumTestDiscriminants::Unsized2 as u8;
        const UNSIZED_1_DISCRIMINANT: u8 = UnsizedEnumTestDiscriminants::Unsized1 as u8;
        const UNSIZED_3_DISCRIMINANT: u8 = UnsizedEnumTestDiscriminants::Unsized3 as u8;
        let start_ptr = data.as_mut_ptr().cast_const().cast::<()>();
        let repr: u8 = <u8>::from_le_bytes(*data.try_advance_array()?);
        let res = match repr {
            UNSIZED_1_DISCRIMINANT => {
                UnsizedEnumTestMut::Unsized1(<Unsized1 as UnsizedType>::get_mut(data)?)
            }
            UNSIZED_2_DISCRIMINANT => {
                UnsizedEnumTestMut::Unsized2(<Unsized2 as UnsizedType>::get_mut(data)?)
            }
            UNSIZED_3_DISCRIMINANT => {
                UnsizedEnumTestMut::Unsized3(<Unsized3 as UnsizedType>::get_mut(data)?)
            }
            _ => bail!("Invalid UnsizedEnum repr",),
        };
        Ok(unsafe { StartPointer::new(start_ptr, res) })
    }

    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned> {
        match r {
            UnsizedEnumTestRef::Unsized1(unsized1) => Ok(UnsizedEnumTestOwned::Unsized1(
                <Unsized1 as UnsizedType>::owned_from_ref(unsized1)?,
            )),
            UnsizedEnumTestRef::Unsized2(unsized2) => Ok(UnsizedEnumTestOwned::Unsized2(
                <Unsized2 as UnsizedType>::owned_from_ref(unsized2)?,
            )),
            UnsizedEnumTestRef::Unsized3(unsized3) => Ok(UnsizedEnumTestOwned::Unsized3(
                <Unsized3 as UnsizedType>::owned_from_ref(unsized3)?,
            )),
        }
    }

    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()> {
        unsafe { Self::Mut::handle_resize_notification(self_mut, source_ptr, change) };
        match &mut self_mut.data {
            UnsizedEnumTestMut::Unsized1(unsized1) => {
                unsafe {
                    <Unsized1 as UnsizedType>::resize_notification(unsized1, source_ptr, change)
                }?;
            }
            UnsizedEnumTestMut::Unsized2(unsized2) => {
                unsafe {
                    <Unsized2 as UnsizedType>::resize_notification(unsized2, source_ptr, change)
                }?;
            }
            UnsizedEnumTestMut::Unsized3(unsized3) => {
                unsafe {
                    <Unsized3 as UnsizedType>::resize_notification(unsized3, source_ptr, change)
                }?;
            }
        }
        Ok(())
    }
}

#[allow(trivial_bounds)]
impl UnsizedInit<DefaultInit> for UnsizedEnumTest
where
    Unsized1: UnsizedInit<DefaultInit>,
{
    const INIT_BYTES: usize = <Unsized1 as UnsizedInit<DefaultInit>>::INIT_BYTES
        + std::mem::size_of::<UnsizedEnumTestDiscriminants>();

    unsafe fn init(bytes: &mut &mut [u8], arg: DefaultInit) -> Result<()> {
        bytes
            .try_advance(size_of::<UnsizedEnumTestDiscriminants>())?
            .copy_from_slice(bytes_of(&(UnsizedEnumTestDiscriminants::Unsized1 as u8)));
        unsafe { <Unsized1 as UnsizedInit<DefaultInit>>::init(bytes, arg)? };
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct UnsizedEnumTestInitUnsized1<Init>(pub Init);

impl<Init> UnsizedInit<UnsizedEnumTestInitUnsized1<Init>> for UnsizedEnumTest
where
    Unsized1: UnsizedInit<Init>,
{
    const INIT_BYTES: usize =
        <Unsized1 as UnsizedInit<Init>>::INIT_BYTES + size_of::<UnsizedEnumTestDiscriminants>();

    unsafe fn init(bytes: &mut &mut [u8], arg: UnsizedEnumTestInitUnsized1<Init>) -> Result<()> {
        bytes
            .try_advance(size_of::<UnsizedEnumTestDiscriminants>())?
            .copy_from_slice(bytes_of(&(UnsizedEnumTestDiscriminants::Unsized1 as u8)));
        unsafe { <Unsized1 as UnsizedInit<Init>>::init(bytes, arg.0)? };
        Ok(())
    }
}
#[derive(Copy, Clone, Debug, Default)]
pub struct UnsizedEnumTestInitUnsized2<Init>(pub Init);

impl<Init> UnsizedInit<UnsizedEnumTestInitUnsized2<Init>> for UnsizedEnumTest
where
    Unsized2: UnsizedInit<Init>,
{
    const INIT_BYTES: usize =
        <Unsized2 as UnsizedInit<Init>>::INIT_BYTES + size_of::<UnsizedEnumTestDiscriminants>();

    unsafe fn init(bytes: &mut &mut [u8], arg: UnsizedEnumTestInitUnsized2<Init>) -> Result<()> {
        bytes
            .try_advance(size_of::<UnsizedEnumTestDiscriminants>())?
            .copy_from_slice(bytes_of(&(UnsizedEnumTestDiscriminants::Unsized2 as u8)));
        unsafe { <Unsized2 as UnsizedInit<Init>>::init(bytes, arg.0)? };
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct UnsizedEnumTestInitUnsized3<Init>(pub Init);

impl<Init> UnsizedInit<UnsizedEnumTestInitUnsized3<Init>> for UnsizedEnumTest
where
    Unsized3: UnsizedInit<Init>,
{
    const INIT_BYTES: usize =
        <Unsized3 as UnsizedInit<Init>>::INIT_BYTES + size_of::<UnsizedEnumTestDiscriminants>();

    unsafe fn init(bytes: &mut &mut [u8], arg: UnsizedEnumTestInitUnsized3<Init>) -> Result<()> {
        bytes
            .try_advance(size_of::<UnsizedEnumTestDiscriminants>())?
            .copy_from_slice(bytes_of(&(UnsizedEnumTestDiscriminants::Unsized3 as u8)));
        unsafe { <Unsized3 as UnsizedInit<Init>>::init(bytes, arg.0)? };
        Ok(())
    }
}

trait UnsizedEnumTestExclusiveExt<'b, 'a, 'info, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
{
    fn get<'c>(&'c mut self) -> UnsizedEnumTestExclusive<'c, 'a, 'info, O, A>;
    fn set_unsized1<Init>(&mut self, init: Init) -> Result<()>
    where
        UnsizedEnumTest: UnsizedInit<UnsizedEnumTestInitUnsized1<Init>>;
    fn set_unsized2<Init>(&mut self, init: Init) -> Result<()>
    where
        UnsizedEnumTest: UnsizedInit<UnsizedEnumTestInitUnsized2<Init>>;
    fn set_unsized3<Init>(&mut self, init: Init) -> Result<()>
    where
        UnsizedEnumTest: UnsizedInit<UnsizedEnumTestInitUnsized3<Init>>;
}
impl<'b, 'a, 'info, O, A> UnsizedEnumTestExclusiveExt<'b, 'a, 'info, O, A>
    for ExclusiveWrapperT<'b, 'a, 'info, UnsizedEnumTest, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
{
    fn get<'c>(&'c mut self) -> UnsizedEnumTestExclusive<'c, 'a, 'info, O, A> {
        unsafe {
            match &***self {
                UnsizedEnumTestMut::Unsized1(_) => {
                    UnsizedEnumTestExclusive::Unsized1(ExclusiveWrapper::map_ref(self, |a| {
                        match &mut **a {
                            UnsizedEnumTestMut::Unsized1(a) => a,
                            _ => unreachable!(),
                        }
                    }))
                }
                UnsizedEnumTestMut::Unsized2(_) => {
                    UnsizedEnumTestExclusive::Unsized2(ExclusiveWrapper::map_ref(self, |a| {
                        match &mut **a {
                            UnsizedEnumTestMut::Unsized2(a) => a,
                            _ => unreachable!(),
                        }
                    }))
                }
                UnsizedEnumTestMut::Unsized3(_) => {
                    UnsizedEnumTestExclusive::Unsized3(ExclusiveWrapper::map_ref(self, |a| {
                        match &mut **a {
                            UnsizedEnumTestMut::Unsized3(a) => a,
                            _ => unreachable!(),
                        }
                    }))
                }
            }
        }
    }

    fn set_unsized1<Init>(&mut self, init: Init) -> Result<()>
    where
        UnsizedEnumTest: UnsizedInit<UnsizedEnumTestInitUnsized1<Init>>,
    {
        unsafe {
            ExclusiveWrapper::set_start_pointer_data::<UnsizedEnumTest, _>(
                self,
                UnsizedEnumTestInitUnsized1(init),
            )
        }
    }

    fn set_unsized2<Init>(&mut self, init: Init) -> Result<()>
    where
        UnsizedEnumTest: UnsizedInit<UnsizedEnumTestInitUnsized2<Init>>,
    {
        unsafe {
            ExclusiveWrapper::set_start_pointer_data::<UnsizedEnumTest, _>(
                self,
                UnsizedEnumTestInitUnsized2(init),
            )
        }
    }

    fn set_unsized3<Init>(&mut self, init: Init) -> Result<()>
    where
        UnsizedEnumTest: UnsizedInit<UnsizedEnumTestInitUnsized3<Init>>,
    {
        unsafe {
            ExclusiveWrapper::set_start_pointer_data::<UnsizedEnumTest, _>(
                self,
                UnsizedEnumTestInitUnsized3(init),
            )
        }
    }
}

#[derive(Debug)]
enum UnsizedEnumTestExclusive<'b, 'a, 'info, O, A>
where
    O: UnsizedType + ?Sized,
    A: UnsizedTypeDataAccess<'info>,
{
    Unsized1(ExclusiveWrapperT<'b, 'a, 'info, Unsized1, O, A>),
    Unsized2(ExclusiveWrapperT<'b, 'a, 'info, Unsized2, O, A>),
    Unsized3(ExclusiveWrapperT<'b, 'a, 'info, Unsized3, O, A>),
}

#[test]
fn unsized_enum_test() -> Result<()> {
    let bytes = TestByteSet::<EnumTestStruct>::new(DefaultInit)?;
    let mut mut_bytes = bytes.data_mut()?;
    let mut exclusive = mut_bytes.exclusive();
    exclusive.list_before().push(100)?;
    if let UnsizedEnumTestExclusive::Unsized1(mut a) = exclusive.enum_test().get() {
        a.list().push(150)?;
        a.list().push(151)?;
        a.sized = 10;
    } else {
        bail!("Expected Unsized1");
    };

    todo!("make owned struct and function to ensure all the derefs and everything are 1:1 with it");
    // assert_eq!(**exclusive.list_before, vec![100]);
    // assert_eq!(**exclusive.list_after, Vec::<u8>::new());
    // if let UnsizedEnumTestMut::Unsized1(unsized1) = &*exclusive.enum_test {
    //     assert_eq!(unsized1.sized, 10);
    //     assert_eq!(**unsized1.list, vec![150, 151]);
    // } else {
    //     bail!("Expected Unsized1");
    // };
    //
    // // let other = exclusive
    // // assert_eq!(*exclusive, )
    // //
    // // exclusive.list1().push(202)?;
    // // exclusive.list1().push(203)?;
    // // exclusive.list1().push(204)?;
    // // exclusive.enum_test().set_unsized2(DefaultInit)?;
    // // exclusive.enum_test().set_unsized1(DefaultInit)?;
    // // exclusive.enum_test().set_unsized3(DefaultInit)?;
    // //
    // // if let UnsizedEnumTestMut::Unsized1(mut_thing) = &*mut_bytes.enum_test {
    // //     let list = &mut_thing.list;
    // //     let shared = list.as_shared();
    // //     println!("Shared: {:?}", &*shared);
    // // }
    // //
    // // println!(
    // //     "{:?}",
    // //     EnumTestStruct::owned_from_ref(mut_bytes.as_shared())
    // // );
    //
    // Ok(())
}
