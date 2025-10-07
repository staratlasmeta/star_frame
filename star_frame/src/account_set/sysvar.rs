//! Typed wrappers for Solana system variables (sysvars).
//!
//! Solana provides several system variables that contain runtime information about the blockchain
//! state. The `Sysvar<T>` type provides type-safe access to these system variables within
//! instruction contexts, automatically validating that the correct sysvar account is provided and
//! providing type-safe access to the sysvar's functionality.

use core::marker::PhantomData;
use pinocchio::{account::Ref, sysvars::slot_hashes::SlotHashes};
use star_frame::prelude::*;
pub trait SysvarId: Sized {
    fn id() -> Address;
}

impl SysvarId for pinocchio::sysvars::rent::Rent {
    fn id() -> Address {
        pinocchio::sysvars::rent::RENT_ID
    }
}

pub const RECENT_BLOCKHASHES_ID: Address = address!("SysvarRecentB1ockHashes11111111111111111111");

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct InstructionsSysvar;

impl SysvarId for InstructionsSysvar {
    fn id() -> Address {
        pinocchio::sysvars::instructions::INSTRUCTIONS_ID
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SlotHashesSysvar;

impl SysvarId for SlotHashesSysvar {
    fn id() -> Address {
        pinocchio::sysvars::slot_hashes::SLOTHASHES_ID
    }
}

/// A typed wrapper for Solana system variable accounts that validates the sysvar address.
///
/// This type ensures that the provided account matches the expected system variable address
/// for type `T`. Provides type-safe access to sysvar-specific functionality for the instruction
/// and slot hashes sysvars.
#[derive(AccountSet, derive_where::DeriveWhere)]
#[derive_where(Clone, Copy, Debug)]
#[account_set(skip_client_account_set)]
#[idl(generics = [])]
#[validate(generics = [])]
pub struct Sysvar<T>
where
    T: SysvarId,
{
    #[single_account_set]
    #[idl(address = T::id())]
    #[validate(address = &T::id())]
    info: AccountView,
    #[account_set(skip = PhantomData)]
    phantom_t: PhantomData<fn() -> T>,
}

#[cfg(not(target_os = "solana"))]
impl<T: SysvarId> crate::account_set::ClientAccountSet for Sysvar<T> {
    type ClientAccounts = Option<Address>;

    const MIN_LEN: usize = 1;

    fn extend_account_metas(
        _program_id: &Address,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta::new_readonly(
            accounts.unwrap_or(T::id()),
            false,
        ));
    }
}

impl Sysvar<InstructionsSysvar> {
    pub fn instructions(
        &self,
    ) -> Result<pinocchio::sysvars::instructions::Instructions<Ref<'_, [u8]>>> {
        Ok(unsafe {
            pinocchio::sysvars::instructions::Instructions::new_unchecked(self.account_data()?)
        })
    }
}

impl Sysvar<SlotHashesSysvar> {
    pub fn slot_hashes(
        &self,
    ) -> Result<pinocchio::sysvars::slot_hashes::SlotHashes<Ref<'_, [u8]>>> {
        Ok(unsafe { SlotHashes::new_unchecked(self.account_data()?) })
    }
}
