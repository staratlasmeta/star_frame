use crate::Result;
use anyhow::anyhow;
use solana_program::program_error::ProgramError;
pub use star_frame_proc::UnitEnumFromRepr;

/// Trait for getting a unit enum value from its repr.
pub trait UnitEnumFromRepr: Copy {
    /// The repr of the enum.
    type Repr;
    /// Gets the enum value from its repr.
    fn from_repr(repr: Self::Repr) -> std::result::Result<Self, Self::Repr>;
    /// Gets the enum value from its repr, or returns an error.
    fn from_repr_or_error(repr: Self::Repr) -> Result<Self> {
        // TODO: Better Error
        Self::from_repr(repr).map_err(|_| anyhow!(ProgramError::InvalidAccountData))
    }
    /// Gets the repr of the enum value.
    fn into_repr(self) -> Self::Repr;
}
