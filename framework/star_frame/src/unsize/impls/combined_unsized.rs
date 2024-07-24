use crate::prelude::*;
use crate::util::OffsetRef;
use advance::Advance;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use solana_program::program_memory::sol_memmove;
use star_frame::unsize::ref_wrapper::{RefBytes, RefWrapperMutExt};
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
    type Owned = (T::Owned, U::Owned);
    type IsUnsized = Or<T::IsUnsized, U::IsUnsized>;

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
        } = unsafe { U::from_bytes(RefWrapper::new(&bytes, OffsetRef(t_len)))? };

        let meta = CombinedUnsizedRefMeta {
            t_meta,
            u_meta,
            t_len: T::IsUnsized::from_len(t_len),
            u_len: U::IsUnsized::from_len(u_len),
        };
        Ok(FromBytesReturn {
            bytes_used: t_len + u_len,
            meta,
            ref_wrapper: unsafe { RefWrapper::new(bytes, CombinedRef(meta)) },
        })
    }

    unsafe fn from_bytes_and_meta<S: AsBytes>(
        super_ref: S,
        meta: Self::RefMeta,
    ) -> Result<FromBytesReturn<S, Self::RefData, Self::RefMeta>> {
        Ok(FromBytesReturn {
            bytes_used: T::IsUnsized::len(meta.t_len) + U::IsUnsized::len(meta.u_len),
            meta,
            ref_wrapper: unsafe { RefWrapper::new(super_ref, CombinedRef(meta)) },
        })
    }

    fn owned<S: AsBytes>(r: RefWrapper<S, Self::RefData>) -> Result<Self::Owned> {
        Ok((T::owned((&r).t()?)?, U::owned(r.u()?)?))
    }
}
impl<T, U, TA, UA> UnsizedInit<(TA, UA)> for CombinedUnsized<T, U>
where
    T: ?Sized + UnsizedInit<TA>,
    U: ?Sized + UnsizedInit<UA>,
    T::IsUnsized: BitOr<U::IsUnsized>,
    <T::IsUnsized as BitOr<U::IsUnsized>>::Output: Bit + LengthAccess<Self>,
{
    const INIT_BYTES: usize = T::INIT_BYTES + U::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        mut super_ref: S,
        (t_arg, u_arg): (TA, UA),
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        let mut bytes = super_ref.as_mut_bytes()?;
        let (_, t_meta) = unsafe { T::init(bytes.try_advance(T::INIT_BYTES)?, t_arg)? };
        let (_, u_meta) = unsafe { U::init(bytes.try_advance(U::INIT_BYTES)?, u_arg)? };
        let meta = CombinedUnsizedRefMeta {
            t_meta,
            u_meta,
            t_len: T::IsUnsized::from_len(T::INIT_BYTES),
            u_len: U::IsUnsized::from_len(U::INIT_BYTES),
        };
        Ok((
            unsafe { RefWrapper::new(super_ref, CombinedRef(meta)) },
            meta,
        ))
    }
}
impl<T, U> UnsizedInit<Zeroed> for CombinedUnsized<T, U>
where
    T: ?Sized + UnsizedInit<Zeroed>,
    U: ?Sized + UnsizedInit<Zeroed>,
    T::IsUnsized: BitOr<U::IsUnsized>,
    <T::IsUnsized as BitOr<U::IsUnsized>>::Output: Bit + LengthAccess<Self>,
{
    const INIT_BYTES: usize = T::INIT_BYTES + U::INIT_BYTES;

    unsafe fn init<S: AsMutBytes>(
        super_ref: S,
        _arg: Zeroed,
    ) -> Result<(RefWrapper<S, Self::RefData>, Self::RefMeta)> {
        unsafe { Self::init(super_ref, (Zeroed, Zeroed)) }
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
impl<T, U> CombinedRef<T, U>
where
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
{
    /// Creates a new [`CombinedRef`] from a [`CombinedUnsizedRefMeta`].
    ///
    /// # Safety
    /// This should only be called where the meta results in a valid ref and usage.
    pub unsafe fn new(meta: CombinedUnsizedRefMeta<T, U>) -> Self {
        Self(meta)
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
    pub(crate) t_meta: T::RefMeta,
    pub(crate) u_meta: U::RefMeta,
    pub(crate) t_len: <T::IsUnsized as LengthAccess<T>>::LengthData,
    pub(crate) u_len: <U::IsUnsized as LengthAccess<U>>::LengthData,
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
unsafe impl<S, T, U> RefResize<S, T::RefMeta> for CombinedTRef<T, U>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: Resize<<CombinedUnsized<T, U> as UnsizedType>::RefMeta>,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
    T::IsUnsized: BitOr<U::IsUnsized>,
    <T::IsUnsized as BitOr<U::IsUnsized>>::Output: Bit + LengthAccess<CombinedUnsized<T, U>>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: T::RefMeta,
    ) -> Result<()> {
        let (sup, r) = unsafe { wrapper.sup_mut().s_r_mut() };
        let old_t_len = T::IsUnsized::len(r.t_len);
        r.t_meta = new_meta;
        r.t_len = T::IsUnsized::from_len(new_byte_len);
        let byte_len = U::IsUnsized::len(r.u_len);
        if new_byte_len > old_t_len {
            unsafe { sup.resize(new_byte_len + byte_len, r.0)? }
            let bytes = sup.as_mut_bytes()?;
            let start_ptr = addr_of_mut!(bytes[old_t_len]);
            let end_ptr = addr_of_mut!(bytes[new_byte_len]);
            unsafe { sol_memmove(end_ptr, start_ptr, byte_len) }
        } else {
            let bytes = sup.as_mut_bytes()?;
            let start_ptr = addr_of_mut!(bytes[old_t_len]);
            let end_ptr = addr_of_mut!(bytes[new_byte_len]);
            unsafe { sol_memmove(start_ptr, end_ptr, byte_len) }
            unsafe { sup.resize(new_byte_len + byte_len, r.0)? }
        }

        Ok(())
    }

    unsafe fn set_meta(wrapper: &mut RefWrapper<S, Self>, new_meta: T::RefMeta) -> Result<()> {
        unsafe { wrapper.sup_mut().r_mut().t_meta = new_meta };
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
unsafe impl<S, T, U> RefResize<S, U::RefMeta> for CombinedURef<T, U>
where
    S: RefWrapperMutExt<Ref = CombinedRef<T, U>>,
    S::Super: Resize<<CombinedUnsized<T, U> as UnsizedType>::RefMeta>,
    T: ?Sized + UnsizedType,
    U: ?Sized + UnsizedType,
    T::IsUnsized: BitOr<U::IsUnsized>,
    <T::IsUnsized as BitOr<U::IsUnsized>>::Output: Bit + LengthAccess<CombinedUnsized<T, U>>,
{
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: U::RefMeta,
    ) -> Result<()> {
        let (sup, r) = unsafe { wrapper.sup_mut().s_r_mut() };
        r.u_meta = new_meta;
        r.u_len = U::IsUnsized::from_len(new_byte_len);
        unsafe { sup.resize(T::IsUnsized::len(r.t_len) + new_byte_len, r.0)? }
        Ok(())
    }

    unsafe fn set_meta(wrapper: &mut RefWrapper<S, Self>, new_meta: U::RefMeta) -> Result<()> {
        unsafe { wrapper.sup_mut().r_mut().u_meta = new_meta };
        Ok(())
    }
}

pub type RefWrapperT<S, T, U> =
    RefWrapper<RefWrapper<S, CombinedTRef<T, U>>, <T as UnsizedType>::RefData>;
pub type RefWrapperU<S, T, U> =
    RefWrapper<RefWrapper<S, CombinedURef<T, U>>, <U as UnsizedType>::RefData>;

#[allow(clippy::type_complexity)]
pub trait CombinedExt: Sized {
    type T: ?Sized + UnsizedType;
    type U: ?Sized + UnsizedType;

    fn t(self) -> Result<RefWrapperT<Self, Self::T, Self::U>>;
    fn u(self) -> Result<RefWrapperU<Self, Self::T, Self::U>>;
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

    fn t(self) -> Result<RefWrapperT<Self, Self::T, Self::U>> {
        let t_meta = self.r().t_meta;
        unsafe { T::from_bytes_and_meta(RefWrapper::new(self, CombinedTRef::default()), t_meta) }
            .map(|r| r.ref_wrapper)
    }

    fn u(self) -> Result<RefWrapperU<Self, Self::T, Self::U>> {
        let u_meta = self.r().u_meta;
        unsafe { U::from_bytes_and_meta(RefWrapper::new(self, CombinedURef::default()), u_meta) }
            .map(|r| r.ref_wrapper)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unsize::{List, TestByteSet};

    #[test]
    fn test_all_sized() -> Result<()> {
        type Thingy = CombinedUnsized<CombinedUnsized<u8, u8>, u8>;
        let bytes = TestByteSet::<Thingy>::new(Zeroed)?;
        let r = bytes.immut()?;
        assert_eq!(*r.t()?.t()?, 0);
        assert_eq!(*r.t()?.u()?, 0);
        assert_eq!(*r.u()?, 0);
        Ok(())
    }

    #[test]
    fn test_all_unsized() -> Result<()> {
        type Thingy = CombinedUnsized<CombinedUnsized<List<u8>, List<u8>>, List<u8>>;
        let bytes = TestByteSet::<Thingy>::new(Zeroed)?;
        let r = bytes.immut()?;
        assert_eq!(r.t()?.t()?.len(), 0);
        assert_eq!(r.t()?.u()?.len(), 0);
        assert_eq!(r.u()?.len(), 0);
        Ok(())
    }

    #[test]
    fn test_combined_size() -> Result<()> {
        type Thingy1 = CombinedUnsized<CombinedUnsized<List<u8>, u8>, List<u8>>;
        type Thingy2 = CombinedUnsized<CombinedUnsized<List<u8>, u8>, u8>;

        let bytes = TestByteSet::<Thingy1>::new(Zeroed)?;
        let r = bytes.immut()?;
        assert_eq!(r.t()?.t()?.len(), 0);
        assert_eq!(*r.t()?.u()?, 0);
        assert_eq!(r.u()?.len(), 0);

        let bytes = TestByteSet::<Thingy2>::new(Zeroed)?;
        let r = bytes.immut()?;
        assert_eq!(r.t()?.t()?.len(), 0);
        assert_eq!(*r.t()?.u()?, 0);
        assert_eq!(*r.u()?, 0);
        Ok(())
    }
}
