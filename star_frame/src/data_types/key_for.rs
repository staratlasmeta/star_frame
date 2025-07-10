use crate::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast, cast_mut, cast_ref};
use derive_where::DeriveWhere;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Allows setting a [`KeyFor`] or [`OptionalKeyFor`] using other types.
pub trait SetKeyFor<T: ?Sized, I> {
    /// Sets the contained pubkey.
    fn set_pubkey(&mut self, pubkey: I);
}

/// Allows getting a [`KeyFor`] from other types.
pub trait GetKeyFor<T: ?Sized> {
    /// Gets the contained `KeyFor`.
    fn key_for(&self) -> &KeyFor<T>;
}

/// A key for an account type
#[derive(BorshDeserialize, BorshSerialize, Align1, DeriveWhere, Serialize, Deserialize)]
#[derive_where(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyFor<T: ?Sized> {
    pubkey: Pubkey,
    phantom: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Display for KeyFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pubkey)
    }
}
impl<T: ?Sized> KeyFor<T> {
    /// Creates a new [`KeyFor`] for any `T`.
    #[must_use]
    pub fn new(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            phantom: PhantomData,
        }
    }

    /// Creates a new reference to [`KeyFor`] for any `T` from a reference to a `Pubkey`.
    #[must_use]
    pub fn new_ref(pubkey: &Pubkey) -> &Self
    where
        T: 'static,
    {
        bytemuck::cast_ref(pubkey)
    }

    /// Gets the contained pubkey.
    #[must_use]
    pub fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }

    /// Sets the contained pubkey.
    pub fn set_pubkey_direct(&mut self, pubkey: Pubkey) {
        self.pubkey = pubkey;
    }
}

impl<T: HasInnerType + SingleAccountSet> SetKeyFor<T::Inner, &T> for KeyFor<T::Inner> {
    fn set_pubkey(&mut self, pubkey: &T) {
        self.pubkey = *pubkey.pubkey();
    }
}

impl<T: HasInnerType + SingleAccountSet> GetKeyFor<T::Inner> for T {
    fn key_for(&self) -> &KeyFor<T::Inner> {
        KeyFor::new_ref(self.pubkey())
    }
}

impl<T: ?Sized> PartialEq<OptionalKeyFor<T>> for KeyFor<T> {
    fn eq(&self, other: &OptionalKeyFor<T>) -> bool {
        self.pubkey() == other.as_inner()
    }
}
impl<'a, T: ?Sized + 'static> From<&'a mut OptionalKeyFor<T>> for &'a mut KeyFor<T> {
    fn from(key_for: &'a mut OptionalKeyFor<T>) -> Self {
        cast_mut(key_for)
    }
}

impl<'a, T: ?Sized + 'static> From<&'a OptionalKeyFor<T>> for &'a KeyFor<T> {
    fn from(key_for: &'a OptionalKeyFor<T>) -> Self {
        cast_ref(key_for)
    }
}

impl<T: 'static + ?Sized> From<OptionalKeyFor<T>> for KeyFor<T> {
    fn from(key: OptionalKeyFor<T>) -> Self {
        cast(key)
    }
}

// Safety: `KeyFor` is a transparent wrapper around a `Pubkey` which is `Zeroable`
#[allow(trivial_bounds)]
unsafe impl<T: ?Sized> Zeroable for KeyFor<T>
where
    Pubkey: Zeroable,
{
    fn zeroed() -> Self {
        Self {
            pubkey: Pubkey::zeroed(),
            phantom: PhantomData,
        }
    }
}
// Safety: `KeyFor` is a transparent wrapper around a `Pubkey` which is `Pod`
#[allow(trivial_bounds)]
unsafe impl<T: 'static + ?Sized> Pod for KeyFor<T> where Pubkey: Pod {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<T> TypeToIdl for KeyFor<T> {
        type AssociatedProgram = System;
        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::Pubkey)
        }
    }
}
