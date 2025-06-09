pub mod solana_runtime;
use crate::prelude::{CanFundRent, CanReceiveRent};
use crate::Result;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::rent::Rent;
use solana_pubkey::Pubkey;

/// Trait for context provided by the solana runtime.
pub trait Context: ContextCore + ContextAccountCache {}
impl<T> Context for T where T: ContextCore + ContextAccountCache {}

/// A trait for caching commonly used accounts in the Context. This allows [`crate::account_set::AccountSetValidate`]
/// implementations to pull from this cache instead of requiring the user to explicitly pass in the accounts.
pub trait ContextAccountCache {
    /// Gets a cached version of the funder if exists and Self has a funder cache
    fn get_funder(&self) -> Option<&dyn CanFundRent> {
        None
    }
    /// Sets the funder cache if Self has one. No-op if it doesn't.
    fn set_funder(&mut self, _funder: Box<dyn CanFundRent>) {}
    /// Gets a cached version of the recipient if exists and Self has a recipient cache
    fn get_recipient(&self) -> Option<&dyn CanReceiveRent> {
        None
    }
    /// Sets the recipient cache if Self has one. No-op if it doesn't.
    fn set_recipient(&mut self, _recipient: Box<dyn CanReceiveRent>) {}
}

/// System calls that all context implementations must provide.
pub trait ContextCore {
    /// Get the current program id.
    fn current_program_id(&self) -> &Pubkey;
    /// Get the rent sysvar.
    fn get_rent(&self) -> Result<Rent>;
    /// Get the clock.
    fn get_clock(&self) -> Result<Clock>;
}
