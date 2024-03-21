use crate::prelude::*;
use crate::serialize::ref_wrapper::{
    AsBytes, AsMutBytes, RefBytesMut, RefWrapper, RefWrapperTypes,
};
use crate::serialize::unsize::resize::Resize;
use crate::serialize::unsize::unsized_type::FromBytesReturn;
use advance::Advance;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use solana_program::program_memory::sol_memmove;
use star_frame::serialize::ref_wrapper::{RefBytes, RefWrapperMutExt};
use std::marker::PhantomData;
use std::ptr::addr_of_mut;

#[derive(Debug, Align1)]
pub struct CombinedUnsized<T: ?Sized, U: ?Sized> {
    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
    _data: [u8],
}

unsafe impl<T, U> UnsizedType for CombinedUnsized<T, U>
where
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    type RefMeta = CombinedUnsizedRefMeta<T::RefMeta, U::RefMeta>;
    type RefData = CombinedRef<T, U>;

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
            t_len,
            u_len,
        };
        Ok(FromBytesReturn {
            bytes_used: t_len + u_len,
            meta,
            ref_wrapper: RefWrapper::new(bytes, CombinedRef(meta)),
        })
    }
}

#[derive(Derivative, Deref, DerefMut)]
#[derivative(
    Debug(bound = "T::RefMeta: Debug, U::RefMeta: Debug"),
    Clone(bound = ""),
    Copy(bound = "")
)]
pub struct CombinedRef<T, U>(CombinedUnsizedRefMeta<T::RefMeta, U::RefMeta>)
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

#[derive(Debug, Copy, Clone)]
pub struct CombinedUnsizedRefMeta<TM, UM> {
    t_meta: TM,
    u_meta: UM,
    t_len: usize,
    u_len: usize,
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
        let t_len = r.t_len;
        bytes.try_advance(t_len).map_err(Into::into)
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
        let t_len = r.t_len;
        bytes.try_advance(t_len).map_err(Into::into)
    }
}
impl<S, T, U> Resize<T::RefMeta> for RefWrapper<S, CombinedTRef<T, U>>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: Resize<<CombinedUnsized<T, U> as UnsizedType>::RefMeta>,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    unsafe fn resize(&mut self, new_byte_len: usize, new_meta: T::RefMeta) -> Result<()> {
        let (sup, r) = unsafe { self.sup_mut().s_r_mut() };
        let old_t_len = r.t_len;
        r.t_meta = new_meta;
        r.t_len = new_byte_len;
        let bytes = sup.as_mut_bytes()?;
        let start_ptr = addr_of_mut!(bytes[old_t_len]);
        let end_ptr = addr_of_mut!(bytes[new_byte_len]);
        let byte_len = r.u_len;
        if new_byte_len > old_t_len {
            unsafe { sup.resize(r.t_len + r.u_len, r.0)? }
            unsafe { sol_memmove(end_ptr, start_ptr, byte_len) }
        } else {
            unsafe { sol_memmove(start_ptr, end_ptr, byte_len) }
            unsafe { sup.resize(r.t_len + r.u_len, r.0)? }
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
        let t_len = r.t_len;
        bytes.try_advance(t_len)?;
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
        let t_len = r.t_len;
        bytes.try_advance(t_len)?;
        Ok(bytes)
    }
}
impl<S, T, U> Resize<U::RefMeta> for RefWrapper<S, CombinedURef<T, U>>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: Resize<<CombinedUnsized<T, U> as UnsizedType>::RefMeta>,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    unsafe fn resize(&mut self, new_byte_len: usize, new_meta: U::RefMeta) -> Result<()> {
        let (sup, r) = unsafe { self.sup_mut().s_r_mut() };
        r.u_meta = new_meta;
        r.u_len = new_byte_len;
        unsafe { sup.resize(r.t_len + r.u_len, r.0)? }
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
        T::from_bytes(RefWrapper::new(self, CombinedTRef::default())).map(|r| r.ref_wrapper)
    }

    fn u(
        self,
    ) -> Result<
        RefWrapper<
            RefWrapper<Self, CombinedURef<Self::T, Self::U>>,
            <Self::U as UnsizedType>::RefData,
        >,
    > {
        U::from_bytes(RefWrapper::new(self, CombinedURef::default())).map(|r| r.ref_wrapper)
    }
}
