pub mod solana_runtime;
use crate::SolanaInstruction;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;

/// Trait for syscalls provided by the solana runtime.
pub trait Syscalls: SyscallReturn + SyscallInvoke {}
impl<T> Syscalls for T where T: SyscallReturn + SyscallInvoke {}

/// Return syscalls for a solana program. Allows for simulation.
pub trait SyscallReturn {
    /// Synonym for [`solana_program::program::set_return_data`].
    fn set_return_data(&mut self, data: &[u8]);
    /// Synonym for [`solana_program::program::get_return_data`].
    fn get_return_data(&self) -> Option<(Pubkey, Vec<u8>)>;
}
/// Invoke syscalls for a solana program. Allows for simulation.
pub trait SyscallInvoke: SyscallCore {
    /// Synonym for [`solana_program::program::invoke`].
    fn invoke(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
    ) -> ProgramResult;
    /// Synonym for [`solana_program::program::invoke_unchecked`].
    ///
    /// # Safety
    /// All account info's [`RefCell`](std::cell::RefCell)s must not be borrowed in a way that conflicts with their writable status.
    unsafe fn invoke_unchecked(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
    ) -> ProgramResult;
    /// Synonym for [`solana_program::program::invoke_signed`].
    fn invoke_signed(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult;
    /// Synonym for [`solana_program::program::invoke_signed_unchecked`].
    ///
    /// # Safety
    /// All account info's [`RefCell`](std::cell::RefCell)s must not be borrowed in a way that conflicts with their writable status.
    unsafe fn invoke_signed_unchecked(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult;
}
/// System calls that all syscall implementations must provide.
pub trait SyscallCore {
    /// Get the current program id.
    fn current_program_id(&self) -> &Pubkey;
    /// Get the rent sysvar.
    fn get_rent(&mut self) -> Result<Rent, ProgramError>;
    /// Get the clock.
    fn get_clock(&mut self) -> Result<Clock, ProgramError>;
}
