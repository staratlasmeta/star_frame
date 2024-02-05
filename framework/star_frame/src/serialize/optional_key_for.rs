use crate::prelude::*;
use crate::serialize::key_for::KeyFor;
use borsh;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast, cast_mut, cast_ref, Pod, Zeroable};
use derivative::Derivative;
use solana_program::pubkey::Pubkey;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Allows setting a [`OptionalKeyFor`] from an [`AccountLoader`] or an [`Account`].
pub trait SetOptionalKeyFor<T: ?Sized, I> {
    /// Sets the contained pubkey.
    fn set_pubkey(&mut self, pubkey: Option<I>);
}

/// A key for an account type
#[derive(Derivative, BorshDeserialize, BorshSerialize)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Copy(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
#[repr(transparent)]
pub struct OptionalKeyFor<T: ?Sized> {
    pubkey: Pubkey,
    phantom: PhantomData<fn() -> T>,
}

impl<T> Display for OptionalKeyFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pubkey)
    }
}
impl<T> OptionalKeyFor<T> {
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

impl<'a, 'info, T: AccountData + ?Sized> SetOptionalKeyFor<T, &'a DataAccount<'info, T>>
    for OptionalKeyFor<T>
{
    fn set_pubkey(&mut self, pubkey: Option<&'a DataAccount<'info, T>>) {
        self.pubkey = pubkey.map_or_else(solana_program::system_program::id, |d| *(d.key()));
    }
}

impl<T> PartialEq<KeyFor<T>> for OptionalKeyFor<T> {
    fn eq(&self, other: &KeyFor<T>) -> bool {
        self.pubkey == *other.pubkey()
    }
}

impl<T> PartialEq<OptionalKeyFor<T>> for KeyFor<T> {
    fn eq(&self, other: &OptionalKeyFor<T>) -> bool {
        *self.pubkey() == other.pubkey
    }
}

impl<'info, T: AccountData + ?Sized> PartialEq<DataAccount<'info, T>> for OptionalKeyFor<T> {
    fn eq(&self, other: &DataAccount<'info, T>) -> bool {
        self.pubkey == *(other.key())
    }
}

impl<T> From<Pubkey> for OptionalKeyFor<T> {
    fn from(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            phantom: PhantomData,
        }
    }
}

impl<T> From<KeyFor<T>> for OptionalKeyFor<T>
where
    T: 'static,
{
    fn from(key: KeyFor<T>) -> Self {
        cast(key)
    }
}

// Safety: `OptionalKeyFor` is a transparent wrapper around a `Pubkey` which is `Zeroable`
unsafe impl<T> Zeroable for OptionalKeyFor<T>
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
unsafe impl<T: 'static> Pod for OptionalKeyFor<T> where Pubkey: Pod {}
