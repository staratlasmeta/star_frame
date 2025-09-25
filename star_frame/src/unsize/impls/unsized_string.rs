//! Resizable UTF-8 string type.
//!
//! This module provides [`UnsizedString`], a variable-sized string represented as a [`List`] of bytes.

use crate::{
    prelude::*,
    unsize::{impls::ListLength, FromOwned},
};

/// A resizable UTF-8 string type.
///
/// Under the hood, this is a [`List`] of bytes, prefixed with `L` representing the length of the string, defaulting to `u32`.
///
/// See [`Self::as_str`](UnsizedStringMut::as_str) and [`Self::as_mut_str`](UnsizedStringMut::as_mut_str) to access the underlying string slice,
/// and [`UnsizedStringExclusiveImpl::set`] to set the string to a given value.

#[unsized_type(skip_idl, owned_type = String, owned_from_ref = unsized_string_owned_from_ref)]
pub struct UnsizedString<L = u32>
where
    L: ListLength,
{
    #[unsized_start]
    chars: List<u8, L>,
}

#[unsized_impl]
impl<L> UnsizedString<L>
where
    L: ListLength,
{
    /// Returns a shared reference to the underlying string slice.
    ///
    /// # Errors
    /// Returns an error if the underlying bytes are not valid UTF-8.
    pub fn as_str(&self) -> Result<&str> {
        Ok(std::str::from_utf8(self.chars.as_slice())?)
    }

    /// Returns a mutable reference to the underlying string slice.
    ///
    /// # Errors
    /// Returns an error if the underlying bytes are not valid UTF-8.
    pub fn as_mut_str(&mut self) -> Result<&mut str> {
        Ok(std::str::from_utf8_mut(self.chars.as_mut_slice())?)
    }

    /// Sets the string to the given value.
    #[exclusive]
    pub fn set(&mut self, s: impl AsRef<str>) -> Result<()> {
        let mut chars = self.chars();
        chars.clear()?;
        chars.push_all(s.as_ref().as_bytes().iter().copied())?;
        Ok(())
    }
}

fn unsized_string_owned_from_ref<L>(r: &UnsizedStringRef<'_, L>) -> Result<String>
where
    L: ListLength,
{
    r.as_str().map(ToOwned::to_owned)
}

impl<L> FromOwned for UnsizedString<L>
where
    L: ListLength,
{
    fn byte_size(owned: &Self::Owned) -> usize {
        List::<u8, L>::byte_size_from_len(owned.len())
    }

    fn from_owned(owned: Self::Owned, bytes: &mut &mut [u8]) -> Result<usize> {
        List::<u8, L>::from_owned_from_iter(owned.bytes(), bytes)
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

    impl TypeToIdl for UnsizedString<u32> {
        type AssociatedProgram = System;

        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlTypeDef> {
            Ok(IdlTypeDef::String)
        }
    }
}
