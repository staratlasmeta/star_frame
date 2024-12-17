#![allow(dead_code)]

use crate::prelude::*;
use crate::unsize::tests::test::TestStruct;
use std::mem::size_of;

#[unsized_type]
#[repr(u8)]
pub enum TestEnum<A: UnsizedGenerics> {
    A,
    B(List<A>) = 1,
    C(CombinedTest),
}

#[unsized_type]
pub struct CombinedTest {
    #[unsized_start]
    pub list1: List<u8>,
    pub list2: List<TestStruct>,
}

#[derive(Copy, Clone, Debug)]
pub struct TestEnumInitA<I>(pub I);
#[derive(Copy, Clone, Debug)]
pub struct TestEnumInitB<I>(pub I);

#[derive(Copy, Clone, Debug)]
pub struct TestEnumInitC<I>(pub I);
impl<I, A> UnsizedInit<TestEnumInitA<I>> for TestEnum<A>
where
    A: UnsizedGenerics,
    (): UnsizedInit<I>,
{
    const INIT_BYTES: usize =
        size_of::<TestEnumDiscriminant>() + <() as UnsizedInit<I>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: TestEnumInitA<I>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        TestEnumVariantA::init(super_ref, arg, |init_type| init_type.0)
    }
}
impl<I, A> UnsizedInit<TestEnumInitB<I>> for TestEnum<A>
where
    List<A>: UnsizedInit<I>,
    A: UnsizedGenerics,
{
    const INIT_BYTES: usize =
        size_of::<TestEnumDiscriminant>() + <List<A> as UnsizedInit<I>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: TestEnumInitB<I>,
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        TestEnumVariantB::init(super_ref, arg, |init_type| init_type.0)
    }
}

impl<A: UnsizedGenerics, I> UnsizedInit<TestEnumInitC<I>> for TestEnum<A>
where
    CombinedTest: UnsizedInit<I>,
{
    const INIT_BYTES: usize =
        size_of::<TestEnumDiscriminant>() + <CombinedTest as UnsizedInit<I>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        arg: TestEnumInitC<I>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        TestEnumVariantC::init(super_ref, arg, |init_type| init_type.0)
    }
}

pub trait TestEnumExt<A>:
    Sized + RefWrapperTypes<Ref = <TestEnum<A> as UnsizedType>::RefData>
where
    Self::Super: AsBytes,
    A: UnsizedGenerics,
{
    fn get(self) -> anyhow::Result<TestEnumRefWrapper<Self, A>>;

    fn discriminant(&self) -> TestEnumDiscriminant {
        TestEnum::discriminant(self)
    }

    fn set_a<I>(
        self,
        a_init: I,
    ) -> anyhow::Result<UnsizedEnumVariantRef<Self, TestEnumVariantA<A>>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        (): UnsizedInit<I>;
    fn set_b<I>(
        self,
        b_init: I,
    ) -> anyhow::Result<UnsizedEnumVariantRef<Self, TestEnumVariantB<A>>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        List<A>: UnsizedInit<I>;
    fn set_c<I>(
        self,
        c_init: I,
    ) -> anyhow::Result<UnsizedEnumVariantRef<Self, TestEnumVariantC<A>>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        CombinedTest: UnsizedInit<I>;
}
pub trait TestEnumMutExt<A>: TestEnumExt<A>
where
    Self::Super: AsBytes,
    A: UnsizedGenerics,
{
}

impl<R, A: UnsizedGenerics> TestEnumExt<A> for R
where
    R: RefWrapperTypes<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    R::Super: AsBytes,
{
    fn get(self) -> anyhow::Result<TestEnumRefWrapper<Self, A>> {
        match *self.r() {
            TestEnumMeta::A(m) => Ok(TestEnumRefWrapper::A(unsafe {
                TestEnumVariantA::get(self, m)?
            })),
            TestEnumMeta::B(m) => Ok(TestEnumRefWrapper::B(unsafe {
                TestEnumVariantB::get(self, m)?
            })),
            TestEnumMeta::C(m) => Ok(TestEnumRefWrapper::C(unsafe {
                TestEnumVariantC::get(self, m)?
            })),
        }
    }

    fn set_a<I>(self, a_init: I) -> anyhow::Result<UnsizedEnumVariantRef<Self, TestEnumVariantA<A>>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        (): UnsizedInit<I>,
    {
        TestEnumVariantA::set(self, a_init)
    }

    fn set_b<I>(self, b_init: I) -> anyhow::Result<UnsizedEnumVariantRef<Self, TestEnumVariantB<A>>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        List<A>: UnsizedInit<I>,
    {
        TestEnumVariantB::set(self, b_init)
    }

    fn set_c<I>(self, c_init: I) -> anyhow::Result<UnsizedEnumVariantRef<Self, TestEnumVariantC<A>>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        CombinedTest: UnsizedInit<I>,
    {
        TestEnumVariantC::set(self, c_init)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut bytes = TestByteSet::<TestEnum<u8>>::new(TestEnumInitA(()))?;
        assert_eq!(bytes.immut()?.discriminant(), TestEnumDiscriminant::A);
        {
            let mut mutable = bytes.mutable()?;
            {
                let b = (&mut mutable).set_b(Zeroed)?;
                assert_eq!(&**b, &[] as &[u8]);
            }
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::B);
            let mutable_b = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
                TestEnumRefWrapper::B(r) => r,
            };
            assert_eq!(&**mutable_b, &[] as &[u8]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
            TestEnumRefWrapper::B(r) => assert_eq!(&**r, &[] as &[u8]),
        };
        {
            let mutable = bytes.mutable()?;
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::B);
            let mut mutable_b = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
                TestEnumRefWrapper::B(r) => r,
            };
            mutable_b.push(0)?;
            assert_eq!(&**mutable_b, &[0]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::C(_) => unreachable!(),
            TestEnumRefWrapper::B(r) => assert_eq!(&**r, &[0]),
        };
        {
            let mut mutable = bytes.mutable()?;
            {
                let c = (&mut mutable).set_c(Zeroed)?;
                assert_eq!(&**(&c).list1()?, &[] as &[u8]);
                assert_eq!(&**c.list2()?, &[]);
            }
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::C);
            let mut mutable_c = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
                TestEnumRefWrapper::C(r) => r,
            };
            assert_eq!(&**(&mut mutable_c).list1()?, &[] as &[u8]);
            assert_eq!(&**mutable_c.list2()?, &[]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
            TestEnumRefWrapper::C(r) => {
                assert_eq!(&**(&r).list1()?, &[] as &[u8]);
                assert_eq!(&**(&r).list2()?, &[]);
            }
        };
        {
            let mutable = bytes.mutable()?;
            assert_eq!(mutable.discriminant(), TestEnumDiscriminant::C);
            let mut mutable_c = match mutable.get()? {
                TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
                TestEnumRefWrapper::C(r) => r,
            };
            (&mut mutable_c).list1()?.push(0)?;
            (&mut mutable_c)
                .list2()?
                .insert(0, TestStruct { val1: 1, val2: 0 })?;
            assert_eq!(&**(&mutable_c).list1()?, &[0]);
            assert_eq!(&**(&mutable_c).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
        }
        match bytes.immut()?.get()? {
            TestEnumRefWrapper::A(_) | TestEnumRefWrapper::B(_) => unreachable!(),
            TestEnumRefWrapper::C(r) => {
                assert_eq!(&**(&r).list1()?, &[0]);
                assert_eq!(&**(&r).list2()?, &[TestStruct { val1: 1, val2: 0 }]);
            }
        };

        Ok(())
    }
}
