//! The runtime while running on Solana.

use crate::prelude::*;
use crate::syscalls::SyscallAccountCache;
use crate::SolanaInstruction;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{
    get_return_data, invoke, invoke_signed_unchecked, invoke_unchecked, set_return_data,
};
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

/// Syscalls provided by the solana runtime.
#[derive(Debug, Clone)]
pub struct SolanaRuntime<'info> {
    /// The program id of the currently executing program.
    pub program_id: Pubkey,
    rent_cache: Option<Rent>,
    clock_cache: Option<Clock>,
    recipient: Option<Mut<AccountInfo<'info>>>,
    funder: Option<Funder<'info>>,
    programs: Vec<AccountInfo<'info>>,
}
impl SolanaRuntime<'_> {
    /// Create a new solana runtime.
    #[must_use]
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            program_id,
            rent_cache: None,
            clock_cache: None,
            recipient: None,
            funder: None,
            programs: Vec::new(),
        }
    }
}
impl SyscallReturn for SolanaRuntime<'_> {
    fn set_return_data(&mut self, data: &[u8]) {
        set_return_data(data);
    }

    fn get_return_data(&self) -> Option<(Pubkey, Vec<u8>)> {
        get_return_data()
    }
}
impl<'info> SyscallInvoke<'info> for SolanaRuntime<'info> {
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
impl<'info> SyscallCore for SolanaRuntime<'info> {
    fn current_program_id(&self) -> &Pubkey {
        &self.program_id
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

impl<'info> SyscallAccountCache<'info> for SolanaRuntime<'info> {
    fn get_funder(&self) -> Option<&Funder<'info>> {
        self.funder.as_ref()
    }

    fn set_funder(&mut self, funder: &(impl SignedAccount<'info> + WritableAccount<'info>)) {
        self.funder.replace(Funder::new(funder));
    }

    fn get_recipient(&self) -> Option<&Mut<AccountInfo<'info>>> {
        self.recipient.as_ref()
    }

    fn set_recipient(&mut self, recipient: &impl WritableAccount<'info>) {
        self.recipient.replace(Mut(recipient.account_info_cloned()));
    }

    fn insert_program<T: StarFrameProgram>(&mut self, program: &Program<'info, T>) {
        if self.programs.iter().any(|p| p.key == program.0.key) {
            return;
        }
        self.programs.push(program.0.clone());
    }

    fn get_program<T: StarFrameProgram>(&self) -> Option<&Program<'info, T>> {
        self.programs
            .iter()
            .find(|p| p.key == &T::PROGRAM_ID)
            .map(|p| {
                // Safety: The program is guaranteed to be the correct type because we only insert
                // programs of the correct type.
                unsafe { &*(p as *const AccountInfo<'_>).cast::<Program<'_, T>>() }
            })
        // let program = self.programs.get(&T::PROGRAM_ID)?;
        // // todo: how bad is this??
        // Some(unsafe { &*(program as *const AccountInfo<'_>).cast::<Program<'_, T>>() })
    }
}
