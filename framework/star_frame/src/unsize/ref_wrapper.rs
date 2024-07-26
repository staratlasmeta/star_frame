use crate::unsize::Resize;
use crate::Result;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Copy, Clone)]
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
    /// Only safe to use if `r` is valid for `super_ref`.
    pub unsafe fn wrap_r<R2>(mut self, f: impl FnOnce(&mut S, R) -> R2) -> RefWrapper<S, R2> {
        let new_r = f(&mut self.super_ref, self.r);
        unsafe { RefWrapper::new(self.super_ref, new_r) }
    }
}
pub trait RefWrapperTypes {
    type Super;
    type Ref;

    fn sup(&self) -> &Self::Super;

    fn r(&self) -> &Self::Ref;

    fn s_r(&self) -> (&Self::Super, &Self::Ref);
}
impl<'a, T> RefWrapperTypes for &'a T
where
    T: RefWrapperTypes,
{
    type Super = T::Super;
    type Ref = T::Ref;

    fn sup(&self) -> &Self::Super {
        T::sup(*self)
    }

    fn r(&self) -> &Self::Ref {
        T::r(*self)
    }

    fn s_r(&self) -> (&Self::Super, &Self::Ref) {
        T::s_r(*self)
    }
}
impl<'a, T> RefWrapperTypes for &'a mut T
where
    T: RefWrapperTypes,
{
    type Super = T::Super;
    type Ref = T::Ref;

    fn sup(&self) -> &Self::Super {
        T::sup(*self)
    }

    fn r(&self) -> &Self::Ref {
        T::r(*self)
    }

    fn s_r(&self) -> (&Self::Super, &Self::Ref) {
        T::s_r(*self)
    }
}
pub trait RefWrapperMutExt: RefWrapperTypes {
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn sup_mut(&mut self) -> &mut Self::Super;
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn r_mut(&mut self) -> &mut Self::Ref;
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn s_r_mut(&mut self) -> (&mut Self::Super, &mut Self::Ref);
}
impl<'a, T> RefWrapperMutExt for &'a mut T
where
    T: RefWrapperMutExt,
{
    unsafe fn sup_mut(&mut self) -> &mut Self::Super {
        unsafe { T::sup_mut(*self) }
    }

    unsafe fn r_mut(&mut self) -> &mut Self::Ref {
        unsafe { T::r_mut(*self) }
    }

    unsafe fn s_r_mut(&mut self) -> (&mut Self::Super, &mut Self::Ref) {
        unsafe { T::s_r_mut(*self) }
    }
}
pub trait RefWrapperExt: RefWrapperMutExt + Sized {
    fn into_super(self) -> Self::Super;
}
impl<S, R> RefWrapperTypes for RefWrapper<S, R> {
    type Super = S;
    type Ref = R;

    fn sup(&self) -> &S {
        &self.super_ref
    }

    fn r(&self) -> &R {
        &self.r
    }

    fn s_r(&self) -> (&S, &R) {
        (&self.super_ref, &self.r)
    }
}
impl<S, R> RefWrapperMutExt for RefWrapper<S, R> {
    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn sup_mut(&mut self) -> &mut S {
        &mut self.super_ref
    }

    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn r_mut(&mut self) -> &mut R {
        &mut self.r
    }

    /// # Safety
    /// Should only be called by implementations for types and not user code.
    unsafe fn s_r_mut(&mut self) -> (&mut S, &mut R) {
        (&mut self.super_ref, &mut self.r)
    }
}
impl<S, R> RefWrapperExt for RefWrapper<S, R> {
    fn into_super(self) -> S {
        self.super_ref
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
    fn as_bytes(&self) -> Result<&[u8]>;
}
/// # Safety
/// Must return the same reference as [`AsBytes::as_bytes`] if `self` was not mutably accessed.
pub unsafe trait AsMutBytes: AsBytes {
    fn as_mut_bytes(&mut self) -> Result<&mut [u8]>;
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
    fn as_bytes(&self) -> Result<&[u8]> {
        T::as_bytes(*self)
    }
}
unsafe impl<'a, T> AsBytes for &'a mut T
where
    T: ?Sized + AsBytes,
{
    fn as_bytes(&self) -> Result<&[u8]> {
        T::as_bytes(*self)
    }
}
unsafe impl<'a, T> AsMutBytes for &'a mut T
where
    T: ?Sized + AsMutBytes,
{
    fn as_mut_bytes(&mut self) -> Result<&mut [u8]> {
        T::as_mut_bytes(*self)
    }
}

unsafe impl AsBytes for [u8] {
    fn as_bytes(&self) -> Result<&[u8]> {
        Ok(self)
    }
}
unsafe impl AsMutBytes for [u8] {
    fn as_mut_bytes(&mut self) -> Result<&mut [u8]> {
        Ok(self)
    }
}
unsafe impl AsBytes for Vec<u8> {
    fn as_bytes(&self) -> Result<&[u8]> {
        Ok(self)
    }
}
unsafe impl AsMutBytes for Vec<u8> {
    fn as_mut_bytes(&mut self) -> Result<&mut [u8]> {
        Ok(self)
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
    fn as_bytes(&self) -> Result<&[u8]> {
        R::bytes(self)
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
    fn as_mut_bytes(&mut self) -> Result<&mut [u8]> {
        R::bytes_mut(self)
    }
}
unsafe impl<S, R, M> Resize<M> for RefWrapper<S, R>
where
    Self: AsMutBytes,
    R: RefResize<S, M>,
{
    unsafe fn resize(&mut self, new_byte_len: usize, new_meta: M) -> Result<()> {
        unsafe { R::resize(self, new_byte_len, new_meta) }
    }

    unsafe fn set_meta(&mut self, new_meta: M) -> Result<()> {
        unsafe { R::set_meta(self, new_meta) }
    }
}
