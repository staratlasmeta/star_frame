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
        FromOwned, RawSliceAdvance, UnsizedType, UnsizedTypePtr,
    },
    Result,
};
use advancer::Advance;
use alloc::format;
use bytemuck::{checked, CheckedBitPattern, NoUninit, Zeroable};
use core::{
    mem::size_of,
    ops::{Deref, DerefMut},
};

/// This is a helper trait for the [`UnsizedType`] trait. The required supertraits meet the [`CheckedBitPattern`] blanket implementation for [`UnsizedType`].
pub trait UnsizedGenerics: CheckedBitPattern + Align1 + NoUninit + Zeroable {}
impl<T> UnsizedGenerics for T where T: CheckedBitPattern + Align1 + NoUninit + Zeroable {}

#[derive(Debug)]
pub struct CheckedPtr<T>(*mut T)
where
    T: CheckedBitPattern + NoUninit + Align1;

impl<T> Deref for CheckedPtr<T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<T> DerefMut for CheckedPtr<T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

unsafe impl<T: CheckedBitPattern + NoUninit + Align1> UnsizedTypePtr for CheckedPtr<T> {
    type UnsizedType = T;
    fn check_pointers(&self, range: &core::ops::Range<usize>, cursor: &mut usize) -> bool {
        let addr = self.0.addr();
        let is_advanced = addr >= *cursor;
        *cursor = addr;
        is_advanced && range.contains(&addr)
    }
}

unsafe impl<T> UnsizedType for T
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Ptr = CheckedPtr<T>;
    type Owned = Self;
    const ZST_STATUS: bool = { size_of::<T>() != 0 };

    #[inline]
    unsafe fn get_ptr(data: &mut *mut [u8]) -> Result<Self::Ptr> {
        let sized = data.try_advance(size_of::<T>()).with_ctx(|| {
            format!(
                "Failed to read {} mutable bytes for checked type {}",
                size_of::<T>(),
                core::any::type_name::<T>()
            )
        })?;

        checked::try_from_bytes::<T>(unsafe { &*sized.cast_const() })?;
        Ok(CheckedPtr(sized.cast()))
    }

    #[inline]
    fn data_len(_m: &Self::Ptr) -> usize {
        size_of::<T>()
    }

    #[inline]
    fn start_ptr(m: &Self::Ptr) -> *mut () {
        m.0.cast::<()>()
    }

    fn owned_from_ptr(r: &Self::Ptr) -> Result<Self::Owned> {
        Ok(**r)
    }

    #[inline]
    unsafe fn resize_notification(
        self_mut: &mut Self::Ptr,
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
                    core::any::type_name::<Self>()
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
                    core::any::type_name::<T>()
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
                    core::any::type_name::<Self>()
                )
            })?
            .copy_from_slice(bytemuck::bytes_of(&T::default_init()));
        Ok(())
    }
}
