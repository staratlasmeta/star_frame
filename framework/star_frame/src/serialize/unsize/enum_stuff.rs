use crate::prelude::{List, UnsizedInit, UnsizedType};
use crate::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefBytes, RefBytesMut, RefWrapper, RefWrapperMutExt, RefWrapperTypes,
};
use crate::serialize::unsize::test::CombinedTest;
use crate::serialize::unsize::unsized_enum::UnsizedEnum;
use crate::serialize::unsize::FromBytesReturn;
use crate::util::OffsetRef;
use advance::Advance;
use bytemuck::checked::try_from_bytes;
use bytemuck::{bytes_of, CheckedBitPattern, NoUninit};
use star_frame::serialize::ref_wrapper::RefResize;
use star_frame::serialize::unsize::resize::Resize;
use std::mem::size_of;
use typenum::True;

// #[repr(u8)]
// pub enum TestEnum {
//     A,
//     B(List<u8>) = 4,
//     C(CombinedTest),
// }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, CheckedBitPattern, NoUninit)]
#[repr(u8)]
pub enum TestEnumDiscriminant {
    A,
    B = 4,
    C,
}
pub type AInner = ();
pub type BInner = List<u8>;
pub type CInner = CombinedTest;

#[derive(Copy, Clone)]
pub enum TestEnumMeta {
    A(<AInner as UnsizedType>::RefMeta),
    B(<BInner as UnsizedType>::RefMeta),
    C(<CInner as UnsizedType>::RefMeta),
}

pub enum TestEnumOwned {
    A(<AInner as UnsizedType>::Owned),
    B(<BInner as UnsizedType>::Owned),
    C(<CInner as UnsizedType>::Owned),
}

pub struct TestEnum([u8]);
unsafe impl UnsizedType for TestEnum {
    type RefMeta = TestEnumMeta;
    type RefData = TestEnumMeta;
    type Owned = TestEnumOwned;
    type IsUnsized = True;

    unsafe fn from_bytes<S: AsBytes>(
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
                    <BInner as UnsizedType>::from_bytes(RefWrapper::new(
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
                    unsafe { <BInner as UnsizedType>::from_bytes_and_meta(&super_ref, m)? };
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
impl UnsizedEnum for TestEnum {
    type Discriminant = TestEnumDiscriminant;

    fn discriminant<S: AsBytes>(r: &RefWrapper<S, Self::RefData>) -> Self::Discriminant {
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
impl<I> UnsizedInit<TestEnumInitB<I>> for TestEnum
where
    BInner: UnsizedInit<I>,
{
    const INIT_BYTES: usize =
        size_of::<TestEnumDiscriminant>() + <BInner as UnsizedInit<I>>::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        arg: TestEnumInitB<I>,
    ) -> anyhow::Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        super_ref.as_mut_bytes()?[..size_of::<TestEnumDiscriminant>()]
            .copy_from_slice(bytes_of(&TestEnumDiscriminant::B));
        let (_, ref_meta) = unsafe {
            <BInner as UnsizedInit<I>>::init(
                RefWrapper::new(&mut super_ref, OffsetRef(size_of::<TestEnumDiscriminant>())),
                arg.0,
            )?
        };
        let meta = TestEnumMeta::B(ref_meta);
        Ok((unsafe { RefWrapper::new(super_ref, meta) }, meta))
    }
}
impl<I> UnsizedInit<TestEnumInitC<I>> for TestEnum
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
pub struct TestEnumVariantA;
#[derive(Copy, Clone, Debug)]
pub struct TestEnumVariantB;
#[derive(Copy, Clone, Debug)]
pub struct TestEnumVariantC;

unsafe impl<S> RefBytes<S> for TestEnumVariantA
where
    S: RefWrapperTypes<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        let mut bytes = wrapper.sup().sup().as_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S> RefBytesMut<S> for TestEnumVariantA
where
    S: RefWrapperMutExt<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        let mut bytes = unsafe { wrapper.sup_mut().sup_mut() }.as_mut_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S> RefResize<S, <AInner as UnsizedType>::RefMeta> for TestEnumVariantA
where
    S: RefWrapperMutExt<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: Resize<<TestEnum as UnsizedType>::RefMeta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <AInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        let meta = TestEnumMeta::A(new_meta);
        *unsafe { wrapper.sup_mut().r_mut() } = meta;
        unsafe { wrapper.sup_mut().sup_mut().resize(new_byte_len, meta) }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <AInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            wrapper
                .sup_mut()
                .sup_mut()
                .set_meta(TestEnumMeta::A(new_meta))
        }
    }
}
unsafe impl<S> RefBytes<S> for TestEnumVariantB
where
    S: RefWrapperTypes<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        let mut bytes = wrapper.sup().sup().as_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S> RefBytesMut<S> for TestEnumVariantB
where
    S: RefWrapperMutExt<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        let mut bytes = unsafe { wrapper.sup_mut().sup_mut() }.as_mut_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S> RefResize<S, <BInner as UnsizedType>::RefMeta> for TestEnumVariantB
where
    S: RefWrapperMutExt<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: Resize<<TestEnum as UnsizedType>::RefMeta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <BInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        let meta = TestEnumMeta::B(new_meta);
        *unsafe { wrapper.sup_mut().r_mut() } = meta;
        unsafe { wrapper.sup_mut().sup_mut().resize(new_byte_len, meta) }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <BInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            wrapper
                .sup_mut()
                .sup_mut()
                .set_meta(TestEnumMeta::B(new_meta))
        }
    }
}
unsafe impl<S> RefBytes<S> for TestEnumVariantC
where
    S: RefWrapperTypes<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        let mut bytes = wrapper.sup().sup().as_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S> RefBytesMut<S> for TestEnumVariantC
where
    S: RefWrapperMutExt<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        let mut bytes = unsafe { wrapper.sup_mut().sup_mut() }.as_mut_bytes()?;
        bytes.advance(size_of::<TestEnumDiscriminant>());
        Ok(bytes)
    }
}
unsafe impl<S> RefResize<S, <CInner as UnsizedType>::RefMeta> for TestEnumVariantB
where
    S: RefWrapperMutExt<Ref = <TestEnum as UnsizedType>::RefData>,
    S::Super: Resize<<TestEnum as UnsizedType>::RefMeta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <CInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        let meta = TestEnumMeta::C(new_meta);
        *unsafe { wrapper.sup_mut().r_mut() } = meta;
        unsafe { wrapper.sup_mut().sup_mut().resize(new_byte_len, meta) }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <CInner as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            wrapper
                .sup_mut()
                .sup_mut()
                .set_meta(TestEnumMeta::C(new_meta))
        }
    }
}

pub type ARef<S> = RefWrapper<RefWrapper<S, TestEnumVariantA>, <AInner as UnsizedType>::RefData>;
pub type BRef<S> = RefWrapper<RefWrapper<S, TestEnumVariantB>, <BInner as UnsizedType>::RefData>;
pub type CRef<S> = RefWrapper<RefWrapper<S, TestEnumVariantC>, <CInner as UnsizedType>::RefData>;
#[derive(Copy, Clone)]
pub enum TestEnumRefWrapper<S> {
    A(ARef<S>),
    B(BRef<S>),
    C(CRef<S>),
}
pub trait TestEnumExt: Sized + RefWrapperTypes<Ref = <TestEnum as UnsizedType>::RefData> {
    fn get(self) -> anyhow::Result<TestEnumRefWrapper<Self>>;
}
pub trait TestEnumMutExt: TestEnumExt {
    fn set_a<I>(self, a_init: I) -> anyhow::Result<ARef<Self>>
    where
        AInner: UnsizedInit<I>;
    fn set_b<I>(self, b_init: I) -> anyhow::Result<BRef<Self>>
    where
        BInner: UnsizedInit<I>;
    fn set_c<I>(self, c_init: I) -> anyhow::Result<CRef<Self>>
    where
        CInner: UnsizedInit<I>;
}

impl<R> TestEnumExt for R
where
    R: RefWrapperTypes<Ref = <TestEnum as UnsizedType>::RefData>,
    R::Super: AsBytes,
{
    fn get(self) -> anyhow::Result<TestEnumRefWrapper<Self>> {
        match *self.r() {
            TestEnumMeta::A(m) => Ok(TestEnumRefWrapper::A(unsafe {
                <AInner as UnsizedType>::from_bytes_and_meta(
                    RefWrapper::new(self, TestEnumVariantA),
                    m,
                )?
                .ref_wrapper
            })),
            TestEnumMeta::B(m) => Ok(TestEnumRefWrapper::B(unsafe {
                <BInner as UnsizedType>::from_bytes_and_meta(
                    RefWrapper::new(self, TestEnumVariantB),
                    m,
                )?
                .ref_wrapper
            })),
            TestEnumMeta::C(m) => Ok(TestEnumRefWrapper::C(unsafe {
                <CInner as UnsizedType>::from_bytes_and_meta(
                    RefWrapper::new(self, TestEnumVariantC),
                    m,
                )?
                .ref_wrapper
            })),
        }
    }
}
