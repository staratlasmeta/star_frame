use crate::{DataSize, PackedValue};
use anchor_lang::ZeroCopy;
use bytemuck::Pod;
use common_utils::prelude::*;
use core::fmt::Debug;

/// A safely implemented zero copy account. Also has an alignment of 1.
///
/// # Safety
/// Should only be implemented with the [`safe_zero_copy_account`](crate::safe_zero_copy_account) macro.
pub unsafe trait SafeZeroCopyAccount: ZeroCopy + Owner + SafeZeroCopy + Debug {}

/// A safely implemented zero copy struct. Also has an alignment of 1.
///
/// # Safety
/// Should only be implemented with the [`safe_zero_copy`](macro@crate::safe_zero_copy) or [`safe_zero_copy_account`](crate::safe_zero_copy_account) macro.
/// Manual implementation requires that the struct is a valid [`Pod`] implementation and has an alignment of `1`.
pub unsafe trait SafeZeroCopy: Pod + DataSize + Debug {}

// Safety: Meets all the requirements of a safe zero copy struct.
unsafe impl SafeZeroCopy for () {}
// Safety: Meets all the requirements of a safe zero copy struct.
unsafe impl SafeZeroCopy for u8 {}
// Safety: Meets all the requirements of a safe zero copy struct.
unsafe impl SafeZeroCopy for i8 {}
// Safety: Meets all the requirements of a safe zero copy struct.
unsafe impl SafeZeroCopy for Pubkey {}
// Safety: Meets all the requirements of a safe zero copy struct.
unsafe impl<T> SafeZeroCopy for PackedValue<T> where T: Pod + DataSize + Debug {}

// Safety: Arrays have the same alignment as their element.
unsafe impl<T, const N: usize> SafeZeroCopy for [T; N] where T: SafeZeroCopy + Pod {}
