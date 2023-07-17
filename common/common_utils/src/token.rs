//! Token Utils.

use anchor_lang::{
    prelude::*,
    solana_program::program_pack::{IsInitialized, Pack},
};
use bytemuck::{Pod, Zeroable};
use derivative::Derivative;
use solana_program::program_option::COption;
use static_assertions::const_assert_eq;
use std::mem::size_of;

/// Check if the account is owned by the given owner
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
    if account.owner == owner {
        Ok(())
    } else {
        Err(Error::from(ProgramError::IllegalOwner).with_source(source!()))
    }
}

/// Check if an account is initialized
pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if account.is_initialized() {
        Ok(account)
    } else {
        Err(Error::from(ProgramError::UninitializedAccount).with_source(source!()))
    }
}

/// Check that the account is an initialized mint account
pub fn assert_mint(account_info: &AccountInfo) -> Result<spl_token::state::Mint> {
    assert_owned_by(account_info, &spl_token::id())?;
    let mint_account: spl_token::state::Mint = assert_initialized(account_info)?;
    Ok(mint_account)
}

/// Check that the account is an initialized token account
pub fn assert_token(account_info: &AccountInfo) -> Result<spl_token::state::Account> {
    assert_owned_by(account_info, &spl_token::id())?;
    let mint_account: spl_token::state::Account = assert_initialized(account_info)?;
    Ok(mint_account)
}

/// A [`COption`] with the [`Pod`] trait
#[derive(Zeroable, Copy, Derivative)]
#[derivative(Debug, Clone)]
#[repr(C, packed)]
pub struct PodCOption<T: Pod> {
    has_value: u32,
    value: T,
}
impl<T: Pod> PodCOption<T> {
    /// Create a new [`PodCOption`] from an [`Option`]
    pub fn new(value: Option<T>) -> Self {
        match value {
            Some(value) => Self {
                has_value: 1,
                value,
            },
            None => Self {
                has_value: 0,
                value: T::zeroed(),
            },
        }
    }

    /// Get the contained value
    pub fn value(self) -> Option<T> {
        if self.has_value > 0 {
            Some(self.value)
        } else {
            None
        }
    }
}
// Safety: This is safe because `PodCOption` is `#[repr(C, packed)]`
unsafe impl<T: Pod> Pod for PodCOption<T> {}

const_assert_eq!(
    size_of::<COption<Pubkey>>(),
    size_of::<PodCOption<Pubkey>>()
);
