use crate::sys_calls::{SysCallCore, SysCallInvoke, SysCallReturn};
use crate::util::Network;
use crate::SolanaInstruction;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::{
    get_return_data, invoke, invoke_signed, invoke_signed_unchecked, invoke_unchecked,
    set_return_data,
};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

/// Sys-Calls provided by the solana runtime.
#[derive(Debug)]
pub struct SolanaRuntime<'a> {
    /// The program id of the currently executing program.
    pub program_id: &'a Pubkey,
    pub network: Network,
    rent_cache: Option<Rent>,
    clock_cache: Option<Clock>,
}
impl<'a> SolanaRuntime<'a> {
    /// Create a new solana runtime.
    pub fn new(program_id: &'a Pubkey, network: Network) -> Self {
        Self {
            program_id,
            network,
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
        invoke_signed(instruction, accounts, signers_seeds)
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

    fn current_network(&self) -> &Network {
        &self.network
    }

    fn get_rent(&mut self) -> Result<Rent, ProgramError> {
        match self.rent_cache {
            None => {
                let rent = Rent::get()?;
                self.rent_cache = Some(rent);
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
