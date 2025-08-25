//! Alignment to 1 byte. Much of the [`crate::unsize`] magic relies on packed alignment and no padding.

use solana_pubkey::Pubkey;
pub use star_frame_proc::Align1;
use std::{
    marker::PhantomData,
    num::{NonZeroI8, NonZeroU8},
};

/// A marker trait for types that are guaranteed to be aligned to 1 byte. The [unsized type system](crate::unsize) relies on `Align1` types for its pointer manipulation.
///
/// # Safety
/// This trait should only be implemented for types that are guaranteed to be aligned to 1 byte.
/// The [`Align1`](star_frame_proc::Align1) macro can be used to safely implement this trait for non-generic types.
pub unsafe trait Align1 {}

macro_rules! impl_align1 {
    ($($name:ty),*) => {
        $(
            // SAFETY:
            // Allowed due to the lower assert.
            unsafe impl $crate::align1::Align1 for $name {}
            $crate::static_assertions::assert_eq_align!($name, u8);
        )*
    };
}

impl_align1!((), u8, i8, bool, Pubkey, NonZeroU8, NonZeroI8);

// SAFETY:
// Allowed because `PhantomData` is a ZST.
unsafe impl<T: ?Sized> Align1 for PhantomData<T> {}
// SAFETY:
// Allowed because a slice of `T` is aligned to `T`.
unsafe impl<T> Align1 for [T] where T: Align1 {}
// SAFETY:
// Allowed because an array of `T` is aligned to `T`.
unsafe impl<T, const N: usize> Align1 for [T; N] where T: Align1 {}
