use crate::unsize::init::{DefaultInit, DefaultInitable, UnsizedInit};
use crate::unsize::{AsShared, FromOwned, UnsizedType};
use crate::{align1::Align1, Result};
use advancer::Advance;
use anyhow::Context;
use bytemuck::{
    checked::{try_from_bytes, try_from_bytes_mut},
    CheckedBitPattern, NoUninit, Zeroable,
};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};

/// This is a helper trait for the [`UnsizedType`] trait. The required supertraits meet the [`CheckedBitPattern`] blanket implementation for [`UnsizedType`].
pub trait UnsizedGenerics: CheckedBitPattern + Align1 + NoUninit + Zeroable {}
impl<T> UnsizedGenerics for T where T: CheckedBitPattern + Align1 + NoUninit + Zeroable {}

#[derive(Debug, Copy, Clone)]
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

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}
impl<T> DerefMut for CheckedMut<'_, T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

impl<'a, T> AsShared<'a> for CheckedMut<'a, T>
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Ref = CheckedRef<'a, T>;
    fn as_shared(&'a self) -> Self::Ref {
        T::mut_as_ref(self)
    }
}

unsafe impl<T> UnsizedType for T
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    type Ref<'a> = CheckedRef<'a, T>;
    type Mut<'a> = CheckedMut<'a, T>;
    type Owned = Self;
    const ZST_STATUS: bool = { size_of::<T>() != 0 };

    fn mut_as_ref<'a>(m: &'a Self::Mut<'_>) -> Self::Ref<'a> {
        CheckedRef(m.0, m.1)
    }

    fn get_ref<'a>(data: &mut &'a [u8]) -> Result<Self::Ref<'a>> {
        try_from_bytes(data.try_advance(size_of::<T>())?)
            .map(std::ptr::from_ref)
            .map(|r| CheckedRef(r, PhantomData))
            .context("Invalid data for type")
    }

    fn get_mut<'a>(data: &mut &'a mut [u8]) -> Result<Self::Mut<'a>> {
        try_from_bytes_mut(data.try_advance(size_of::<T>())?)
            .map(std::ptr::from_mut)
            .map(|r| CheckedMut(r, PhantomData))
            .context("Invalid data for type")
    }

    fn owned_from_ref(r: Self::Ref<'_>) -> Result<Self::Owned> {
        Ok(*r)
    }

    #[inline]
    unsafe fn resize_notification(
        self_mut: &mut Self::Mut<'_>,
        source_ptr: *const (),
        change: isize,
    ) -> Result<()> {
        let self_ptr = self_mut.0;
        if source_ptr < self_ptr.cast_const().cast() {
            self_mut.0 = unsafe { self_ptr.byte_offset(change) };
        }
        Ok(())
    }
}

unsafe impl<T> FromOwned for T
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    fn byte_size(_owned: &Self::Owned) -> usize {
        size_of::<T>()
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        bytes
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(&owned));
        Ok(size_of::<T>())
    }
}

unsafe impl<T> UnsizedInit<T> for T
where
    T: CheckedBitPattern + NoUninit + Align1,
{
    const INIT_BYTES: usize = size_of::<T>();

    unsafe fn init(bytes: &mut &mut [u8], arg: T) -> Result<()> {
        bytes
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(&arg));
        Ok(())
    }
}

unsafe impl<T> UnsizedInit<DefaultInit> for T
where
    T: CheckedBitPattern + NoUninit + Align1 + DefaultInitable,
{
    const INIT_BYTES: usize = size_of::<T>();

    unsafe fn init(bytes: &mut &mut [u8], _arg: DefaultInit) -> Result<()> {
        bytes
            .try_advance(size_of::<T>())?
            .copy_from_slice(bytemuck::bytes_of(&T::default_init()));
        Ok(())
    }
}
