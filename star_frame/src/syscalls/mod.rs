pub mod solana_runtime;
use crate::prelude::{CanFundRent, CanReceiveRent};
use crate::{Result, SolanaInstruction};
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;

/// Trait for syscalls provided by the solana runtime.
pub trait Syscalls<'info>:
    SyscallReturn + SyscallInvoke<'info> + SyscallAccountCache<'info>
{
}
impl<'info, T> Syscalls<'info> for T where T: SyscallReturn + SyscallInvoke<'info> {}

/// Return syscalls for a solana program. Allows for simulation.
pub trait SyscallReturn {
    /// Synonym for [`solana_program::program::set_return_data`].
    fn set_return_data(&self, data: &[u8]);
    /// Synonym for [`solana_program::program::get_return_data`].
    fn get_return_data(&self) -> Option<(Pubkey, Vec<u8>)>;
}
/// Invoke syscalls for a solana program. Allows for simulation.
pub trait SyscallInvoke<'info>: SyscallCore + SyscallAccountCache<'info> {
    /// Synonym for [`solana_program::program::invoke`].
    fn invoke(&self, instruction: &SolanaInstruction, accounts: &[AccountInfo]) -> Result<()>;
    /// Synonym for [`solana_program::program::invoke_signed`].
    fn invoke_signed(
        &self,
        instruction: &SolanaInstruction,
        accounts: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> Result<()>;
}

/// A trait for caching commonly used accounts in the Syscall. This allows [`crate::account_set::AccountSetValidate`]
/// implementations to pull from this cache instead of requiring the user to explicitly pass in the accounts.
pub trait SyscallAccountCache<'info> {
    /// Gets a cached version of the funder if exists and Self has a funder cache
    fn get_funder(&self) -> Option<&dyn CanFundRent<'info>> {
        None
    }
    /// Sets the funder cache if Self has one. No-op if it doesn't.
    fn set_funder(&mut self, _funder: Box<dyn CanFundRent<'info> + 'info>) {}
    /// Gets a cached version of the funder if exists and Self has a funder cache
    fn get_recipient(&self) -> Option<&dyn CanReceiveRent<'info>> {
        None
    }
    /// Sets the recipient cache if Self has one. No-op if it doesn't.
    fn set_recipient(&mut self, _recipient: Box<dyn CanReceiveRent<'info> + 'info>) {}
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
