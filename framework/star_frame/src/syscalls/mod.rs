pub mod solana_runtime;
use crate::account_set::{Funder, Mut, Program, SignedAccount, WritableAccount};
use crate::program::StarFrameProgram;
use crate::SolanaInstruction;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program::program_error::ProgramError;
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
    fn set_return_data(&mut self, data: &[u8]);
    /// Synonym for [`solana_program::program::get_return_data`].
    fn get_return_data(&self) -> Option<(Pubkey, Vec<u8>)>;
}
/// Invoke syscalls for a solana program. Allows for simulation.
pub trait SyscallInvoke<'info>: SyscallCore + SyscallAccountCache<'info> {
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

/// A trait for caching commonly used accounts in the Syscall. This allows [`crate::account_set::AccountSetValidate`]
/// implementations to pull from this cache instead of requiring the user to explicitly pass in the accounts.
pub trait SyscallAccountCache<'info> {
    /// Gets a cached version of the funder if exists and Self has a funder cache
    fn get_funder(&self) -> Option<&Funder<'info>> {
        None
    }
    /// Sets the funder cache if Self has one. No-op if it doesn't.
    fn set_funder(&mut self, _funder: &(impl SignedAccount<'info> + WritableAccount<'info>)) {}
    /// Gets a cached version of the funder if exists and Self has a funder cache
    fn get_recipient(&self) -> Option<&Mut<AccountInfo<'info>>> {
        None
    }
    /// Sets the recipient cache if Self has one. No-op if it doesn't.
    fn set_recipient(&mut self, _recipient: &impl WritableAccount<'info>) {}

    /// Inserts a program into the program cache if it doesn't already exist.
    fn insert_program<T: StarFrameProgram>(&mut self, _program: &Program<'info, T>) {}

    /// Gets the program from the cache if it exists.
    fn get_program<T: StarFrameProgram>(&self) -> Option<&Program<'info, T>> {
        None
    }
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
