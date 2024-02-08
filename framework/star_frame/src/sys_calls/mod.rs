#[cfg(any(target_os = "solana", feature = "fake_solana_os"))]
pub mod solana_runtime;

use crate::util::Network;
use crate::SolanaInstruction;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;

/// Trait for sys-calls provided by the solana runtime.
pub trait SysCalls: SysCallReturn + SysCallInvoke {}
impl<T> SysCalls for T where T: SysCallReturn + SysCallInvoke {}

/// Return sys-calls for a solana program. Allows for simulation.
pub trait SysCallReturn {
    /// Synonym for [`set_return_data`].
    fn set_return_data(&mut self, data: &[u8]);
    /// Synonym for [`get_return_data`].
    fn get_return_data(&self) -> Option<(Pubkey, Vec<u8>)>;
}
/// Invoke sys-calls for a solana program. Allows for simulation.
pub trait SysCallInvoke: SysCallCore {
    /// Synonym for [`invoke`].
    fn invoke(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
    ) -> ProgramResult;
    /// Synonym for [`invoke_unchecked`].
    ///
    /// # Safety
    /// All account info's [`RefCell`](std::cell::RefCell)s must not be borrowed in a way that conflicts with their writable status.
    unsafe fn invoke_unchecked(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
    ) -> ProgramResult;
    /// Synonym for [`invoke_signed`].
    fn invoke_signed(
        &mut self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult;
    /// Synonym for [`invoke_signed_unchecked`].
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
/// System calls that all sys-call implementations must provide.
pub trait SysCallCore {
    /// Get the current program id.
    fn current_program_id(&self) -> &Pubkey;
    /// Gets the current network.
    /// Not determined by solana runtime but by build config.
    fn current_network(&self) -> &Network;
    /// Get the rent sysvar.
    fn get_rent(&mut self) -> Result<Rent, ProgramError>;
    /// Get the clock.
    fn get_clock(&mut self) -> Result<Clock, ProgramError>;
}
