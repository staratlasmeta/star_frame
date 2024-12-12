#![allow(dead_code)]

use crate::prelude::*;
use crate::unsize::tests::test::TestStruct;
use crate::util::OffsetRef;
use advance::Advance;
use bytemuck::checked::try_from_bytes;
use bytemuck::{bytes_of, CheckedBitPattern, NoUninit};
use std::marker::PhantomData;
use std::mem::size_of;
use typenum::True;
// #[repr(u8)]
// pub enum TestEnum {
//     A(Inner) = 4,
// }

#[unsized_type]
pub struct Inner {
    #[unsized_start]
    pub list1: List<u8>,
}

unsafe impl UnsizedEnumVariant for TestEnumVariantA {
    type UnsizedEnum = TestEnum;
    type InnerType = Inner;
    const DISCRIMINANT: <Self::UnsizedEnum as UnsizedEnum>::Discriminant = TestEnumDiscriminant::A;

    fn new() -> Self {
        Self(PhantomData)
    }

    fn new_meta(
        meta: <Self::InnerType as UnsizedType>::RefMeta,
    ) -> <Self::UnsizedEnum as UnsizedType>::RefMeta {
        TestEnumMeta::A(meta)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, CheckedBitPattern, NoUninit)]
#[repr(u8)]
pub enum TestEnumDiscriminant {
    A = 4,
}
pub type AInner = Inner;

#[derive(Copy, Clone)]
pub enum TestEnumMeta {
    A(<AInner as UnsizedType>::RefMeta),
}

pub enum TestEnumOwned {
    A(<AInner as UnsizedType>::Owned),
}

#[repr(transparent)]
pub struct TestEnum {
    __phantom_generics: PhantomData<()>,
    data: [u8],
}
unsafe impl UnsizedType for TestEnum {
    type RefMeta = TestEnumMeta;
    type RefData = TestEnumMeta;
    type Owned = TestEnumOwned;
    type IsUnsized = True;

    fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        let repr: &TestEnumDiscriminant =
            try_from_bytes(&super_ref.as_bytes()?[..size_of::<TestEnumDiscriminant>()])?;

        match repr {
            TestEnumDiscriminant::A => unsafe { TestEnumVariantA::from_bytes(super_ref) },
        }
    }

    unsafe fn from_bytes_and_meta<S: AsBytes>(
        super_ref: S,
        meta: Self::RefMeta,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        match meta {
            TestEnumMeta::A(m) => unsafe {
                TestEnumVariantA::from_bytes_and_meta(super_ref, meta, m)
            },
        }
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
        match r.get()? {
            TestEnumRefWrapper::A(r) => AInner::owned(r).map(TestEnumOwned::A),
        }
    }
}
#[automatically_derived]
#[allow(clippy::ignored_unit_patterns)]
impl UnsizedEnum for TestEnum {
    type Discriminant = TestEnumDiscriminant;

    fn discriminant<S: AsBytes>(
        r: &impl RefWrapperTypes<Super = S, Ref = Self::RefData>,
    ) -> Self::Discriminant {
        match r.r() {
            TestEnumMeta::A(_) => TestEnumVariantA::DISCRIMINANT,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TestEnumInitA<I>(pub I);

impl<I> UnsizedInit<TestEnumInitA<I>> for TestEnum
where
    AInner: UnsizedInit<I>,
{
    const INIT_BYTES: usize =
        size_of::<TestEnumDiscriminant>() + <AInner as UnsizedInit<I>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        arg: TestEnumInitA<I>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        super_ref.as_mut_bytes()?[..size_of::<TestEnumDiscriminant>()]
            .copy_from_slice(bytes_of(&TestEnumDiscriminant::A));
        let (_, ref_meta) = unsafe {
            <AInner as UnsizedInit<I>>::init(
                RefWrapper::new(&mut super_ref, OffsetRef(size_of::<TestEnumDiscriminant>())),
                arg.0,
            )?
        };
        let meta = TestEnumMeta::A(ref_meta);
        Ok((unsafe { RefWrapper::new(super_ref, meta) }, meta))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TestEnumVariantA(PhantomData<()>);
impl TestEnumVariantA {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

// pub type ARef<S> = RefWrapper<RefWrapper<S, TestEnumVariantA>, <AInner as UnsizedType>::RefData>;
pub type ARef<S> = UnsizedEnumVariantRef<S, TestEnumVariantA>;

#[derive(Copy, Clone)]
pub enum TestEnumRefWrapper<S> {
    A(ARef<S>),
}
pub trait TestEnumExt: Sized + RefWrapperTypes<Ref = <TestEnum as UnsizedType>::RefData>
where
    Self::Super: AsBytes,
{
    fn get(self) -> anyhow::Result<TestEnumRefWrapper<Self>>;

    fn discriminant(&self) -> TestEnumDiscriminant {
        TestEnum::discriminant(self)
    }

    fn set_a<I>(self, a_init: I) -> anyhow::Result<ARef<Self>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum as UnsizedType>::RefMeta>,
        AInner: UnsizedInit<I>;
}
pub trait TestEnumMutExt: TestEnumExt
where
    Self::Super: AsBytes,
{
}

impl<R> TestEnumExt for R
where
    R: RefWrapperTypes<Ref = <TestEnum as UnsizedType>::RefData>,
    R::Super: AsBytes,
{
    fn get(self) -> anyhow::Result<TestEnumRefWrapper<Self>> {
        match *self.r() {
            TestEnumMeta::A(m) => Ok(TestEnumRefWrapper::A(unsafe {
                TestEnumVariantA::get(self, m)?
            })),
        }
    }

    fn set_a<I>(self, a_init: I) -> anyhow::Result<ARef<Self>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum as UnsizedType>::RefMeta>,
        AInner: UnsizedInit<I>,
    {
        TestEnumVariantA::set(self, a_init)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        // let mut bytes = TestByteSet::<TestEnum<u8>>::new(TestEnumInitA(()))?;
        // assert_eq!(bytes.immut()?.discriminant(), TestEnumDiscriminant::A);
        // {
        //     let mut mutable = bytes.mutable()?;
        //     {
        //         let b = (&mut mutable).set_b(Zeroed)?;
        //         assert_eq!(&**b, &[] as &[u8]);
        //     }
        //     assert_eq!(mutable.discriminant(), TestEnumDiscriminant::B);
        //     let mutable_b = match mutable.get()? {
        //         TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
        //         TestEnumRefWrapper::B(r) => r,
        //     };
        //     assert_eq!(&**mutable_b, &[] as &[u8]);
        // }
        // match bytes.immut()?.get()? {
        //     TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
        //     TestEnumRefWrapper::B(r) => assert_eq!(&**r, &[] as &[u8]),
        // };
        // {
        //     let mutable = bytes.mutable()?;
        //     assert_eq!(mutable.discriminant(), TestEnumDiscriminant::B);
        //     let mut mutable_b = match mutable.get()? {
        //         TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
        //         TestEnumRefWrapper::B(r) => r,
        //     };
        //     mutable_b.push(0)?;
        //     assert_eq!(&**mutable_b, &[0]);
        // }
        // match bytes.immut()?.get()? {
        //     TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
        //     TestEnumRefWrapper::B(r) => assert_eq!(&**r, &[0]),
        // };
        // {
        //     let mut mutable = bytes.mutable()?;
        //     {
        //         let c = (&mut mutable).set_c(Zeroed)?;
        //         assert_eq!(&**(&c).list1()?, &[] as &[u8]);
        //         assert_eq!(&**c.list2()?, &[]);
        //     }
        //     assert_eq!(mutable.discriminant(), TestEnumDiscriminant::C);
        //     let mut mutable_c = match mutable.get()? {
        //         TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
        //         TestEnumRefWrapper::C(r) => r,
        //     };
        //     assert_eq!(&**(&mut mutable_c).list1()?, &[] as &[u8]);
        //     assert_eq!(&**mutable_c.list2()?, &[]);
        // }
        // match bytes.immut()?.get()? {
        //     TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
        //     TestEnumRefWrapper::C(r) => {
        //         assert_eq!(&**(&r).list1()?, &[] as &[u8]);
        //         assert_eq!(&**(&r).list2()?, &[]);
        //     }
        // };
        // {
        //     let mutable = bytes.mutable()?;
        //     assert_eq!(mutable.discriminant(), TestEnumDiscriminant::C);
        //     let mut mutable_c = match mutable.get()? {
        //         TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
        //         TestEnumRefWrapper::C(r) => r,
        //     };
        //     (&mut mutable_c).list1()?.push(0)?;
        //     (&mut mutable_c)
        //         .list2()?
        //         .insert(0, TestStruct { val1: 1, val2: 0 })?;
        //     assert_eq!(&**(&mutable_c).list1()?, &[0]);
        //     assert_eq!(&**(&mutable_c).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
        // }
        // match bytes.immut()?.get()? {
        //     TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
        //     TestEnumRefWrapper::C(r) => {
        //         assert_eq!(&**(&r).list1()?, &[0]);
        //         assert_eq!(&**(&r).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
        //     }
        // };

        Ok(())
    }
}
