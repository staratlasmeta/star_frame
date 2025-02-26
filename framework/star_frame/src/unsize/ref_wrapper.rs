use crate::unsize::Resize;
use crate::Result;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RefWrapper<S, R> {
    super_ref: S,
    r: R,
}
impl<S, R> RefWrapper<S, R> {
    /// # Safety
    /// Only safe to use if `r` is valid for `super_ref`.
    pub const unsafe fn new(super_ref: S, r: R) -> Self {
        Self { super_ref, r }
    }

    /// # Safety
    /// This method is insanely unsafe. `R2` must be transparent to `R`.
    /// There are probably other requirements too.
    pub unsafe fn cast_r<R2>(s: &Self) -> &RefWrapper<S, R2> {
        // SAFETY: `R2` is transparent to `R`.
        unsafe { &*std::ptr::from_ref::<Self>(s).cast::<RefWrapper<S, R2>>() }
    }

    /// # Safety
    /// This method is insanely unsafe. `R2` must be transparent to `R`.
    /// There are probably other requirements too.
    pub unsafe fn cast_r_mut<R2>(s: &mut Self) -> &mut RefWrapper<S, R2> {
        // SAFETY: `R2` is transparent to `R`.
        unsafe { &mut *std::ptr::from_mut::<Self>(s).cast::<RefWrapper<S, R2>>() }
    }

    /// # Safety
    /// Only safe to use if `r` is valid for `super_ref`.
    pub unsafe fn wrap_r<R2>(mut s: Self, f: impl FnOnce(&mut S, R) -> R2) -> RefWrapper<S, R2> {
        let new_r = f(&mut s.super_ref, s.r);
        unsafe { RefWrapper::new(s.super_ref, new_r) }
    }
}
pub trait RefWrapperTypes {
    type Super;
    type Ref;

    fn sup(s: &Self) -> &Self::Super;

    fn r(s: &Self) -> &Self::Ref;

    fn s_r(s: &Self) -> (&Self::Super, &Self::Ref);
}
impl<'a, T> RefWrapperTypes for &'a T
where
    T: RefWrapperTypes,
{
    type Super = T::Super;
    type Ref = T::Ref;

    fn sup(s: &Self) -> &Self::Super {
        T::sup(*s)
    }

    fn r(this: &Self) -> &Self::Ref {
        T::r(*this)
    }

    fn s_r(s: &Self) -> (&Self::Super, &Self::Ref) {
        T::s_r(*s)
    }
}
impl<'a, T> RefWrapperTypes for &'a mut T
where
    T: RefWrapperTypes,
{
    type Super = T::Super;
    type Ref = T::Ref;

    fn sup(s: &Self) -> &Self::Super {
        T::sup(*s)
    }

    fn r(s: &Self) -> &Self::Ref {
        T::r(*s)
    }

    fn s_r(s: &Self) -> (&Self::Super, &Self::Ref) {
        T::s_r(*s)
    }
}
pub trait RefWrapperMutExt: RefWrapperTypes {
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn sup_mut(s: &mut Self) -> &mut Self::Super;
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn r_mut(s: &mut Self) -> &mut Self::Ref;
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn s_r_mut(s: &mut Self) -> (&mut Self::Super, &mut Self::Ref);
}

impl<'a, T> RefWrapperMutExt for &'a mut T
where
    T: RefWrapperMutExt,
{
    unsafe fn sup_mut(s: &mut Self) -> &mut Self::Super {
        unsafe { T::sup_mut(*s) }
    }

    unsafe fn r_mut(s: &mut Self) -> &mut Self::Ref {
        unsafe { T::r_mut(*s) }
    }

    unsafe fn s_r_mut(s: &mut Self) -> (&mut Self::Super, &mut Self::Ref) {
        unsafe { T::s_r_mut(*s) }
    }
}

pub trait RefWrapperExt: RefWrapperMutExt + Sized {
    #[allow(clippy::wrong_self_convention)]
    fn into_super(s: Self) -> Self::Super;
}
impl<S, R> RefWrapperTypes for RefWrapper<S, R> {
    type Super = S;
    type Ref = R;

    fn sup(s: &Self) -> &S {
        &s.super_ref
    }

    fn r(s: &Self) -> &R {
        &s.r
    }

    fn s_r(s: &Self) -> (&S, &R) {
        (&s.super_ref, &s.r)
    }
}
impl<S, R> RefWrapperMutExt for RefWrapper<S, R> {
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn sup_mut(s: &mut Self) -> &mut S {
        &mut s.super_ref
    }

    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn r_mut(s: &mut Self) -> &mut R {
        &mut s.r
    }

    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn s_r_mut(s: &mut Self) -> (&mut S, &mut R) {
        (&mut s.super_ref, &mut s.r)
    }
}
impl<S, R> RefWrapperExt for RefWrapper<S, R> {
    fn into_super(s: Self) -> S {
        s.super_ref
    }
}

pub trait RefDeref<S>: Sized {
    type Target: ?Sized;
    fn deref(wrapper: &RefWrapper<S, Self>) -> &Self::Target;
}
/// # Safety
/// Same requirements as [`AsBytes`].
pub unsafe trait RefBytes<S>: Sized {
    fn bytes(wrapper: &RefWrapper<S, Self>) -> Result<&[u8]>;
}
pub trait RefDerefMut<S>: RefDeref<S> {
    fn deref_mut(wrapper: &mut RefWrapper<S, Self>) -> &mut Self::Target;
}
/// # Safety
/// Same requirements as [`AsMutBytes`].
pub unsafe trait RefBytesMut<S>: RefBytes<S> {
    fn bytes_mut(wrapper: &mut RefWrapper<S, Self>) -> Result<&mut [u8]>;
}

/// # Safety
/// Must return the same reference if `self` was not mutably accessed.
pub unsafe trait AsBytes {
    #[allow(clippy::wrong_self_convention)]
    fn as_bytes(s: &Self) -> Result<&[u8]>;
}
/// # Safety
/// Must return the same reference as [`AsBytes::as_bytes`] if `self` was not mutably accessed.
pub unsafe trait AsMutBytes: AsBytes {
    /// # Safety
    /// Modifying the underlying bytes of a [`RefWrapper`] may violate it's safety guarantees
    #[allow(clippy::wrong_self_convention)]
    unsafe fn as_mut_bytes(s: &mut Self) -> Result<&mut [u8]>;
}

/// # Safety
/// Same requirements as [`Resize`].
pub unsafe trait RefResize<S, M>: Sized {
    /// # Safety
    /// Same requirements as [`Resize::resize`].
    unsafe fn resize(
        wrapper: &mut RefWrapper<S, Self>,
        new_byte_len: usize,
        new_meta: M,
    ) -> Result<()>;

    /// # Safety
    /// Same requirements as [`Resize::set_meta`].
    unsafe fn set_meta(wrapper: &mut RefWrapper<S, Self>, new_meta: M) -> Result<()>;
}

unsafe impl<'a, T> AsBytes for &'a T
where
    T: ?Sized + AsBytes,
{
    fn as_bytes(s: &Self) -> Result<&[u8]> {
        T::as_bytes(*s)
    }
}
unsafe impl<'a, T> AsBytes for &'a mut T
where
    T: ?Sized + AsBytes,
{
    fn as_bytes(s: &Self) -> Result<&[u8]> {
        T::as_bytes(*s)
    }
}
unsafe impl<'a, T> AsMutBytes for &'a mut T
where
    T: ?Sized + AsMutBytes,
{
    unsafe fn as_mut_bytes(s: &mut Self) -> Result<&mut [u8]> {
        unsafe { T::as_mut_bytes(*s) }
    }
}

unsafe impl AsBytes for [u8] {
    fn as_bytes(s: &Self) -> Result<&[u8]> {
        Ok(s)
    }
}
unsafe impl AsMutBytes for [u8] {
    unsafe fn as_mut_bytes(s: &mut Self) -> Result<&mut [u8]> {
        Ok(s)
    }
}
unsafe impl AsBytes for Vec<u8> {
    fn as_bytes(s: &Self) -> Result<&[u8]> {
        Ok(s)
    }
}
unsafe impl AsMutBytes for Vec<u8> {
    unsafe fn as_mut_bytes(s: &mut Self) -> Result<&mut [u8]> {
        Ok(s)
    }
}

impl<S, R> Deref for RefWrapper<S, R>
where
    R: RefDeref<S>,
{
    type Target = R::Target;
    fn deref(&self) -> &Self::Target {
        R::deref(self)
    }
}
unsafe impl<S, R> AsBytes for RefWrapper<S, R>
where
    R: RefBytes<S>,
{
    fn as_bytes(s: &Self) -> Result<&[u8]> {
        R::bytes(s)
    }
}
impl<S, R> DerefMut for RefWrapper<S, R>
where
    R: RefDerefMut<S>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        R::deref_mut(self)
    }
}
unsafe impl<S, R> AsMutBytes for RefWrapper<S, R>
where
    R: RefBytesMut<S>,
{
    unsafe fn as_mut_bytes(s: &mut Self) -> Result<&mut [u8]> {
        R::bytes_mut(s)
    }
}
unsafe impl<S, R, M> Resize<M> for RefWrapper<S, R>
where
    Self: AsMutBytes,
    R: RefResize<S, M>,
{
    unsafe fn resize(s: &mut Self, new_byte_len: usize, new_meta: M) -> Result<()> {
        unsafe { R::resize(s, new_byte_len, new_meta) }
    }

    unsafe fn set_meta(s: &mut Self, new_meta: M) -> Result<()> {
        unsafe { R::set_meta(s, new_meta) }
    }
}
