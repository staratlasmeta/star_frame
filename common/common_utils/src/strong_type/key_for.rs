use anchor_lang::ZeroCopy;
use bytemuck::{Pod, Zeroable};
use common_utils::prelude::*;
use derivative::Derivative;
use solana_program::pubkey::Pubkey;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Allows setting a [`KeyFor`] from an [`AccountLoader`] or an [`Account`].
pub trait SetKeyFor<T, I> {
    /// Sets the contained pubkey.
    fn set_pubkey(&mut self, pubkey: I);
}

/// A key for an account type
#[derive(Derivative)]
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
pub struct KeyFor<T> {
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
impl<'a, 'info, T: Owner + ZeroCopy> SetKeyFor<T, &'a AccountLoader<'info, T>> for KeyFor<T> {
    fn set_pubkey(&mut self, pubkey: &'a AccountLoader<'info, T>) {
        self.pubkey = pubkey.key();
    }
}
impl<'a, 'info, T: Owner + AccountSerialize + AccountDeserialize + Clone>
    SetKeyFor<T, &'a Account<'info, T>> for KeyFor<T>
{
    fn set_pubkey(&mut self, pubkey: &'a Account<'info, T>) {
        self.pubkey = pubkey.key();
    }
}
impl<'info, T: Owner + ZeroCopy> PartialEq<AccountLoader<'info, T>> for KeyFor<T> {
    fn eq(&self, other: &AccountLoader<'info, T>) -> bool {
        self.pubkey == other.key()
    }
}
impl<'info, T: Owner + AccountSerialize + AccountDeserialize + Clone> PartialEq<Account<'info, T>>
    for KeyFor<T>
{
    fn eq(&self, other: &Account<'info, T>) -> bool {
        self.pubkey == other.key()
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

/// Adds methods to get a [`KeyFor`] from an [`AccountLoader`].
pub trait KeyForAnchor<T> {
    /// Gets a [`KeyFor`] from an [`AccountLoader`].
    fn key_for(&self) -> KeyFor<T>;
}
impl<'info, T: Owner + ZeroCopy> KeyForAnchor<T> for AccountLoader<'info, T> {
    fn key_for(&self) -> KeyFor<T> {
        KeyFor {
            pubkey: self.key(),
            phantom: PhantomData,
        }
    }
}
impl<'info, T: Owner + AccountSerialize + AccountDeserialize + Clone> KeyForAnchor<T>
    for Account<'info, T>
{
    fn key_for(&self) -> KeyFor<T> {
        KeyFor {
            pubkey: self.key(),
            phantom: PhantomData,
        }
    }
}
