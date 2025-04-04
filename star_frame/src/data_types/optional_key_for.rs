use crate::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast, cast_mut, cast_ref};
use derive_where::DeriveWhere;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Allows setting an [`OptionalKeyFor`] using other types.
pub trait SetOptionalKeyFor<T: ?Sized, I> {
    /// Sets the contained pubkey.
    fn set_pubkey(&mut self, pubkey: Option<I>);
}

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

    /// Turns a ref to a [`KeyFor`] into a ref to a [`OptionalKeyFor`].
    #[must_use]
    pub fn from_key_for_ref(key_for: &KeyFor<T>) -> &Self
    where
        T: 'static,
    {
        cast_ref(key_for)
    }

    /// Turns a mut ref to a [`KeyFor`] into a mut ref to a [`OptionalKeyFor`].
    #[must_use]
    pub fn from_key_for_mut(key_for: &mut KeyFor<T>) -> &mut Self
    where
        T: 'static,
    {
        cast_mut(key_for)
    }
}

impl<'info, T: HasProgramAccount + SingleAccountSet<'info>> SetOptionalKeyFor<T::ProgramAccount, &T>
    for OptionalKeyFor<T::ProgramAccount>
{
    fn set_pubkey(&mut self, pubkey: Option<&T>) {
        self.pubkey = pubkey.map_or_else(solana_program::system_program::id, |d| *(d.key()));
    }
}

impl<'info, T: HasProgramAccount + SingleAccountSet<'info>> GetOptionalKeyFor<T::ProgramAccount>
    for T
{
    fn optional_key_for(&self) -> OptionalKeyFor<T::ProgramAccount> {
        (*self.key()).into()
    }
}

impl<T: ?Sized> PartialEq<KeyFor<T>> for OptionalKeyFor<T> {
    fn eq(&self, other: &KeyFor<T>) -> bool {
        self.pubkey == *other.pubkey()
    }
}

impl<T: ?Sized> PartialEq<OptionalKeyFor<T>> for KeyFor<T> {
    fn eq(&self, other: &OptionalKeyFor<T>) -> bool {
        *self.pubkey() == other.pubkey
    }
}

impl<'info, T: HasProgramAccount + SingleAccountSet<'info>> PartialEq<T> for OptionalKeyFor<T> {
    fn eq(&self, other: &T) -> bool {
        self.pubkey == *(other.key())
    }
}

impl<T: ?Sized> From<Pubkey> for OptionalKeyFor<T> {
    fn from(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            phantom: PhantomData,
        }
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
