//! The runtime while running on Solana.

use crate::context::ContextAccountCache;
use crate::prelude::*;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::{clock::Clock, Sysvar};
use std::cell::Cell;

/// Syscalls provided by the solana runtime.
#[derive(derive_more::Debug)]
pub struct SolanaRuntime {
    /// The program id of the currently executing program.
    pub program_id: Pubkey,
    rent_cache: Cell<Option<Rent>>,
    clock_cache: Cell<Option<Clock>>,
    #[debug("{:?}", recipient.as_ref().map(|r| std::any::type_name_of_val(r)))]
    recipient: Option<Box<dyn CanReceiveRent>>,
    #[debug("{:?}", funder.as_ref().map(|f| std::any::type_name_of_val(f)))]
    funder: Option<Box<dyn CanFundRent>>,
}

impl SolanaRuntime {
    /// Create a new solana runtime.
    #[must_use]
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            program_id,
            rent_cache: Cell::new(None),
            clock_cache: Cell::new(None),
            recipient: None,
            funder: None,
        }
    }
}

impl ContextCore for SolanaRuntime {
    fn current_program_id(&self) -> &Pubkey {
        &self.program_id
    }

    fn get_rent(&self) -> Result<Rent> {
        match self.rent_cache.get() {
            None => {
                let new_rent = Rent::get()?;
                self.rent_cache.set(Some(new_rent));
                Ok(new_rent)
            }
            Some(rent) => Ok(rent),
        }
    }

    fn get_clock(&self) -> Result<Clock> {
        match self.clock_cache.get() {
            None => {
                let new_clock = Clock::get()?;
                self.clock_cache.set(Some(new_clock));
                Ok(new_clock)
            }
            Some(clock) => Ok(clock),
        }
    }
}

impl ContextAccountCache for SolanaRuntime {
    fn get_funder(&self) -> Option<&dyn CanFundRent> {
        self.funder.as_ref().map(std::convert::AsRef::as_ref)
    }

    fn set_funder(&mut self, funder: Box<dyn CanFundRent>) {
        self.funder.replace(funder);
    }

    fn get_recipient(&self) -> Option<&dyn CanReceiveRent> {
        self.recipient.as_ref().map(std::convert::AsRef::as_ref)
    }

    fn set_recipient(&mut self, recipient: Box<dyn CanReceiveRent>) {
        self.recipient.replace(recipient);
    }
}
