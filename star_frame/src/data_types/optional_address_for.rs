use crate::{account_set::modifiers::HasInnerType, data_types::SetAddressFor, prelude::*};
use bytemuck::{cast, cast_mut, cast_ref};
use core::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};
use derive_where::DeriveWhere;
use serde::{Deserialize, Serialize};

/// Allows getting an [`OptionalAddressFor`] from other types.
pub trait GetOptionalAddressFor<T: ?Sized> {
    /// Gets the contained `OptionalAddressFor`.
    fn optional_addr_for(&self) -> &OptionalAddressFor<T>;
}

/// An optional key for an account type
#[derive(
    borsh::BorshDeserialize, borsh::BorshSerialize, Align1, DeriveWhere, Serialize, Deserialize,
)]
#[derive_where(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
#[repr(transparent)]
pub struct OptionalAddressFor<T: ?Sized> {
    address: Address,
    phantom: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Default for OptionalAddressFor<T> {
    fn default() -> Self {
        Self::NONE
    }
}

/// An optionally set [`Address`].
pub type OptionalAddress = OptionalAddressFor<()>;

impl<T> Display for OptionalAddressFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.address)
    }
}
impl<T: ?Sized> OptionalAddressFor<T> {
    /// Gets the contained address.
    /// An [`OptionalAddressFor`] with the [`None`] variant.
    pub const NONE: OptionalAddressFor<T> = OptionalAddressFor {
        address: Address::new_from_array([0; 32]),
        phantom: PhantomData,
    };

    /// Creates a new [`OptionalAddressFor`] for any `T`.
    #[must_use]
    pub fn new(address: Address) -> Self {
        Self {
            address,
            phantom: PhantomData,
        }
    }

    /// Creates a new reference to [`OptionalAddressFor`] for any `T` from a reference to a `Address`.
    #[must_use]
    pub fn new_ref(address: &Address) -> &Self
    where
        T: 'static,
    {
        bytemuck::cast_ref(address)
    }

    /// Attempts to return a reference to a [`AddressFor`] if the contained address is not [`None`].
    #[must_use]
    pub fn addr_for(&self) -> Option<&AddressFor<T>>
    where
        T: 'static,
    {
        if &self.address == &Address::new_from_array([0; 32]) {
            None
        } else {
            Some(AddressFor::new_ref(&self.address))
        }
    }

    /// Attempts to return a reference to the contained [`Address`] if not [`None`].
    #[must_use]
    pub fn addr(&self) -> Option<&Address> {
        if &self.address == &Address::new_from_array([0; 32]) {
            None
        } else {
            Some(&self.address)
        }
    }

    /// Returns a reference to the contained [`Address`].
    #[must_use]
    pub fn as_inner(&self) -> &Address {
        &self.address
    }

    /// Sets the contained [`Address`].
    pub fn set_addr_direct(&mut self, address: Option<Address>) {
        self.address = address.unwrap_or_default();
    }
}

impl<T: HasInnerType + SingleAccountSet> SetAddressFor<T::Inner, &T>
    for OptionalAddressFor<T::Inner>
{
    fn set_addr(&mut self, address: &T) {
        self.address = *(address.addr());
    }
}

impl<T: HasInnerType + SingleAccountSet> SetAddressFor<T::Inner, &Option<T>>
    for OptionalAddressFor<T::Inner>
{
    fn set_addr(&mut self, address: &Option<T>) {
        self.address = address
            .as_ref()
            .map_or_else(Address::default, |acc| *(acc.addr()));
    }
}

impl<T: HasInnerType + SingleAccountSet> GetOptionalAddressFor<T::Inner> for T {
    fn optional_addr_for(&self) -> &OptionalAddressFor<T::Inner> {
        self.addr_for().into()
    }
}

impl<T: HasInnerType + SingleAccountSet> GetOptionalAddressFor<T::Inner> for Option<T> {
    fn optional_addr_for(&self) -> &OptionalAddressFor<T::Inner> {
        self.as_ref().map_or(
            &OptionalAddressFor::NONE,
            GetOptionalAddressFor::optional_addr_for,
        )
    }
}

impl<T: ?Sized> PartialEq<AddressFor<T>> for OptionalAddressFor<T> {
    fn eq(&self, other: &AddressFor<T>) -> bool {
        self.address == *other.addr()
    }
}

impl<'a, T: ?Sized + 'static> From<&'a mut AddressFor<T>> for &'a mut OptionalAddressFor<T> {
    fn from(key_for: &'a mut AddressFor<T>) -> Self {
        cast_mut(key_for)
    }
}

impl<'a, T: ?Sized + 'static> From<&'a AddressFor<T>> for &'a OptionalAddressFor<T> {
    fn from(key_for: &'a AddressFor<T>) -> Self {
        cast_ref(key_for)
    }
}

impl<T: 'static + ?Sized> From<AddressFor<T>> for OptionalAddressFor<T> {
    fn from(key: AddressFor<T>) -> Self {
        cast(key)
    }
}

// SAFETY:
// `OptionalAddressFor` is a transparent wrapper around a `Address` which is `Zeroable`
#[allow(trivial_bounds)]
unsafe impl<T: ?Sized> Zeroable for OptionalAddressFor<T>
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
// `OptionalAddressFor` is a transparent wrapper around a `Address` which is `Pod`
#[allow(trivial_bounds)]
unsafe impl<T: 'static + ?Sized> Pod for OptionalAddressFor<T> where Address: Pod {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

    impl<T> TypeToIdl for OptionalAddressFor<T> {
        type AssociatedProgram = System;
        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlTypeDef> {
            Ok(IdlTypeDef::Address)
        }
    }
}
