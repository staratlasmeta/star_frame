use crate::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast, cast_mut, cast_ref};
use derive_where::DeriveWhere;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Allows getting an [`OptionalKeyFor`] from other types.
pub trait GetOptionalKeyFor<T: ?Sized> {
    /// Gets the contained `OptionalKeyFor`.
    fn optional_key_for(&self) -> OptionalKeyFor<T>;
}

/// A key for an account type
#[derive(BorshDeserialize, BorshSerialize, Align1, DeriveWhere)]
#[derive_where(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct OptionalKeyFor<T: ?Sized> {
    pubkey: Pubkey,
    phantom: PhantomData<fn() -> T>,
}

impl<T: ?Sized> Default for OptionalKeyFor<T> {
    fn default() -> Self {
        Self::NONE
    }
}

/// An optionally set [`Pubkey`].
pub type OptionalPubkey = OptionalKeyFor<()>;

impl<T> Display for OptionalKeyFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pubkey)
    }
}
impl<T: ?Sized> OptionalKeyFor<T> {
    /// Gets the contained pubkey.
    /// An [`OptionalKeyFor`] with the [`None`] variant.
    pub const NONE: OptionalKeyFor<T> = OptionalKeyFor {
        pubkey: solana_program::system_program::id(),
        phantom: PhantomData,
    };

    /// Creates a new [`OptionalKeyFor`] for any `T`.
    #[must_use]
    pub fn new(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            phantom: PhantomData,
        }
    }

    /// Gets the contained pub
    #[must_use]
    pub fn pubkey(&self) -> Option<&Pubkey> {
        if self.pubkey == solana_program::system_program::id() {
            None
        } else {
            Some(&self.pubkey)
        }
    }

    /// Pulls out the contained pubkey.
    #[must_use]
    pub fn as_inner(&self) -> &Pubkey {
        &self.pubkey
    }

    /// Sets the contained pubkey.
    pub fn set_pubkey_direct(&mut self, pubkey: Option<Pubkey>) {
        self.pubkey = pubkey.unwrap_or_else(solana_program::system_program::id);
    }
}

impl<'info, T: HasInnerType + SingleAccountSet<'info>> SetKeyFor<T::Inner, &T>
    for OptionalKeyFor<T::Inner>
{
    fn set_pubkey(&mut self, pubkey: &T) {
        self.pubkey = *(pubkey.key());
    }
}

impl<'info, T: HasInnerType + SingleAccountSet<'info>> SetKeyFor<T::Inner, &Option<T>>
    for OptionalKeyFor<T::Inner>
{
    fn set_pubkey(&mut self, pubkey: &Option<T>) {
        self.pubkey = pubkey
            .as_ref()
            .map_or_else(solana_program::system_program::id, |acc| *(acc.key()));
    }
}

impl<'info, T: HasInnerType + SingleAccountSet<'info>> GetOptionalKeyFor<T::Inner> for T {
    fn optional_key_for(&self) -> OptionalKeyFor<T::Inner> {
        self.key_for().into()
    }
}

impl<'info, T: HasInnerType + SingleAccountSet<'info>> GetOptionalKeyFor<T::Inner> for Option<T> {
    fn optional_key_for(&self) -> OptionalKeyFor<T::Inner> {
        self.as_ref()
            .map_or(OptionalKeyFor::NONE, GetOptionalKeyFor::optional_key_for)
    }
}

impl<T: ?Sized> PartialEq<KeyFor<T>> for OptionalKeyFor<T> {
    fn eq(&self, other: &KeyFor<T>) -> bool {
        self.pubkey == *other.pubkey()
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

// Safety: `OptionalKeyFor` is a transparent wrapper around a `Pubkey` which is `Zeroable`
#[allow(trivial_bounds)]
unsafe impl<T: ?Sized> Zeroable for OptionalKeyFor<T>
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
// Safety: `OptionalKeyFor` is a transparent wrapper around a `Pubkey` which is `Pod`
#[allow(trivial_bounds)]
unsafe impl<T: 'static + ?Sized> Pod for OptionalKeyFor<T> where Pubkey: Pod {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame_idl::ty::IdlTypeDef;
    use star_frame_idl::IdlDefinition;

    impl<T> TypeToIdl for OptionalKeyFor<T> {
        type AssociatedProgram = System;
        fn type_to_idl(_idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::Pubkey)
        }
    }
}
