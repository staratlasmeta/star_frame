pub mod solana_runtime;
use crate::prelude::{CanFundRent, CanReceiveRent};
use crate::Result;
use pinocchio::account_info::AccountInfo;
use pinocchio::cpi::ReturnData;
use pinocchio::sysvars::clock::Clock;
use pinocchio::sysvars::rent::Rent;
use solana_instruction::Instruction as SolanaInstruction;
use solana_pubkey::Pubkey;

/// Trait for syscalls provided by the solana runtime.
pub trait Syscalls: SyscallReturn + SyscallInvoke + SyscallAccountCache {}
impl<T> Syscalls for T where T: SyscallReturn + SyscallInvoke {}

/// Return syscalls for a solana program. Allows for simulation.
pub trait SyscallReturn {
    /// Synonym for [`pinocchio::program::set_return_data`].
    fn set_return_data(&self, data: &[u8]);
    /// Synonym for [`pinocchio::program::get_return_data`].
    fn get_return_data(&self) -> Option<ReturnData>;
}
/// Invoke syscalls for a solana program. Allows for simulation.
pub trait SyscallInvoke: SyscallCore + SyscallAccountCache {
    /// Synonym for [`pinocchio::program::invoke`].
    fn invoke(&self, instruction: &SolanaInstruction, accounts: &[AccountInfo]) -> Result<()>;
    /// Synonym for [`pinocchio::program::invoke_signed`].
    fn invoke_signed(
        &self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> Result<()>;
}

/// A trait for caching commonly used accounts in the Syscall. This allows [`crate::account_set::AccountSetValidate`]
/// implementations to pull from this cache instead of requiring the user to explicitly pass in the accounts.
pub trait SyscallAccountCache {
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

/// System calls that all syscall implementations must provide.
pub trait SyscallCore {
    /// Get the current program id.
    fn current_program_id(&self) -> &Pubkey;
    /// Get the rent sysvar.
    fn get_rent(&self) -> Result<Rent>;
    /// Get the clock.
    fn get_clock(&self) -> Result<Clock>;
}
