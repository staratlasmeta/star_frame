use crate::prelude::*;
use borsh;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use derivative::Derivative;
use solana_program::pubkey::Pubkey;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Allows setting a [`KeyFor`] from a [`DataAccount`].
pub trait SetKeyFor<T: ?Sized, I> {
    /// Sets the contained pubkey.
    fn set_pubkey(&mut self, pubkey: I);
}

/// A key for an account type
#[derive(Derivative, BorshDeserialize, BorshSerialize, Align1)]
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
pub struct KeyFor<T: ?Sized> {
    pubkey: Pubkey,
    phantom: PhantomData<fn() -> T>,
}

impl<T> Display for KeyFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pubkey)
    }
}
impl<T> KeyFor<T> {
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

impl<'a, 'info, T: AccountData + ?Sized> SetKeyFor<T, &'a DataAccount<'info, T>> for KeyFor<T> {
    fn set_pubkey(&mut self, pubkey: &'a DataAccount<'info, T>) {
        self.pubkey = *(pubkey.key());
    }
}

impl<'info, T: AccountData + ?Sized> PartialEq<DataAccount<'info, T>> for KeyFor<T> {
    fn eq(&self, other: &DataAccount<'info, T>) -> bool {
        self.pubkey == *(other.key())
    }
}

impl<T> From<Pubkey> for KeyFor<T> {
    fn from(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            phantom: PhantomData,
        }
    }
}

// Safety: `KeyFor` is a transparent wrapper around a `Pubkey` which is `Zeroable`
unsafe impl<T> Zeroable for KeyFor<T>
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
unsafe impl<T: 'static> Pod for KeyFor<T> where Pubkey: Pod {}
