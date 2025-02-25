use crate::prelude::{
    AsMutBytes, FromBytesReturn, RefBytes, RefWrapper, UnsizedInitReturn, UnsizedType,
};
use crate::unsize::ref_wrapper::{AsBytes, RefWrapperTypes};
use crate::unsize::{RefBytesMut, RefResize, RefWrapperMutExt, Resize, UnsizedInit};
use advance::Advance;
use bytemuck::checked::try_from_bytes;
use bytemuck::{bytes_of, CheckedBitPattern, NoUninit};
use core::mem::size_of;
use star_frame::util::OffsetRef;

pub trait UnsizedEnum: UnsizedType {
    type Discriminant: CheckedBitPattern + NoUninit;

    #[inline]
    fn discriminant_from_bytes<S: AsBytes>(super_ref: &S) -> anyhow::Result<&Self::Discriminant> {
        Ok(try_from_bytes(
            &AsBytes::as_bytes(super_ref)?[..size_of::<Self::Discriminant>()],
        )?)
    }

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

/// A helper trait for implementing [`UnsizedType`] methods on [`UnsizedEnum`]s.
///
/// # Safety
/// This trait should be implemented via the [`super::unsized_type`] proc macro.
pub unsafe trait UnsizedEnumVariant: Sized + Default {
    type UnsizedEnum: UnsizedEnum<RefData = <Self::UnsizedEnum as UnsizedType>::RefMeta> + ?Sized;
    type InnerType: UnsizedType + ?Sized;
    const DISCRIMINANT: <Self::UnsizedEnum as UnsizedEnum>::Discriminant;

    fn new_meta(
        meta: <Self::InnerType as UnsizedType>::RefMeta,
    ) -> <Self::UnsizedEnum as UnsizedType>::RefMeta;

    /// # Safety
    /// The enum discriminant must already be checked before calling this.
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

    /// # Safety
    /// Has the same requirements as [`Self::from_bytes`] and [`UnsizedType::from_bytes_and_meta`]
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

    /// # Safety
    /// The enum discriminant must be checked before calling this method
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
                RefWrapper::new(r, Self::default()),
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
            let current_meta = *RefWrapperTypes::r(&r);
            let sup = RefWrapperMutExt::sup_mut(&mut r);
            Resize::resize(
                sup,
                size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>()
                    + <Self::InnerType as UnsizedInit<I>>::INIT_BYTES,
                current_meta,
            )?;
            AsMutBytes::as_mut_bytes(sup)?
                [..size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>()]
                .copy_from_slice(bytes_of(&Self::DISCRIMINANT));
        }
        let (mut r, m) = unsafe {
            <Self::InnerType as UnsizedInit<I>>::init(RefWrapper::new(r, Self::default()), init)?
        };
        unsafe {
            let sup2 = RefWrapperMutExt::sup_mut(RefWrapperMutExt::sup_mut(&mut r));
            Resize::set_meta(RefWrapperMutExt::sup_mut(sup2), Self::new_meta(m))?;
            *RefWrapperMutExt::r_mut(sup2) = Self::new_meta(m);
        }
        Ok(r)
    }

    #[inline]
    fn init<S, InitType, I>(
        mut super_ref: S,
        arg: InitType,
        inner: impl Fn(InitType) -> I,
    ) -> anyhow::Result<UnsizedInitReturn<S, Self::UnsizedEnum>>
    where
        S: AsMutBytes,
        Self::InnerType: UnsizedInit<I>,
    {
        unsafe { AsMutBytes::as_mut_bytes(&mut super_ref) }?
            [..size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>()]
            .copy_from_slice(bytes_of(&Self::DISCRIMINANT));
        let (_, ref_meta) = unsafe {
            <Self::InnerType as UnsizedInit<I>>::init(
                RefWrapper::new(
                    &mut super_ref,
                    OffsetRef(size_of::<<Self::UnsizedEnum as UnsizedEnum>::Discriminant>()),
                ),
                inner(arg),
            )?
        };
        let meta = Self::new_meta(ref_meta);
        Ok((unsafe { RefWrapper::new(super_ref, meta) }, meta))
    }
}

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
        *unsafe { RefWrapperMutExt::r_mut(RefWrapperMutExt::sup_mut(wrapper)) } = meta;
        let new_byte_len =
            size_of::<<T::UnsizedEnum as UnsizedEnum>::Discriminant>() + new_byte_len;
        unsafe {
            let sups = RefWrapperMutExt::sup_mut(RefWrapperMutExt::sup_mut(wrapper));
            Resize::resize(sups, new_byte_len, meta)
        }
    }

    unsafe fn set_meta(
        wrapper: &mut RefWrapper<S, Self>,
        new_meta: <T::InnerType as UnsizedType>::RefMeta,
    ) -> anyhow::Result<()> {
        unsafe {
            let meta = T::new_meta(new_meta);
            let sup = RefWrapperMutExt::sup_mut(wrapper);
            *RefWrapperMutExt::r_mut(sup) = meta;
            Resize::set_meta(RefWrapperMutExt::sup_mut(sup), meta)
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
        let mut bytes = AsBytes::as_bytes(RefWrapperTypes::sup(RefWrapperTypes::sup(wrapper)))?;
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
        let sup_muts = unsafe { RefWrapperMutExt::sup_mut(RefWrapperMutExt::sup_mut(wrapper)) };
        let mut bytes = unsafe { AsMutBytes::as_mut_bytes(sup_muts) }?;
        bytes.advance(size_of::<<T::UnsizedEnum as UnsizedEnum>::Discriminant>());
        Ok(bytes)
    }
}
