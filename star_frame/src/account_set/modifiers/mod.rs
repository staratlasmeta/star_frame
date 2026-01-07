//! Account Sets that wrap and modify other Account Sets.

use crate::{account_set::AccountSetValidate, prelude::*};

pub mod init;
pub mod mutable;
pub mod seeded;
pub mod signer;

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

/// Shorthand for getting the [`StarFrameProgram::AccountDiscriminant`] type from a [`HasOwnerProgram`] type.
pub type OwnerProgramDiscriminant<T> =
    <<T as HasOwnerProgram>::OwnerProgram as StarFrameProgram>::AccountDiscriminant;

/// A marker trait that indicates the underlying type has seeds in it.
pub trait HasSeeds {
    type Seeds: GetSeeds;
}

/// A trait that allows setting seeds on the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanInitSeeds<A>: SingleAccountSet + AccountSetValidate<A> {
    #[rust_analyzer::completions(ignore_flyimport)]
    fn init_seeds(&mut self, arg: &A, ctx: &Context) -> Result<()>;
}

/// A trait that provides logic for the initializing the underlying account. This helps enable the [`Init`] and [`Seeded`] modifiers.
pub trait CanInitAccount<A>: SingleAccountSet {
    /// Returns whether the account was just initialized (if it was already initialized, returns `false`).
    ///
    /// If `IF_NEEDED` is `false`, initialization is always attempted and may error if the account is already initialized.
    /// If `IF_NEEDED` is `true`, initialization is skipped (returning `false`) if the account is already initialized.
    #[rust_analyzer::completions(ignore_flyimport)]
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: A,
        account_seeds: Option<&[&[u8]]>,
        ctx: &Context,
    ) -> Result<bool>;
}
