use crate::{account_set::modifiers::HasInnerType, prelude::*};
use bytemuck::{cast, cast_mut, cast_ref};
use core::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};
use derive_where::DeriveWhere;
use serde::{Deserialize, Serialize};

/// Allows setting a [`KeyFor`] or [`OptionalKeyFor`] using other types.
pub trait SetKeyFor<T: ?Sized, I> {
    /// Sets the contained address.
    fn set_address(&mut self, address: I);
}

/// Allows getting a [`KeyFor`] from other types.
pub trait GetKeyFor<T: ?Sized> {
    /// Gets the contained `KeyFor`.
    fn key_for(&self) -> &KeyFor<T>;
}

/// A key for an account type
#[derive(
    borsh::BorshDeserialize, borsh::BorshSerialize, Align1, DeriveWhere, Serialize, Deserialize,
)]
#[derive_where(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord)]
#[serde(transparent)]
#[repr(transparent)]
pub struct KeyFor<T: ?Sized> {
    address: Address,
    phantom: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Display for KeyFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.address)
    }
}
impl<T: ?Sized> KeyFor<T> {
    /// Creates a new [`KeyFor`] for any `T`.
    #[must_use]
    pub fn new(address: Address) -> Self {
        Self {
            address,
            phantom: PhantomData,
        }
    }

    /// Returns a reference to [`KeyFor`] for any `T` from a reference to a `Address`.
    #[must_use]
    pub fn new_ref(address: &Address) -> &Self
    where
        T: 'static,
    {
        bytemuck::cast_ref(address)
    }

    /// Returns a reference to the contained address.
    #[must_use]
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Sets the contained address.
    pub fn set_address_direct(&mut self, address: Address) {
        self.address = address;
    }
}

impl<T: HasInnerType + SingleAccountSet> SetKeyFor<T::Inner, &T> for KeyFor<T::Inner> {
    fn set_address(&mut self, address: &T) {
        self.address = *address.address();
    }
}

impl<T: HasInnerType + SingleAccountSet> GetKeyFor<T::Inner> for T {
    fn key_for(&self) -> &KeyFor<T::Inner> {
        KeyFor::new_ref(self.address())
    }
}

impl<T: ?Sized> PartialEq<OptionalKeyFor<T>> for KeyFor<T> {
    fn eq(&self, other: &OptionalKeyFor<T>) -> bool {
        self.address().fast_eq(other.as_inner())
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

// SAFETY:
// `KeyFor` is a transparent wrapper around a `Address` which is `Zeroable`
#[allow(trivial_bounds)]
unsafe impl<T: ?Sized> Zeroable for KeyFor<T>
where
    Address: Zeroable,
{
    fn zeroed() -> Self {
        Self {
            address: Address::zeroed(),
            phantom: PhantomData,
        }
    }
}
// SAFETY:
// `KeyFor` is a transparent wrapper around a `Address` which is `Pod`
#[allow(trivial_bounds)]
unsafe impl<T: 'static + ?Sized> Pod for KeyFor<T> where Address: Pod {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

    impl<T> TypeToIdl for KeyFor<T> {
        type AssociatedProgram = System;
        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlTypeDef> {
            Ok(IdlTypeDef::Address)
        }
    }
}
