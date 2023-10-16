//! Conversion between usize and numeric types.

use crate::{Result, UtilError};
use anchor_lang::error;
use num_traits::ToPrimitive;

/// Value can be converted to and from a usize.
pub trait ToFromUsize {
    /// Converts the value to a usize.
    fn to_usize(self) -> Result<usize>;
    /// Converts a usize to the value.
    fn from_usize(value: usize) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! impl_to_from_usize {
    (inner $ty:ty, $to_self:ident) => {
        impl ToFromUsize for $ty {
            fn to_usize(self) -> Result<usize> {
                Ok(ToPrimitive::to_usize(&self).ok_or_else(|| UtilError::NumericOverflow)?)
            }

            fn from_usize(value: usize) -> Result<Self> {
                value
                    .$to_self()
                    .ok_or_else(|| error!(UtilError::NumericOverflow).into())
            }
        }
    };
    ($([$ty:ty, $to_self:ident $(,)?]),* $(,)?) => {
        $(impl_to_from_usize!(inner $ty, $to_self);)*
    }
}
impl_to_from_usize!(
    [u8, to_u8],
    [u16, to_u16],
    [u32, to_u32],
    [u64, to_u64],
    [u128, to_u128],
);
