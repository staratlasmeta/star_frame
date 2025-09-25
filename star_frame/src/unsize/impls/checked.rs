//! Blanket [`UnsizedType`] implementation for types with validated bit patterns.
//!
//! This module provides a blanket [`UnsizedType`] implementation for any type `T` that implements
//! [`CheckedBitPattern`] + [`NoUninit`] + [`Align1`]. This enables any such type to automatically work
//! within the unsized type system with runtime validation for data integrity.
use crate::{
    align1::Align1,
    errors::ErrorInfo as _,
    unsize::{
        init::{DefaultInit, DefaultInitable, UnsizedInit},
        AsShared, FromOwned, RawSliceAdvance, UnsizedType, UnsizedTypeMut,
    },
    Result,
};
use advancer::Advance;
use bytemuck::{checked, CheckedBitPattern, NoUninit, Zeroable};
use std::{
    marker::PhantomData,
    mem::size_of,
    ops::{Deref, DerefMut},
};

/// This is a helper trait for the [`UnsizedType`] trait. The required supertraits meet the [`CheckedBitPattern`] blanket implementation for [`UnsizedType`].
pub trait UnsizedGenerics: CheckedBitPattern + Align1 + NoUninit + Zeroable {}
impl<T> UnsizedGenerics for T where T: CheckedBitPattern + Align1 + NoUninit + Zeroable {}

#[derive(Debug)]
pub struct CheckedRef<'a, T>(*const T, PhantomData<&'a ()>)
where
    T: CheckedBitPattern + NoUninit + Align1;

impl<T> Deref for CheckedRef<'_, T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
#[derive(Debug)]
pub struct CheckedMut<'a, T>(*mut T, PhantomData<&'a ()>)
where
    T: CheckedBitPattern + NoUninit + Align1;
impl<T> Deref for CheckedMut<'_, T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<T> DerefMut for CheckedMut<'_, T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl<T> AsShared for CheckedRef<'_, T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Ref<'a>
        = CheckedRef<'a, T>
    where
        Self: 'a;
    fn as_shared(&self) -> Self::Ref<'_> {
        CheckedRef(self.0, PhantomData)
    }
}
impl<T> AsShared for CheckedMut<'_, T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Ref<'a>
        = CheckedRef<'a, T>
    where
        Self: 'a;
    fn as_shared(&self) -> Self::Ref<'_> {
        T::mut_as_ref(self)
    }
}

unsafe impl<T: CheckedBitPattern + NoUninit + Align1> UnsizedTypeMut for CheckedMut<'_, T> {
    type UnsizedType = T;
}

unsafe impl<T> UnsizedType for T
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Ref<'a> = CheckedRef<'a, T>;
    type Mut<'a> = CheckedMut<'a, T>;
    type Owned = Self;
    const ZST_STATUS: bool = { size_of::<T>() != 0 };

    #[inline]
    fn ref_as_ref<'a>(r: &'a Self::Ref<'_>) -> Self::Ref<'a> {
        CheckedRef(r.0, r.1)
    }

    #[inline]
    fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a> {
        CheckedRef(m.0, m.1)
    }

    #[inline]
    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        checked::try_from_bytes(data.try_advance(size_of::<T>()).with_ctx(|| {
            format!(
                "Failed to read {} bytes for checked type {}",
                size_of::<T>(),
                std::any::type_name::<T>()
            )
        })?)
        .map(std::ptr::from_ref)
        .map(|r| CheckedRef(r, PhantomData))
        .ctx("Invalid data for type")
    }

    #[inline]
    unsafe fn get_mut<'a>(data: &mut *mut [u8]) -> Result<Self::Mut<'a>> {
        let sized = data.try_advance(size_of::<T>()).with_ctx(|| {
            format!(
                "Failed to read {} mutable bytes for checked type {}",
                size_of::<T>(),
                std::any::type_name::<T>()
            )
        })?;

        checked::try_from_bytes::<T>(unsafe { &*sized })?;
        Ok(CheckedMut(sized.cast(), PhantomData))
    }

    #[inline]
    fn data_len(_m: &Self::Mut<'_>) -> usize {
        size_of::<T>()
    }

    #[inline]
    fn start_ptr(m: &Self::Mut<'_>) -> *mut () {
        m.0.cast::<()>()
    }

    fn owned_from_ref(r: &Self::Ref<'_>) -> Result<Self::Owned> {
        Ok(**r)
    }

    #[inline]
    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()> {
        let self_ptr = self_mut.0;
        if source_ptr < self_ptr.cast_const().cast() {
            self_mut.0 = self_ptr.wrapping_byte_offset(change);
        }
        Ok(())
    }
}

impl<T> FromOwned for T
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    fn byte_size(_owned: &Self::Owned) -> usize {
        size_of::<T>()
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        bytes
            .try_advance(size_of::<T>())
            .with_ctx(|| {
                format!(
                    "Failed to advance bytes during `FromOwned` of {}",
                    std::any::type_name::<Self>()
                )
            })?
            .copy_from_slice(bytemuck::bytes_of(&owned));
        Ok(size_of::<T>())
    }
}

impl<T> UnsizedInit<T> for T
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    const INIT_BYTES: usize = size_of::<T>();

    #[inline]
    fn init(bytes: &mut &mut [u8], arg: T) -> Result<()> {
        bytes
            .try_advance(size_of::<T>())
            .with_ctx(|| {
                format!(
                    "Failed to advance bytes during initialization of {}",
                    std::any::type_name::<T>()
                )
            })?
            .copy_from_slice(bytemuck::bytes_of(&arg));
        Ok(())
    }
}

impl<T> UnsizedInit<DefaultInit> for T
where
    T: CheckedBitPattern + NoUninit + Align1 + DefaultInitable,
{
    const INIT_BYTES: usize = size_of::<T>();

    #[inline]
    fn init(bytes: &mut &mut [u8], _arg: DefaultInit) -> Result<()> {
        bytes
            .try_advance(size_of::<T>())
            .with_ctx(|| {
                format!(
                    "Failed to advance bytes during default initialization of {}",
                    std::any::type_name::<Self>()
                )
            })?
            .copy_from_slice(bytemuck::bytes_of(&T::default_init()));
        Ok(())
    }
}
