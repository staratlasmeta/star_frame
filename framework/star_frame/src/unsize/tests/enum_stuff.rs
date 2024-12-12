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
//     A(()),
//     B(List<u8>) = 4,
//     C(CombinedTest),
// }

#[unsized_type]
pub struct CombinedTest {
    #[unsized_start]
    pub list1: List<u8>,
    pub list2: List<TestStruct>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, CheckedBitPattern, NoUninit)]
#[repr(u8)]
pub enum TestEnumDiscriminant {
    A,
    B = 4,
    C,
}
pub type AInner = ();
pub type BInner<A> = List<A>;
pub type CInner = CombinedTest;

#[derive(Copy, Clone)]
pub enum TestEnumMeta<A: UnsizedGenerics> {
    A(<AInner as UnsizedType>::RefMeta),
    B(<BInner<A> as UnsizedType>::RefMeta),
    C(<CInner as UnsizedType>::RefMeta),
}

pub enum TestEnumOwned<A: UnsizedGenerics> {
    A(<AInner as UnsizedType>::Owned),
    B(<BInner<A> as UnsizedType>::Owned),
    C(<CInner as UnsizedType>::Owned),
}

#[repr(transparent)]
pub struct TestEnum<A> {
    __phantom_generics: PhantomData<fn() -> (A)>,
    data: [u8],
}
unsafe impl<A: UnsizedGenerics> UnsizedType for TestEnum<A> {
    type RefMeta = TestEnumMeta<A>;
    type RefData = TestEnumMeta<A>;
    type Owned = TestEnumOwned<A>;
    type IsUnsized = True;

    fn from_bytes<S: AsBytes>(
        super_ref: S,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        let repr: &TestEnumDiscriminant =
            try_from_bytes(&super_ref.as_bytes()?[..size_of::<TestEnumDiscriminant>()])?;

        match repr {
            TestEnumDiscriminant::A => {
                let FromBytesReturn {
                    bytes_used, meta, ..
                } = unsafe {
                    <AInner as UnsizedType>::from_bytes(RefWrapper::new(
                        &super_ref,
                        OffsetRef(size_of::<TestEnumDiscriminant>()),
                    ))?
                };
                let meta = TestEnumMeta::A(meta);
                Ok(FromBytesReturn {
                    bytes_used: bytes_used + size_of::<TestEnumDiscriminant>(),
                    meta,
                    ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
                })
            }
            TestEnumDiscriminant::B => {
                let FromBytesReturn {
                    bytes_used, meta, ..
                } = unsafe {
                    <BInner<A> as UnsizedType>::from_bytes(RefWrapper::new(
                        &super_ref,
                        OffsetRef(size_of::<TestEnumDiscriminant>()),
                    ))?
                };
                let meta = TestEnumMeta::B(meta);
                Ok(FromBytesReturn {
                    bytes_used: bytes_used + size_of::<TestEnumDiscriminant>(),
                    meta,
                    ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
                })
            }
            TestEnumDiscriminant::C => {
                let FromBytesReturn {
                    bytes_used, meta, ..
                } = unsafe {
                    <CInner as UnsizedType>::from_bytes(RefWrapper::new(
                        &super_ref,
                        OffsetRef(size_of::<TestEnumDiscriminant>()),
                    ))?
                };
                let meta = TestEnumMeta::C(meta);
                Ok(FromBytesReturn {
                    bytes_used: bytes_used + size_of::<TestEnumDiscriminant>(),
                    meta,
                    ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
                })
            }
        }
    }

    unsafe fn from_bytes_and_meta<S: AsBytes>(
        super_ref: S,
        meta: Self::RefMeta,
    ) -> anyhow::Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        match meta {
            TestEnumMeta::A(m) => {
                let FromBytesReturn { bytes_used, .. } =
                    unsafe { <AInner as UnsizedType>::from_bytes_and_meta(&super_ref, m)? };
                Ok(FromBytesReturn {
                    bytes_used,
                    meta,
                    ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
                })
            }
            TestEnumMeta::B(m) => {
                let FromBytesReturn { bytes_used, .. } =
                    unsafe { <BInner<A> as UnsizedType>::from_bytes_and_meta(&super_ref, m)? };
                Ok(FromBytesReturn {
                    bytes_used,
                    meta,
                    ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
                })
            }
            TestEnumMeta::C(m) => {
                let FromBytesReturn { bytes_used, .. } =
                    unsafe { <CInner as UnsizedType>::from_bytes_and_meta(&super_ref, m)? };
                Ok(FromBytesReturn {
                    bytes_used,
                    meta,
                    ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
                })
            }
        }
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> anyhow::Result<Self::Owned> {
        match r.get()? {
            TestEnumRefWrapper::A(r) => AInner::owned(r).map(TestEnumOwned::A),
            TestEnumRefWrapper::B(r) => BInner::owned(r).map(TestEnumOwned::B),
            TestEnumRefWrapper::C(r) => CInner::owned(r).map(TestEnumOwned::C),
        }
    }
}
#[automatically_derived]
#[allow(clippy::ignored_unit_patterns)]
impl<A: UnsizedGenerics> UnsizedEnum for TestEnum<A> {
    type Discriminant = TestEnumDiscriminant;

    fn discriminant<S: AsBytes>(
        r: &impl RefWrapperTypes<Super = S, Ref = Self::RefData>,
    ) -> Self::Discriminant {
        match r.r() {
            TestEnumMeta::A(_) => Self::Discriminant::A,
            TestEnumMeta::B(_) => Self::Discriminant::B,
            TestEnumMeta::C(_) => Self::Discriminant::C,
        }
    }
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
impl<I, A> UnsizedInit<TestEnumInitB<I>> for TestEnum<A>
where
    BInner<A>: UnsizedInit<I>,
    A: UnsizedGenerics,
{
    const INIT_BYTES: usize =
        size_of::<TestEnumDiscriminant>() + <BInner<A> as UnsizedInit<I>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        arg: TestEnumInitB<I>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        super_ref.as_mut_bytes()?[..size_of::<TestEnumDiscriminant>()]
            .copy_from_slice(bytes_of(&TestEnumDiscriminant::B));
        let (_, ref_meta) = unsafe {
            <BInner<A> as UnsizedInit<I>>::init(
                RefWrapper::new(&mut super_ref, OffsetRef(size_of::<TestEnumDiscriminant>())),
                arg.0,
            )?
        };
        let meta = TestEnumMeta::B(ref_meta);
        Ok((unsafe { RefWrapper::new(super_ref, meta) }, meta))
    }
}
impl<A: UnsizedGenerics, I> UnsizedInit<TestEnumInitC<I>> for TestEnum<A>
where
    CInner: UnsizedInit<I>,
{
    const INIT_BYTES: usize =
        size_of::<TestEnumDiscriminant>() + <CInner as UnsizedInit<I>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        arg: TestEnumInitC<I>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        super_ref.as_mut_bytes()?[..size_of::<TestEnumDiscriminant>()]
            .copy_from_slice(bytes_of(&TestEnumDiscriminant::C));
        let (_, ref_meta) = unsafe {
            <CInner as UnsizedInit<I>>::init(
                RefWrapper::new(&mut super_ref, OffsetRef(size_of::<TestEnumDiscriminant>())),
                arg.0,
            )?
        };
        let meta = TestEnumMeta::C(ref_meta);
        Ok((unsafe { RefWrapper::new(super_ref, meta) }, meta))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TestEnumVariantA<A>(PhantomData<fn() -> A>);
impl<A> TestEnumVariantA<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
#[derive(Copy, Clone, Debug)]
pub struct TestEnumVariantB<A>(PhantomData<fn() -> A>);
impl<A> TestEnumVariantB<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
#[derive(Copy, Clone, Debug)]
pub struct TestEnumVariantC<A>(PhantomData<fn() -> A>);
impl<A> TestEnumVariantC<A> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

unsafe impl<S, A> RefBytes<S> for TestEnumVariantA<A>
where
    S: RefWrapperTypes<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: AsBytes,
    A: UnsizedGenerics,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        let mut bytes = wrapper.sup().sup().as_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S, A> RefBytesMut<S> for TestEnumVariantA<A>
where
    S: RefWrapperMutExt<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: AsMutBytes,
    A: UnsizedGenerics,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        let mut bytes = unsafe { wrapper.sup_mut().sup_mut() }.as_mut_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S, A: UnsizedGenerics> RefResize<S, <AInner as UnsizedType>::RefMeta>
    for TestEnumVariantA<A>
where
    S: RefWrapperMutExt<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <AInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        let meta = TestEnumMeta::A(new_meta);
        *unsafe { wrapper.sup_mut().r_mut() } = meta;
        unsafe {
            wrapper
                .sup_mut()
                .sup_mut()
                .resize(size_of::<TestEnumDiscriminant>() + new_byte_len, meta)
        }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <AInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            let meta = TestEnumMeta::A(new_meta);
            *wrapper.sup_mut().r_mut() = meta;
            wrapper.sup_mut().sup_mut().set_meta(meta)
        }
    }
}
unsafe impl<S, A: UnsizedGenerics> RefBytes<S> for TestEnumVariantB<A>
where
    S: RefWrapperTypes<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        let mut bytes = wrapper.sup().sup().as_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S, A: UnsizedGenerics> RefBytesMut<S> for TestEnumVariantB<A>
where
    S: RefWrapperMutExt<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        let mut bytes = unsafe { wrapper.sup_mut().sup_mut() }.as_mut_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S, A: UnsizedGenerics> RefResize<S, <BInner<A> as UnsizedType>::RefMeta>
    for TestEnumVariantB<A>
where
    S: RefWrapperMutExt<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <BInner<A> as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        let meta = TestEnumMeta::B(new_meta);
        *unsafe { wrapper.sup_mut().r_mut() } = meta;
        unsafe {
            wrapper
                .sup_mut()
                .sup_mut()
                .resize(size_of::<TestEnumDiscriminant>() + new_byte_len, meta)
        }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <BInner<A> as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            let meta = TestEnumMeta::B(new_meta);
            *wrapper.sup_mut().r_mut() = meta;
            wrapper.sup_mut().sup_mut().set_meta(meta)
        }
    }
}
unsafe impl<S, A: UnsizedGenerics> RefBytes<S> for TestEnumVariantC<A>
where
    S: RefWrapperTypes<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        let mut bytes = wrapper.sup().sup().as_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S, A: UnsizedGenerics> RefBytesMut<S> for TestEnumVariantC<A>
where
    S: RefWrapperMutExt<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        let mut bytes = unsafe { wrapper.sup_mut().sup_mut() }.as_mut_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S, A: UnsizedGenerics> RefResize<S, <CInner as UnsizedType>::RefMeta>
    for TestEnumVariantC<A>
where
    S: RefWrapperMutExt<Ref = <TestEnum<A> as UnsizedType>::RefData>,
    S::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <CInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        let meta = TestEnumMeta::C(new_meta);
        *unsafe { wrapper.sup_mut().r_mut() } = meta;
        unsafe {
            wrapper
                .sup_mut()
                .sup_mut()
                .resize(size_of::<TestEnumDiscriminant>() + new_byte_len, meta)
        }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <CInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            let meta = TestEnumMeta::C(new_meta);
            *wrapper.sup_mut().r_mut() = meta;
            wrapper.sup_mut().sup_mut().set_meta(meta)
        }
    }
}

pub type ARef<S, A> =
    RefWrapper<RefWrapper<S, TestEnumVariantA<A>>, <AInner as UnsizedType>::RefData>;
pub type BRef<S, A> =
    RefWrapper<RefWrapper<S, TestEnumVariantB<A>>, <BInner<A> as UnsizedType>::RefData>;
pub type CRef<S, A> =
    RefWrapper<RefWrapper<S, TestEnumVariantC<A>>, <CInner as UnsizedType>::RefData>;
#[derive(Copy, Clone)]
pub enum TestEnumRefWrapper<S, A>
where
    A: UnsizedGenerics,
{
    A(ARef<S, A>),
    B(BRef<S, A>),
    C(CRef<S, A>),
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

    fn set_a<I>(self, a_init: I) -> anyhow::Result<ARef<Self, A>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        AInner: UnsizedInit<I>;
    fn set_b<I>(self, b_init: I) -> anyhow::Result<BRef<Self, A>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        BInner<A>: UnsizedInit<I>;
    fn set_c<I>(self, c_init: I) -> anyhow::Result<CRef<Self, A>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        CInner: UnsizedInit<I>;
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
                <AInner as UnsizedType>::from_bytes_and_meta(
                    RefWrapper::new(self, TestEnumVariantA::new()),
                    m,
                )?
                .ref_wrapper
            })),
            TestEnumMeta::B(m) => Ok(TestEnumRefWrapper::B(unsafe {
                <BInner<A> as UnsizedType>::from_bytes_and_meta(
                    RefWrapper::new(self, TestEnumVariantB::new()),
                    m,
                )?
                .ref_wrapper
            })),
            TestEnumMeta::C(m) => Ok(TestEnumRefWrapper::C(unsafe {
                <CInner as UnsizedType>::from_bytes_and_meta(
                    RefWrapper::new(self, TestEnumVariantC::new()),
                    m,
                )?
                .ref_wrapper
            })),
        }
    }

    fn set_a<I>(mut self, a_init: I) -> anyhow::Result<ARef<Self, A>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        AInner: UnsizedInit<I>,
    {
        unsafe {
            let current_meta = *self.r();
            let sup = self.sup_mut();
            sup.resize(
                size_of::<TestEnumDiscriminant>() + <AInner as UnsizedInit<I>>::INIT_BYTES,
                current_meta,
            )?;
            sup.as_mut_bytes()?[..size_of::<TestEnumDiscriminant>()]
                .copy_from_slice(bytes_of(&TestEnumDiscriminant::A));
        }
        let (mut r, m) = unsafe {
            <AInner as UnsizedInit<I>>::init(
                RefWrapper::new(self, TestEnumVariantA::new()),
                a_init,
            )?
        };
        unsafe {
            r.sup_mut()
                .sup_mut()
                .sup_mut()
                .set_meta(TestEnumMeta::A(m))?;
            *r.sup_mut().sup_mut().r_mut() = TestEnumMeta::A(m);
        }
        Ok(r)
    }

    fn set_b<I>(mut self, b_init: I) -> anyhow::Result<BRef<Self, A>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        BInner<A>: UnsizedInit<I>,
    {
        unsafe {
            let current_meta = *self.r();
            let sup = self.sup_mut();
            sup.resize(
                size_of::<TestEnumDiscriminant>() + <BInner<A> as UnsizedInit<I>>::INIT_BYTES,
                current_meta,
            )?;
            sup.as_mut_bytes()?[..size_of::<TestEnumDiscriminant>()]
                .copy_from_slice(bytes_of(&TestEnumDiscriminant::B));
        }
        let (mut r, m) = unsafe {
            <BInner<A> as UnsizedInit<I>>::init(
                RefWrapper::new(self, TestEnumVariantB::new()),
                b_init,
            )?
        };
        unsafe {
            r.sup_mut()
                .sup_mut()
                .sup_mut()
                .set_meta(TestEnumMeta::B(m))?;
            *r.sup_mut().sup_mut().r_mut() = TestEnumMeta::B(m);
        }
        Ok(r)
    }

    fn set_c<I>(mut self, c_init: I) -> anyhow::Result<CRef<Self, A>>
    where
        Self: RefWrapperMutExt,
        Self::Super: Resize<<TestEnum<A> as UnsizedType>::RefMeta>,
        CInner: UnsizedInit<I>,
    {
        unsafe {
            let current_meta = *self.r();
            let sup = self.sup_mut();
            sup.resize(
                size_of::<TestEnumDiscriminant>() + <CInner as UnsizedInit<I>>::INIT_BYTES,
                current_meta,
            )?;
            sup.as_mut_bytes()?[..size_of::<TestEnumDiscriminant>()]
                .copy_from_slice(bytes_of(&TestEnumDiscriminant::C));
        }
        let (mut r, m) = unsafe {
            <CInner as UnsizedInit<I>>::init(
                RefWrapper::new(self, TestEnumVariantC::new()),
                c_init,
            )?
        };
        unsafe {
            r.sup_mut()
                .sup_mut()
                .sup_mut()
                .set_meta(TestEnumMeta::C(m))?;
            *r.sup_mut().sup_mut().r_mut() = TestEnumMeta::C(m);
        }
        Ok(r)
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
