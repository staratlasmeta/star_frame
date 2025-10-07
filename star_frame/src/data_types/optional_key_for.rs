use crate::{account_set::modifiers::HasInnerType, data_types::SetKeyFor, prelude::*};
use bytemuck::{cast, cast_mut, cast_ref};
use core::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};
use derive_where::DeriveWhere;
use serde::{Deserialize, Serialize};

/// Allows getting an [`OptionalKeyFor`] from other types.
pub trait GetOptionalKeyFor<T: ?Sized> {
    /// Gets the contained `OptionalKeyFor`.
    fn optional_key_for(&self) -> &OptionalKeyFor<T>;
}

/// An optional key for an account type
#[derive(
    borsh::BorshDeserialize, borsh::BorshSerialize, Align1, DeriveWhere, Serialize, Deserialize,
)]
#[derive_where(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
#[repr(transparent)]
pub struct OptionalKeyFor<T: ?Sized> {
    address: Address,
    phantom: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Default for OptionalKeyFor<T> {
    fn default() -> Self {
        Self::NONE
    }
}

/// An optionally set [`Address`].
pub type OptionalAddress = OptionalKeyFor<()>;

impl<T> Display for OptionalKeyFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.address)
    }
}
impl<T: ?Sized> OptionalKeyFor<T> {
    /// Gets the contained address.
    /// An [`OptionalKeyFor`] with the [`None`] variant.
    pub const NONE: OptionalKeyFor<T> = OptionalKeyFor {
        address: Address::new_from_array([0; 32]),
        phantom: PhantomData,
    };

    /// Creates a new [`OptionalKeyFor`] for any `T`.
    #[must_use]
    pub fn new(address: Address) -> Self {
        Self {
            address,
            phantom: PhantomData,
        }
    }

    /// Creates a new reference to [`OptionalKeyFor`] for any `T` from a reference to a `Address`.
    #[must_use]
    pub fn new_ref(address: &Address) -> &Self
    where
        T: 'static,
    {
        bytemuck::cast_ref(address)
    }

    /// Attempts to return a reference to a [`KeyFor`] if the contained address is not [`None`].
    #[must_use]
    pub fn key_for(&self) -> Option<&KeyFor<T>>
    where
        T: 'static,
    {
        if self.address.fast_eq(&Address::new_from_array([0; 32])) {
            None
        } else {
            Some(KeyFor::new_ref(&self.address))
        }
    }

    /// Attempts to return a reference to the contained [`Address`] if not [`None`].
    #[must_use]
    pub fn address(&self) -> Option<&Address> {
        if self.address.fast_eq(&Address::new_from_array([0; 32])) {
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
    pub fn set_address_direct(&mut self, address: Option<Address>) {
        self.address = address.unwrap_or_default();
    }
}

impl<T: HasInnerType + SingleAccountSet> SetKeyFor<T::Inner, &T> for OptionalKeyFor<T::Inner> {
    fn set_address(&mut self, address: &T) {
        self.address = *(address.address());
    }
}

impl<T: HasInnerType + SingleAccountSet> SetKeyFor<T::Inner, &Option<T>>
    for OptionalKeyFor<T::Inner>
{
    fn set_address(&mut self, address: &Option<T>) {
        self.address = address
            .as_ref()
            .map_or_else(Address::default, |acc| *(acc.address()));
    }
}

impl<T: HasInnerType + SingleAccountSet> GetOptionalKeyFor<T::Inner> for T {
    fn optional_key_for(&self) -> &OptionalKeyFor<T::Inner> {
        self.key_for().into()
    }
}

impl<T: HasInnerType + SingleAccountSet> GetOptionalKeyFor<T::Inner> for Option<T> {
    fn optional_key_for(&self) -> &OptionalKeyFor<T::Inner> {
        self.as_ref()
            .map_or(&OptionalKeyFor::NONE, GetOptionalKeyFor::optional_key_for)
    }
}

impl<T: ?Sized> PartialEq<KeyFor<T>> for OptionalKeyFor<T> {
    fn eq(&self, other: &KeyFor<T>) -> bool {
        self.address == *other.address()
    }
}

impl<'a, T: ?Sized + 'static> From<&'a mut KeyFor<T>> for &'a mut OptionalKeyFor<T> {
    fn from(key_for: &'a mut KeyFor<T>) -> Self {
        cast_mut(key_for)
    }
}

impl<'a, T: ?Sized + 'static> From<&'a KeyFor<T>> for &'a OptionalKeyFor<T> {
    fn from(key_for: &'a KeyFor<T>) -> Self {
        cast_ref(key_for)
    }
}

impl<T: 'static + ?Sized> From<KeyFor<T>> for OptionalKeyFor<T> {
    fn from(key: KeyFor<T>) -> Self {
        cast(key)
    }
}

// SAFETY:
// `OptionalKeyFor` is a transparent wrapper around a `Address` which is `Zeroable`
#[allow(trivial_bounds)]
unsafe impl<T: ?Sized> Zeroable for OptionalKeyFor<T>
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
// `OptionalKeyFor` is a transparent wrapper around a `Address` which is `Pod`
#[allow(trivial_bounds)]
unsafe impl<T: 'static + ?Sized> Pod for OptionalKeyFor<T> where Address: Pod {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::{ty::IdlTypeDef, IdlDefinition};

    impl<T> TypeToIdl for OptionalKeyFor<T> {
        type AssociatedProgram = System;
        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> crate::IdlResult<IdlTypeDef> {
            Ok(IdlTypeDef::Address)
        }
    }
}
