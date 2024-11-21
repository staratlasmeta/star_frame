use crate::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use derivative::Derivative;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

/// Allows setting a [`KeyFor`] using other types.
pub trait SetKeyFor<T: ?Sized, I> {
    /// Sets the contained pubkey.
    fn set_pubkey(&mut self, pubkey: I);
}

/// Allows getting a [`KeyFor`] from other types.
pub trait GetKeyFor<T: ?Sized> {
    /// Gets the contained `KeyFor`.
    fn key_for(&self) -> KeyFor<T>;
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

impl<T: ?Sized> Display for KeyFor<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pubkey)
    }
}
impl<T: ?Sized> KeyFor<T> {
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

impl<'info, T: HasProgramAccount + SingleAccountSet<'info>> SetKeyFor<T::ProgramAccount, &T>
    for KeyFor<T::ProgramAccount>
{
    fn set_pubkey(&mut self, pubkey: &T) {
        self.pubkey = *pubkey.key();
    }
}

impl<'info, T: HasProgramAccount + SingleAccountSet<'info>> GetKeyFor<T::ProgramAccount> for T {
    fn key_for(&self) -> KeyFor<T::ProgramAccount> {
        (*self.key()).into()
    }
}

impl<'info, T: HasProgramAccount + SingleAccountSet<'info>> PartialEq<T> for KeyFor<T> {
    fn eq(&self, other: &T) -> bool {
        self.pubkey == *(other.key())
    }
}

impl<T: ?Sized> From<Pubkey> for KeyFor<T> {
    fn from(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            phantom: PhantomData,
        }
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

    impl<T: AccountToIdl + ?Sized> TypeToIdl for KeyFor<T> {
        type AssociatedProgram = SystemProgram;
        fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
            Ok(IdlTypeDef::PubkeyFor {
                id: T::account_to_idl(idl_definition)?,
                optional: false,
            })
        }
    }
}
