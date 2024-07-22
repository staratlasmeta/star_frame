//! The runtime while running on Solana.

use crate::prelude::*;
use crate::SolanaInstruction;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{
    get_return_data, invoke, invoke_signed_unchecked, invoke_unchecked, set_return_data,
};
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

/// Sys-Calls provided by the solana runtime.
#[derive(Debug)]
pub struct SolanaRuntime<'a> {
    /// The program id of the currently executing program.
    pub program_id: &'a Pubkey,
    rent_cache: Option<Rent>,
    clock_cache: Option<Clock>,
}
impl<'a> SolanaRuntime<'a> {
    /// Create a new solana runtime.
    #[must_use]
    pub fn new(program_id: &'a Pubkey) -> Self {
        Self {
            program_id,
            rent_cache: None,
            clock_cache: None,
        }
    }
}
impl<'a> SysCallReturn for SolanaRuntime<'a> {
    fn set_return_data(&mut self, data: &[u8]) {
        set_return_data(data);
    }

    fn get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        get_return_data()
    }
}
impl<'b> SysCallInvoke for SolanaRuntime<'b> {
    fn invoke(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        invoke(instruction, accounts)
    }

    unsafe fn invoke_unchecked(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        invoke_unchecked(instruction, accounts)
    }

    fn invoke_signed(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        // Check that the account RefCells are consistent with the request
        for account_meta in &instruction.accounts {
            for account_info in accounts {
                if account_meta.pubkey == *account_info.key {
                    if account_meta.is_writable {
                        let _ = account_info.try_borrow_mut_lamports().map_err(|e| {
                            msg!("lamports borrow_mut failed for {}", account_info.key);
                            e
                        })?;
                        let _ = account_info.try_borrow_mut_data().map_err(|e| {
                            msg!("data borrow_mut failed for {}", account_info.key);
                            e
                        })?;
                    } else {
                        let _ = account_info.try_borrow_lamports().map_err(|e| {
                            msg!("lamports borrow failed for {}", account_info.key);
                            e
                        })?;
                        let _ = account_info.try_borrow_data().map_err(|e| {
                            msg!("data borrow failed for {}", account_info.key);
                            e
                        })?;
                    }
                    break;
                }
            }
        }

        // Safety: check logic for solana's invoke_signed is duplicated, with the only difference
        // being the problematic pubkeys are now logged out on error.
        invoke_signed_unchecked(instruction, accounts, signers_seeds)
    }

    unsafe fn invoke_signed_unchecked(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        invoke_signed_unchecked(instruction, accounts, signers_seeds)
    }
}
impl<'a> SysCallCore for SolanaRuntime<'a> {
    fn current_program_id(&self) -> &Pubkey {
        self.program_id
    }

    fn get_rent(&mut self) -> Result<Rent, ProgramError> {
        match self.rent_cache.clone() {
            None => {
                let rent = Rent::get()?;
                self.rent_cache = Some(rent.clone());
                Ok(rent)
            }
            Some(rent) => Ok(rent),
        }
    }

    fn get_clock(&mut self) -> Result<Clock, ProgramError> {
        match self.clock_cache.clone() {
            None => {
                let clock = Clock::get()?;
                self.clock_cache = Some(clock.clone());
                Ok(clock)
            }
            Some(clock) => Ok(clock),
        }
    }
}
