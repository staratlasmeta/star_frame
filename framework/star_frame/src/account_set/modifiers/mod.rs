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

/// A marker trait that indicates the underlying type is owned by a [`StarFrameProgram`].
pub trait HasOwnerProgram {
    type OwnerProgram: StarFrameProgram;
}

/// A marker trait that indicates the underlying type has seeds in it.
pub trait HasSeeds {
    type Seeds: GetSeeds;
}

/// A trait that allows setting seeds on the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanInitSeeds<'info, A>: SingleAccountSet<'info> + AccountSetValidate<'info, A> {
    fn init_seeds(&mut self, arg: &A, syscalls: &mut impl SyscallInvoke<'info>) -> Result<()>;
}

/// A trait that provides logic for the initializing the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanInitAccount<'info, A>: SingleAccountSet<'info> {
    fn init_account(
        &mut self,
        arg: A,
        syscalls: &mut impl SyscallInvoke<'info>,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()>;
}
