//! The runtime while running on Solana.

use crate::prelude::*;
use crate::syscalls::SyscallAccountCache;
use crate::SolanaInstruction;
use itertools::Itertools;
use pinocchio::cpi::{slice_invoke_signed, ReturnData};
use pinocchio::instruction::{
    AccountMeta as PinocchioAccountMeta, Instruction as PinocchioInstruction,
    Seed as PinocchioSeed, Signer as PinocchioSigner,
};
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::{clock::Clock, Sysvar};
use pinocchio::ProgramResult;
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
impl SyscallReturn for SolanaRuntime {
    fn set_return_data(&self, data: &[u8]) {
        pinocchio::cpi::set_return_data(data);
    }

    fn get_return_data(&self) -> Option<ReturnData> {
        pinocchio::cpi::get_return_data()
    }
}

#[inline(never)]
fn invoke_signed_never_inline(
    instruction: &PinocchioInstruction,
    accounts: &[&AccountInfo],
    signers: &[PinocchioSigner],
) -> ProgramResult {
    slice_invoke_signed(instruction, accounts, signers)
}

#[inline]
fn convert_account_metas(instruction: &SolanaInstruction) -> Vec<PinocchioAccountMeta> {
    instruction
        .accounts
        .iter()
        .map(|meta| PinocchioAccountMeta {
            pubkey: meta.pubkey.as_array(),
            is_writable: meta.is_writable,
            is_signer: meta.is_signer,
        })
        .collect_vec()
}

#[inline]
fn convert_instruction<'a>(
    instruction: &'a SolanaInstruction,
    metas: &'a [PinocchioAccountMeta<'a>],
) -> PinocchioInstruction<'a, 'a, 'a, 'a> {
    PinocchioInstruction {
        program_id: instruction.program_id.as_array(),
        data: instruction.data.as_slice(),
        accounts: metas,
    }
}

impl SyscallInvoke for SolanaRuntime {
    fn invoke(&self, instruction: &SolanaInstruction, accounts: &[AccountInfo]) -> Result<()> {
        self.invoke_signed(instruction, accounts, &[])
    }

    fn invoke_signed(
        &self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let metas = convert_account_metas(instruction);
        let pinocchio_ix = convert_instruction(instruction, &metas);
        let accounts = accounts.iter().collect_vec();

        let nested_seeds: Vec<Vec<PinocchioSeed>> = signers_seeds
            .iter()
            .map(|seeds| {
                seeds
                    .iter()
                    .map(|seed| PinocchioSeed::from(*seed))
                    .collect_vec()
            })
            .collect_vec();
        let signers = nested_seeds
            .iter()
            .map(|seeds| seeds.as_slice().into())
            .collect_vec();

        // TODO: Make this log the errors better
        invoke_signed_never_inline(&pinocchio_ix, &accounts, &signers)?;
        Ok(())
    }
}
impl SyscallCore for SolanaRuntime {
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

impl SyscallAccountCache for SolanaRuntime {
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
