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
    programs: Vec<AccountInfo<'info>>,
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
            programs: vec![],
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
    fn invoke(&self, instruction: &SolanaInstruction, accounts: &[AccountInfo]) -> ProgramResult {
        invoke(instruction, accounts)
    }

    unsafe fn invoke_unchecked(
        &self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        invoke_unchecked(instruction, accounts)
    }

    fn invoke_signed(
        &self,
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
        &self,
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

    fn get_rent(&self) -> std::result::Result<Rent, ProgramError> {
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

    fn get_clock(&self) -> std::result::Result<Clock, ProgramError> {
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
                // Safety: Because Program is transparent over AccountInfo, we can safely cast references of AccountInfo to Programt. {}
                // Casting with a specific `T` is fine because it's only in phantom data. Because `T::PROGRAM_ID`
                // should match the key, this should be fine from a correctness perspective.
                unsafe { &*std::ptr::from_ref::<AccountInfo<'_>>(p).cast::<Program<'_, T>>() }
            })
    }
}
