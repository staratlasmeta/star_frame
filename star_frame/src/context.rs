use crate::prelude::*;
use pinocchio::sysvars::{clock::Clock, rent::Rent, Sysvar};
use std::{cell::Cell, collections::BTreeMap};

/// Additional context given to [`crate::instruction::Instruction`]s, enabling programs to cache and retrieve helpful information during instruction execution.
#[derive(Debug, Default)]
pub struct Context {
    /// The program id of the currently executing program.
    program_id: Pubkey,
    // Rent cache to avoid repeated `Rent::get()` calls
    rent_cache: Cell<Option<Rent>>,
    // Clock cache to avoid repeated `Clock::get()` calls
    clock_cache: Cell<Option<Clock>>,
    // Cached recipient for rent. Usually set during `AccountSetValidate`
    recipient: Option<Box<dyn CanAddLamports>>,
    // Cached funder for rent. Usually set during `AccountSetValidate`
    funder: Option<Box<dyn CanFundRent>>,
    program_cache: BTreeMap<Pubkey, AccountInfo>,
}

impl Context {
    /// Create a new context from a program id.
    #[must_use]
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            program_id,
            rent_cache: Cell::new(None),
            clock_cache: Cell::new(None),
            recipient: None,
            funder: None,
            program_cache: BTreeMap::new(),
        }
    }

    /// Get the program id of the currently executing program.
    pub fn current_program_id(&self) -> &Pubkey {
        &self.program_id
    }

    /// Gets the rent sysvar from the cache, populating the cache with a call to `Rent::get()` if empty.
    pub fn get_rent(&self) -> Result<Rent> {
        match self.rent_cache.get() {
            None => {
                let new_rent = Rent::get()?;
                self.rent_cache.set(Some(new_rent));
                Ok(new_rent)
            }
            Some(rent) => Ok(rent),
        }
    }

    /// Gets the clock sysvar from the cache, populating the cache with a call to `Clock::get()` if empty.
    pub fn get_clock(&self) -> Result<Clock> {
        match self.clock_cache.get() {
            None => {
                let new_clock = Clock::get()?;
                self.clock_cache.set(Some(new_clock));
                Ok(new_clock)
            }
            Some(clock) => Ok(clock),
        }
    }

    /// Gets the cached funder for rent if it has been set.
    pub fn get_funder(&self) -> Option<&dyn CanFundRent> {
        self.funder.as_ref().map(std::convert::AsRef::as_ref)
    }

    /// Sets the funder for rent.
    pub fn set_funder(&mut self, funder: Box<dyn CanFundRent>) {
        self.funder.replace(funder);
    }

    /// Gets the cached recipient for rent if it has been set.
    pub fn get_recipient(&self) -> Option<&dyn CanAddLamports> {
        self.recipient.as_ref().map(std::convert::AsRef::as_ref)
    }

    /// Sets the recipient for rent.
    pub fn set_recipient(&mut self, recipient: Box<dyn CanAddLamports>) {
        self.recipient.replace(recipient);
    }

    pub fn add_program(&mut self, key: Pubkey, info: AccountInfo) {
        self.program_cache.insert(key, info);
    }

    pub fn program_for_key(&self, key: &Pubkey) -> Option<&AccountInfo> {
        self.program_cache.get(key)
    }
}
