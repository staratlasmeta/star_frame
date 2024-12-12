#![allow(clippy::missing_safety_doc)] // todo: turn this off
use crate::prelude::{
    AsMutBytes, FromBytesReturn, RefBytes, RefWrapper, UnsizedInitReturn, UnsizedType,
};
use crate::unsize::ref_wrapper::{AsBytes, RefWrapperTypes};
use crate::unsize::{RefBytesMut, RefResize, RefWrapperMutExt, Resize, UnsizedInit};
use advance::Advance;
use bytemuck::{bytes_of, CheckedBitPattern, NoUninit};
use core::mem::size_of;
use star_frame::util::OffsetRef;

pub trait UnsizedEnum: UnsizedType {
    type Discriminant: CheckedBitPattern + NoUninit;

    fn discriminant<S: AsBytes>(
        r: &impl RefWrapperTypes<Super = S, Ref = Self::RefData>,
    ) -> Self::Discriminant;
}

pub type UnsizedEnumVariantRef<S, R> =
    RefWrapper<RefWrapper<S, R>, <<R as UnsizedEnumVariant>::InnerType as UnsizedType>::RefData>;

type UnsizedEnumFromBytesReturn<Super, SelfType> = FromBytesReturn<
    Super,
    <<SelfType as UnsizedEnumVariant>::UnsizedEnum as UnsizedType>::RefData,
    <<SelfType as UnsizedEnumVariant>::UnsizedEnum as UnsizedType>::RefMeta,
>;

pub unsafe trait UnsizedEnumVariant: Sized {
    type UnsizedEnum: UnsizedEnum<RefData = <Self::UnsizedEnum as UnsizedType>::RefMeta> + ?Sized;
    type InnerType: UnsizedType + ?Sized;
    const DISCRIMINANT: <Self::UnsizedEnum as UnsizedEnum>::Discriminant;

    fn new() -> Self;
    fn new_meta(
        meta: <Self::InnerType as UnsizedType>::RefMeta,
    ) -> <Self::UnsizedEnum as UnsizedType>::RefMeta;

    #[inline]
    unsafe fn from_bytes<S>(super_ref: S) -> anyhow::Result<UnsizedEnumFromBytesReturn<S, Self>>
    where
        S: AsBytes,
    {
        let FromBytesReturn {
            bytes_used, meta, ..
        } = unsafe {
            <Self::InnerType as UnsizedType>::from_bytes(RefWrapper::new(
                &super_ref,
                OffsetRef(size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>()),
            ))?
        };
        let meta = Self::new_meta(meta);
        Ok(FromBytesReturn {
            bytes_used: bytes_used + size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>(),
            meta,
            ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
        })
    }

    #[inline]
    unsafe fn from_bytes_and_meta<S>(
        super_ref: S,
        meta: <Self::UnsizedEnum as UnsizedType>::RefMeta,
        inner_meta: <Self::InnerType as UnsizedType>::RefMeta,
    ) -> anyhow::Result<UnsizedEnumFromBytesReturn<S, Self>>
    where
        S: AsBytes,
    {
        let FromBytesReturn { bytes_used, .. } = unsafe {
            <Self::InnerType as UnsizedType>::from_bytes_and_meta(&super_ref, inner_meta)?
        };
        Ok(FromBytesReturn {
            bytes_used,
            meta,
            ref_wrapper: unsafe { RefWrapper::new(super_ref, meta) },
        })
    }

    #[inline]
    unsafe fn get<R>(
        r: R,
        meta: <Self::InnerType as UnsizedType>::RefMeta,
    ) -> anyhow::Result<UnsizedEnumVariantRef<R, Self>>
    where
        R: RefWrapperTypes<Ref = <Self::UnsizedEnum as UnsizedType>::RefData>,
        R::Super: AsBytes,
    {
        Ok(unsafe {
            <Self::InnerType as UnsizedType>::from_bytes_and_meta(
                RefWrapper::new(r, Self::new()),
                meta,
            )?
            .ref_wrapper
        })
    }

    #[inline]
    fn set<R, I>(mut r: R, init: I) -> anyhow::Result<UnsizedEnumVariantRef<R, Self>>
    where
        R: RefWrapperTypes<Ref = <Self::UnsizedEnum as UnsizedType>::RefData> + RefWrapperMutExt,
        R::Super: AsBytes + Resize<<Self::UnsizedEnum as UnsizedType>::RefMeta>,
        Self::InnerType: UnsizedInit<I>,
    {
        unsafe {
            let current_meta = *r.r();
            let sup = r.sup_mut();
            sup.resize(
                size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>()
                    + <Self::InnerType as UnsizedInit<I>>::INIT_BYTES,
                current_meta,
            )?;
            sup.as_mut_bytes()?[..size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>()]
                .copy_from_slice(bytes_of(&Self::DISCRIMINANT));
        }
        let (mut r, m) = unsafe {
            <Self::InnerType as UnsizedInit<I>>::init(RefWrapper::new(r, Self::new()), init)?
        };
        unsafe {
            r.sup_mut()
                .sup_mut()
                .sup_mut()
                .set_meta(Self::new_meta(m))?;
            *r.sup_mut().sup_mut().r_mut() = Self::new_meta(m);
        }
        Ok(r)
    }
}

pub unsafe trait UnsizedEnumVariantInit {}

unsafe impl<S, T> RefResize<S, <T::InnerType as UnsizedType>::RefMeta> for T
where
    T: UnsizedEnumVariant,
    S: RefWrapperMutExt<Ref = <T::UnsizedEnum as UnsizedType>::RefData>,
    S::Super: Resize<<T::UnsizedEnum as UnsizedType>::RefMeta>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: <T::InnerType as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        let meta = T::new_meta(new_meta);
        *unsafe { wrapper.sup_mut().r_mut() } = meta;
        unsafe {
            wrapper.sup_mut().sup_mut().resize(
                size_of::<<T::UnsizedEnum as UnsizedEnum>::Discriminant>() + new_byte_len,
                meta,
            )
        }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <T::InnerType as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            let meta = T::new_meta(new_meta);
            *wrapper.sup_mut().r_mut() = meta;
            wrapper.sup_mut().sup_mut().set_meta(meta)
        }
    }
}

unsafe impl<S, T> RefBytes<S> for T
where
    T: UnsizedEnumVariant,
    S: RefWrapperTypes<Ref = <T::UnsizedEnum as UnsizedType>::RefData>,
    S::Super: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> anyhow::Result<&[u8]> {
        let mut bytes = wrapper.sup().sup().as_bytes()?;
        bytes.advance(size_of::<<T::UnsizedEnum as UnsizedEnum>::Discriminant>());
        Ok(bytes)
    }
}

unsafe impl<S, T> RefBytesMut<S> for T
where
    T: UnsizedEnumVariant,
    S: RefWrapperMutExt<Ref = <T::UnsizedEnum as UnsizedType>::RefData>,
    S::Super: AsMutBytes,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> anyhow::Result<&mut [u8]> {
        let mut bytes = unsafe { wrapper.sup_mut().sup_mut() }.as_mut_bytes()?;
        bytes.advance(size_of::<<T::UnsizedEnum as UnsizedEnum>::Discriminant>());
        Ok(bytes)
    }
}
