//! The runtime while running on Solana.

use crate::prelude::*;
use crate::syscalls::SyscallAccountCache;
use crate::SolanaInstruction;
use solana_program::clock::Clock;
use solana_program::program::{get_return_data, invoke_signed_unchecked, set_return_data};
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use std::cell::RefCell;

/// Syscalls provided by the solana runtime.
#[derive(Debug, Clone)]
pub struct SolanaRuntime<'info> {
    /// The program id of the currently executing program.
    pub program_id: Pubkey,
    rent_cache: RefCell<Option<Rent>>,
    clock_cache: RefCell<Option<Clock>>,
    recipient: Option<Mut<AccountInfo<'info>>>,
    funder: Option<Mut<SignerInfo<'info>>>,
}
impl SolanaRuntime<'_> {
    /// Create a new solana runtime.
    #[must_use]
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            program_id,
            rent_cache: RefCell::new(None),
            clock_cache: RefCell::new(None),
            recipient: None,
            funder: None,
        }
    }
}
impl SyscallReturn for SolanaRuntime<'_> {
    fn set_return_data(&self, data: &[u8]) {
        set_return_data(data);
    }

    fn get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        get_return_data()
    }
}
impl<'info> SyscallInvoke<'info> for SolanaRuntime<'info> {
    fn invoke(&self, instruction: &SolanaInstruction, accounts: &[AccountInfo]) -> Result<()> {
        self.invoke_signed(instruction, accounts, &[])
    }

    fn invoke_signed(
        &self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        // Check that the account RefCells are consistent with the request
        for account_meta in &instruction.accounts {
            for account_info in accounts {
                if account_meta.pubkey == *account_info.key {
                    if account_meta.is_writable {
                        let _ = account_info
                            .account_info()
                            .lamports
                            .try_borrow_mut()
                            .map_err(|_| {
                                msg!("lamports borrow failed for {}", account_info.key);
                                ProgramError::AccountBorrowFailed
                            })?;
                        let _ = account_info.info_data_bytes_mut()?;
                    } else {
                        let _ =
                            account_info
                                .account_info()
                                .lamports
                                .try_borrow()
                                .map_err(|_| {
                                    msg!("lamports borrow failed for {}", account_info.key);
                                    ProgramError::AccountBorrowFailed
                                })?;
                        let _ = account_info.info_data_bytes()?;
                    }
                    break;
                }
            }
        }

        // Safety: check logic for solana's invoke_signed is duplicated, with the only difference
        // being the problematic pubkeys are now logged out on error.
        invoke_signed_unchecked(instruction, accounts, signers_seeds)?;
        Ok(())
    }
}
impl<'info> SyscallCore for SolanaRuntime<'info> {
    fn current_program_id(&self) -> &Pubkey {
        &self.program_id
    }

    fn get_rent(&self) -> Result<Rent> {
        let mut rent = self.rent_cache.borrow_mut();
        #[allow(clippy::clone_on_copy)]
        match &*rent {
            None => {
                let new_rent = Rent::get()?;
                *rent = Some(new_rent.clone());
                Ok(new_rent)
            }
            Some(rent) => Ok(rent.clone()),
        }
    }

    fn get_clock(&self) -> Result<Clock> {
        let mut clock = self.clock_cache.borrow_mut();
        match &*clock {
            None => {
                let new_clock = Clock::get()?;
                *clock = Some(new_clock.clone());
                Ok(new_clock)
            }
            Some(clock) => Ok(clock.clone()),
        }
    }
}

impl<'info> SyscallAccountCache<'info> for SolanaRuntime<'info> {
    fn get_funder(&self) -> Option<&Mut<SignerInfo<'info>>> {
        self.funder.as_ref()
    }

    fn set_funder(&mut self, funder: &(impl SignedAccount<'info> + WritableAccount<'info>)) {
        self.funder
            .replace(Mut(Signer(funder.account_info_cloned())));
    }

    fn get_recipient(&self) -> Option<&Mut<AccountInfo<'info>>> {
        self.recipient.as_ref()
    }

    fn set_recipient(&mut self, recipient: &impl WritableAccount<'info>) {
        self.recipient.replace(Mut(recipient.account_info_cloned()));
    }
}
