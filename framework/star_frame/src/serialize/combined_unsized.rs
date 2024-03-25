use crate::prelude::*;
use crate::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefBytesMut, RefWrapper, RefWrapperTypes,
};
use crate::serialize::unsize::resize::Resize;
use crate::serialize::unsize::{FromBytesReturn, LengthAccess};
use advance::Advance;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use solana_program::program_memory::sol_memmove;
use star_frame::serialize::ref_wrapper::{RefBytes, RefWrapperMutExt};
use std::marker::PhantomData;
use std::ops::BitOr;
use std::ptr::addr_of_mut;
use typenum::{Bit, Or};

#[derive(Debug, Align1)]
pub struct CombinedUnsized<T: ?Sized, U: ?Sized> {
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
}

unsafe impl<T, U> UnsizedType for CombinedUnsized<T, U>
where
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
    T::IsUnsized: BitOr<U::IsUnsized>,
    <T::IsUnsized as BitOr<U::IsUnsized>>::Output: Bit + LengthAccess<Self>,
{
    type RefMeta = CombinedUnsizedRefMeta<T, U>;
    type RefData = CombinedRef<T, U>;
    type IsUnsized = Or<T::IsUnsized, U::IsUnsized>;
    type Owned = (T::Owned, U::Owned);

    fn from_bytes<S: AsBytes>(
        bytes: S,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        let FromBytesReturn {
            bytes_used: t_len,
            meta: t_meta,
            ..
        } = T::from_bytes(&bytes)?;
        let FromBytesReturn {
            bytes_used: u_len,
            meta: u_meta,
            ..
        } = U::from_bytes(RefWrapper::new(&bytes, OffsetRef(t_len)))?;

        let meta = CombinedUnsizedRefMeta {
            t_meta,
            u_meta,
            t_len: T::IsUnsized::from_len(t_len),
            u_len: U::IsUnsized::from_len(u_len),
        };
        Ok(FromBytesReturn {
            bytes_used: t_len + u_len,
            meta,
            ref_wrapper: RefWrapper::new(bytes, CombinedRef(meta)),
        })
    }

    unsafe fn from_bytes_and_meta<S: AsBytes>(
        super_ref: S,
        meta: Self::RefMeta,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        Ok(FromBytesReturn {
            bytes_used: T::IsUnsized::len(meta.t_len) + U::IsUnsized::len(meta.u_len),
            meta,
            ref_wrapper: RefWrapper::new(super_ref, CombinedRef(meta)),
        })
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned> {
        Ok((T::owned((&r).t()?)?, U::owned(r.u()?)?))
    }
}

#[derive(Derivative, Deref, DerefMut)]
#[derivative(
    Debug(bound = "T::RefMeta: Debug, U::RefMeta: Debug"),
    Clone(bound = ""),
    Copy(bound = "")
)]
pub struct CombinedRef<T, U>(CombinedUnsizedRefMeta<T, U>)
where
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType;

struct OffsetRef(usize);
unsafe impl<S> RefBytes<S> for OffsetRef
where
    S: AsBytes,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
        let mut bytes = wrapper.sup().as_bytes()?;
        bytes.try_advance(wrapper.r().0)?;
        Ok(bytes)
    }
}

#[derive(Derivative)]
#[derivative(
    Debug(bound = "T::RefMeta: Debug, U::RefMeta: Debug"),
    Clone(bound = ""),
    Copy(bound = "")
)]
pub struct CombinedUnsizedRefMeta<T, U>
where
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    t_meta: T::RefMeta,
    u_meta: U::RefMeta,
    t_len: <T::IsUnsized as LengthAccess<T>>::LengthData,
    u_len: <U::IsUnsized as LengthAccess<U>>::LengthData,
}

#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Default(bound = "")
)]
pub struct CombinedTRef<T, U>(PhantomData<T>, PhantomData<U>)
where
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType;
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Default(bound = "")
)]
pub struct CombinedURef<T, U>(PhantomData<T>, PhantomData<U>)
where
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType;

unsafe impl<S, T, U> RefBytes<S> for CombinedTRef<T, U>
where
    S: RefWrapperTypes<Ref = CombinedRef<T, U>>,
    S::Super: AsBytes,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
        let (sup, r) = wrapper.sup().s_r();
        let mut bytes = sup.as_bytes()?;
        bytes
            .try_advance(T::IsUnsized::len(r.t_len))
            .map_err(Into::into)
    }
}
unsafe impl<S, T, U> RefBytesMut<S> for CombinedTRef<T, U>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: AsMutBytes,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]> {
        let (sup, r) = unsafe { wrapper.sup_mut().s_r_mut() };
        let mut bytes = sup.as_mut_bytes()?;
        bytes
            .try_advance(T::IsUnsized::len(r.t_len))
            .map_err(Into::into)
    }
}
unsafe impl<S, T, U> Resize<T::RefMeta> for RefWrapper<S, CombinedTRef<T, U>>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: Resize<<CombinedUnsized<T, U> as UnsizedType>::RefMeta>,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
    T::IsUnsized: BitOr<U::IsUnsized>,
    <T::IsUnsized as BitOr<U::IsUnsized>>::Output: Bit + LengthAccess<CombinedUnsized<T, U>>,
{
    unsafe fn resize(&mut self, new_byte_len: usize, new_meta: T::RefMeta) -> Result<()> {
        let (sup, r) = unsafe { self.sup_mut().s_r_mut() };
        let old_t_len = T::IsUnsized::len(r.t_len);
        r.t_meta = new_meta;
        r.t_len = T::IsUnsized::from_len(new_byte_len);
        let bytes = sup.as_mut_bytes()?;
        let start_ptr = addr_of_mut!(bytes[old_t_len]);
        let end_ptr = addr_of_mut!(bytes[new_byte_len]);
        let byte_len = U::IsUnsized::len(r.u_len);
        if new_byte_len > old_t_len {
            unsafe { sup.resize(new_byte_len + byte_len, r.0)? }
            unsafe { sol_memmove(end_ptr, start_ptr, byte_len) }
        } else {
            unsafe { sol_memmove(start_ptr, end_ptr, byte_len) }
            unsafe { sup.resize(new_byte_len + byte_len, r.0)? }
        }

        Ok(())
    }
}

unsafe impl<S, T, U> RefBytes<S> for CombinedURef<T, U>
where
    S: RefWrapperTypes<Ref = CombinedRef<T, U>>,
    S::Super: AsBytes,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]> {
        let (sup, r) = wrapper.sup().s_r();
        let mut bytes = sup.as_bytes()?;
        bytes.try_advance(T::IsUnsized::len(r.t_len))?;
        Ok(bytes)
    }
}
unsafe impl<S, T, U> RefBytesMut<S> for CombinedURef<T, U>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: AsMutBytes,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]> {
        let (sup, r) = unsafe { wrapper.sup_mut().s_r_mut() };
        let mut bytes = sup.as_mut_bytes()?;
        bytes.try_advance(T::IsUnsized::len(r.t_len))?;
        Ok(bytes)
    }
}
unsafe impl<S, T, U> Resize<U::RefMeta> for RefWrapper<S, CombinedURef<T, U>>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: Resize<<CombinedUnsized<T, U> as UnsizedType>::RefMeta>,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
    T::IsUnsized: BitOr<U::IsUnsized>,
    <T::IsUnsized as BitOr<U::IsUnsized>>::Output: Bit + LengthAccess<CombinedUnsized<T, U>>,
{
    unsafe fn resize(&mut self, new_byte_len: usize, new_meta: U::RefMeta) -> Result<()> {
        let (sup, r) = unsafe { self.sup_mut().s_r_mut() };
        r.u_meta = new_meta;
        r.u_len = U::IsUnsized::from_len(new_byte_len);
        unsafe { sup.resize(T::IsUnsized::len(r.t_len) + new_byte_len, r.0)? }
        Ok(())
    }
}

#[allow(clippy::type_complexity)]
pub trait CombinedExt: Sized {
    type T: ?Sized + UnsizedType;
    type U: ?Sized + UnsizedType;

    fn t(
        self,
    ) -> Result<
        RefWrapper<
            RefWrapper<Self, CombinedTRef<Self::T, Self::U>>,
            <Self::T as UnsizedType>::RefData,
        >,
    >;
    fn u(
        self,
    ) -> Result<
        RefWrapper<
            RefWrapper<Self, CombinedURef<Self::T, Self::U>>,
            <Self::U as UnsizedType>::RefData,
        >,
    >;
}
impl<S, T, U> CombinedExt for S
where
    S: RefWrapperTypes<Ref = CombinedRef<T, U>>,
    S::Super: AsBytes,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    type T = T;
    type U = U;

    fn t(
        self,
    ) -> Result<
        RefWrapper<
            RefWrapper<Self, CombinedTRef<Self::T, Self::U>>,
            <Self::T as UnsizedType>::RefData,
        >,
    > {
        let t_meta = self.r().t_meta;
        unsafe { T::from_bytes_and_meta(RefWrapper::new(self, CombinedTRef::default()), t_meta) }
            .map(|r| r.ref_wrapper)
    }

    fn u(
        self,
    ) -> Result<
        RefWrapper<
            RefWrapper<Self, CombinedURef<Self::T, Self::U>>,
            <Self::U as UnsizedType>::RefData,
        >,
    > {
        let u_meta = self.r().u_meta;
        unsafe { U::from_bytes_and_meta(RefWrapper::new(self, CombinedURef::default()), u_meta) }
            .map(|r| r.ref_wrapper)
    }
}
