use crate::account_set::SingleAccountSet;
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
pub trait SignedAccount: SingleAccountSet {
    /// Gets the seeds of the account if it is seeded.
    fn signer_seeds(&self) -> Option<Vec<&[u8]>>;
}

/// A marker trait that indicates the underlying account is writable.
pub trait WritableAccount: SingleAccountSet {}

/// A marker trait that indicates the underlying type has some inner type in it.
pub trait HasInnerType {
    type Inner: ?Sized + 'static;
}

/// A marker trait that indicates the underlying type is owned by a [`StarFrameProgram`].
pub trait HasOwnerProgram {
    type OwnerProgram: StarFrameProgram;
}

pub type OwnerProgramDiscriminant<T> =
    <<T as HasOwnerProgram>::OwnerProgram as StarFrameProgram>::AccountDiscriminant;

/// A marker trait that indicates the underlying type has seeds in it.
pub trait HasSeeds {
    type Seeds: GetSeeds;
}

/// A trait that allows setting seeds on the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanInitSeeds<A>: SingleAccountSet + AccountSetValidate<A> {
    fn init_seeds(&mut self, arg: &A, syscalls: &impl SyscallInvoke) -> Result<()>;
}

/// A trait that provides logic for the initializing the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanInitAccount<A>: SingleAccountSet {
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: A,
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke,
    ) -> Result<()>;
}
