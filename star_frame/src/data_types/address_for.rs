use crate::{account_set::modifiers::HasInnerType, prelude::*};
use bytemuck::{cast, cast_mut, cast_ref};
use core::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};
use derive_where::DeriveWhere;
use serde::{Deserialize, Serialize};

/// Allows setting a [`AddressFor`] or [`OptionalAddressFor`] using other types.
pub trait SetAddressFor<T: ?Sized, I> {
    /// Sets the contained address.
    fn set_addr(&mut self, address: I);
}

/// Allows getting a [`AddressFor`] from other types.
pub trait GetAddressFor<T: ?Sized> {
    /// Gets the contained `AddressFor`.
    fn addr_for(&self) -> &AddressFor<T>;
}

/// A key for an account type
#[derive(
    borsh::BorshDeserialize, borsh::BorshSerialize, Align1, DeriveWhere, Serialize, Deserialize,
)]
#[derive_where(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, PartialOrd, Ord)]
#[serde(transparent)]
#[repr(transparent)]
pub struct AddressFor<T: ?Sized> {
    address: Address,
    phantom: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Display for AddressFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.address)
    }
}
impl<T: ?Sized> AddressFor<T> {
    /// Creates a new [`AddressFor`] for any `T`.
    #[must_use]
    pub fn new(address: Address) -> Self {
        Self {
            address,
            phantom: PhantomData,
        }
    }

    /// Returns a reference to [`AddressFor`] for any `T` from a reference to a `Address`.
    #[must_use]
    pub fn new_ref(address: &Address) -> &Self
    where
        T: 'static,
    {
        bytemuck::cast_ref(address)
    }

    /// Returns a reference to the contained address.
    #[must_use]
    pub fn addr(&self) -> &Address {
        &self.address
    }

    /// Sets the contained address.
    pub fn set_addr_direct(&mut self, address: Address) {
        self.address = address;
    }
}

impl<T: HasInnerType + SingleAccountSet> SetAddressFor<T::Inner, &T> for AddressFor<T::Inner> {
    fn set_addr(&mut self, address: &T) {
        self.address = *address.addr();
    }
}

impl<T: HasInnerType + SingleAccountSet> GetAddressFor<T::Inner> for T {
    fn addr_for(&self) -> &AddressFor<T::Inner> {
        AddressFor::new_ref(self.addr())
    }
}

impl<T: ?Sized> PartialEq<OptionalAddressFor<T>> for AddressFor<T> {
    fn eq(&self, other: &OptionalAddressFor<T>) -> bool {
        self.addr() == other.as_inner()
    }
}
impl<'a, T: ?Sized + 'static> From<&'a mut OptionalAddressFor<T>> for &'a mut AddressFor<T> {
    fn from(key_for: &'a mut OptionalAddressFor<T>) -> Self {
        cast_mut(key_for)
    }
}

impl<'a, T: ?Sized + 'static> From<&'a OptionalAddressFor<T>> for &'a AddressFor<T> {
    fn from(key_for: &'a OptionalAddressFor<T>) -> Self {
        cast_ref(key_for)
    }
}

impl<T: 'static + ?Sized> From<OptionalAddressFor<T>> for AddressFor<T> {
    fn from(key: OptionalAddressFor<T>) -> Self {
        cast(key)
    }
}

// SAFETY:
// `AddressFor` is a transparent wrapper around a `Address` which is `Zeroable`
#[allow(trivial_bounds)]
unsafe impl<T: ?Sized> Zeroable for AddressFor<T>
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
// `AddressFor` is a transparent wrapper around a `Address` which is `Pod`
#[allow(trivial_bounds)]
unsafe impl<T: 'static + ?Sized> Pod for AddressFor<T> where Address: Pod {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

    impl<T> TypeToIdl for AddressFor<T> {
        type AssociatedProgram = System;
        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlTypeDef> {
            Ok(IdlTypeDef::Address)
        }
    }
}
