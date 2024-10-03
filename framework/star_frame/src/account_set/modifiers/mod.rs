use crate::account_set::{ProgramAccount, SingleAccountSet};
use crate::prelude::*;

mod init;
mod mutable;
mod seeded;
mod signer;

pub use init::*;
pub use mutable::*;
pub use seeded::*;
pub use signer::*;

// TODO: Add macros to make propagating the marker traits easier.

/// A marker trait that indicates the underlying account is a signer
pub trait SignedAccount<'info>: SingleAccountSet<'info> {
    /// Gets the seeds of the account if it is seeded.
    fn signer_seeds(&self) -> Option<Vec<&[u8]>>;
}

/// A marker trait that indicates the underlying account is writable.
pub trait WritableAccount<'info>: SingleAccountSet<'info> {}

/// A marker trait that indicates the underlying type has a [`ProgramAccount`] in it.
pub trait HasProgramAccount {
    type ProgramAccount: ProgramAccount + ?Sized;
}

/// A marker trait that indicates the underlying type has seeds in it.
pub trait HasSeeds {
    type Seeds: GetSeeds;
}

/// A trait that allows setting seeds on the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanSetSeeds<'info, A>: SingleAccountSet<'info> {
    fn set_seeds(&mut self, arg: &A, syscalls: &mut impl SyscallInvoke) -> Result<()>;
}

/// A trait that allows initializing the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanInitAccount<'info, A>: SingleAccountSet<'info> {
    fn init(
        &mut self,
        arg: A,
        syscalls: &mut impl SyscallInvoke,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()>;
}
