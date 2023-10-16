//! Errors for `cargo`

use common_utils::prelude::*;
use common_utils::BoxedAnchorError;

/// Errors for `cargo`
// // Would love to do this but not supported by anchor...
// #[error_code(offset = anchor_lang::error::ERROR_CODE_OFFSET + 1000)]
#[error_code(offset = 7000)]
pub enum UtilError {
    /// Generic placeholder error. Should be replaced with a better error.
    #[msg("Generic placeholder error. Should be replaced with a better error.")]
    GenericError,
    /// System program is missing when normalizing rent
    #[msg("System program is missing when normalizing rent")]
    MissingSystemProgram,
    /// Rent funder is not valid for normalizing rent
    #[msg("Rent funder is not valid for normalizing rent")]
    InvalidRentFunder,
    /// Not enough data left
    #[msg("Not enough data left")]
    NotEnoughData,
    /// Popped too many items
    #[msg("Popped too many items")]
    TooManyPopped,
    /// Numeric overflow
    #[msg("Numeric overflow")]
    NumericOverflow,
    /// Invalid enum discriminant
    #[msg("Invalid enum discriminant")]
    InvalidEnumDiscriminant,
    /// Cannot find bump
    #[msg("Cannot find bump")]
    NoBump,
    /// Flags are invalid
    #[msg("Flags are invalid")]
    InvalidFlags,
    /// Index out of bounds
    #[msg("Index out of bounds.")]
    IndexOutOfBounds,
    /// An expected remaining account is missing
    #[msg("An expected remaining account is missing.")]
    MissingRemainingAccount,
    /// The key provided does not match the expected key
    #[msg("The key provided does not match the expected key")]
    InvalidKey,
    /// Invalid pointer
    #[msg("Invalid pointer")]
    InvalidPointer,
    /// Error reallocating
    #[msg("Error reallocating")]
    ReallocError,
}
impl From<UtilError> for BoxedAnchorError {
    fn from(e: UtilError) -> Self {
        anchor_lang::prelude::Error::from(e).into()
    }
}
