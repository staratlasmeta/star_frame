mod bool_wrapper;
mod enum_wrapper;
mod fixed_point;
mod key_for;
mod optional_key_for;

use anchor_lang::prelude::AccountLoader;
use anchor_lang::Result;
use anchor_lang::{Owner, ZeroCopy};

pub use bool_wrapper::*;
pub use enum_wrapper::*;
pub use fixed_point::*;
pub use key_for::*;
pub use optional_key_for::*;
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

use crate::SafeZeroCopy;

/// Struct can be converted to fixed point representation
///
/// # Safety
/// [`Self::StrongTyped`] type must be transparent to `Self`.
pub unsafe trait StrongTypedStruct: Debug {
    /// The fixed point version of the struct, must be transparent to `Self`.
    type StrongTyped: ?Sized;

    /// Converts a ref to self to a ref to the fixed point version
    fn as_strong_typed(&self) -> &Self::StrongTyped;
    /// Converts a mutable ref to self to a mutable ref to the fixed point version
    fn as_strong_typed_mut(&mut self) -> &mut Self::StrongTyped;
}

/// Allows loading an Strong Typed [`AccountLoader`] as a [`StrongTypedStruct`].
pub trait StrongTypedAccountLoader {
    /// The Strong Typed version of the Loader's type.
    type LoadedStrongTyped: ?Sized;

    /// Mutably loads the account as strong typed init.
    fn load_init_strong(&self) -> Result<RefMut<Self::LoadedStrongTyped>>;

    /// Loads the account as strong typed.
    fn load_strong(&self) -> Result<Ref<Self::LoadedStrongTyped>>;

    /// Mutably loads the account as strong typed mut.
    fn load_mut_strong(&self) -> Result<RefMut<Self::LoadedStrongTyped>>;
}

impl<'info, T: StrongTypedStruct + ZeroCopy + Owner + SafeZeroCopy> StrongTypedAccountLoader
    for AccountLoader<'info, T>
{
    type LoadedStrongTyped = T::StrongTyped;

    fn load_init_strong(&self) -> Result<RefMut<Self::LoadedStrongTyped>> {
        Ok(RefMut::map(
            self.load_init()?,
            StrongTypedStruct::as_strong_typed_mut,
        ))
    }

    fn load_strong(&self) -> Result<Ref<Self::LoadedStrongTyped>> {
        Ok(Ref::map(self.load()?, StrongTypedStruct::as_strong_typed))
    }

    fn load_mut_strong(&self) -> Result<RefMut<Self::LoadedStrongTyped>> {
        Ok(RefMut::map(
            self.load_mut()?,
            StrongTypedStruct::as_strong_typed_mut,
        ))
    }
}

// Safety: [T] and [T::StrongTyped] have the same layout due to `StrongTypedStruct`'s requirements
unsafe impl<T> StrongTypedStruct for [T]
where
    T: StrongTypedStruct,
    T::StrongTyped: Sized,
{
    type StrongTyped = [T::StrongTyped];

    fn as_strong_typed(&self) -> &Self::StrongTyped {
        // Safety: [T] and [T::StrongTyped] have the same layout due to `StrongTypedStruct`'s requirements
        unsafe { &*(self as *const [T] as *const [T::StrongTyped]) }
    }

    fn as_strong_typed_mut(&mut self) -> &mut Self::StrongTyped {
        // Safety: [T] and [T::StrongTyped] have the same layout due to `StrongTypedStruct`'s requirements
        unsafe { &mut *(self as *mut [T] as *mut [T::StrongTyped]) }
    }
}

/// Can be converted into strong typed.
pub trait IntoStrongTyped {
    /// The output of the conversion.
    type Output;

    /// Converts self into strong typed.
    fn into_strong_typed(self) -> Self::Output;
}
impl<T> IntoStrongTyped for T
where
    T: StrongTypedStruct,
    T::StrongTyped: Copy,
{
    type Output = T::StrongTyped;

    fn into_strong_typed(self) -> Self::Output {
        *self.as_strong_typed()
    }
}
impl<'a, T> IntoStrongTyped for Ref<'a, T>
where
    T: StrongTypedStruct,
{
    type Output = Ref<'a, T::StrongTyped>;

    fn into_strong_typed(self) -> Self::Output {
        Ref::map(self, StrongTypedStruct::as_strong_typed)
    }
}
impl<'a, T> IntoStrongTyped for RefMut<'a, T>
where
    T: StrongTypedStruct,
{
    type Output = RefMut<'a, T::StrongTyped>;

    fn into_strong_typed(self) -> Self::Output {
        RefMut::map(self, StrongTypedStruct::as_strong_typed_mut)
    }
}
