//! Data section of an account data.

use std::ops::{Deref, DerefMut};
use std::ptr;

/// Data section of an account data.
#[repr(C)]
#[derive(Debug)]
pub struct AccountDataSection {
    /// Length of the data section.
    pub length: u64,
    /// Data section.
    pub data: [u8],
}

impl AccountDataSection {
    /// # Safety
    /// Caller must ensure that the length is <= `original_data_length` +
    /// [`MAX_PERMITTED_DATA_INCREASE`](solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE)
    /// and <= [`MAX_PERMITTED_DATA_LENGTH`](solana_program::system_instruction::MAX_PERMITTED_DATA_LENGTH).
    pub unsafe fn set_length(this: &mut &mut Self, length: usize) {
        this.length = length as u64;
        *this = &mut *ptr::from_raw_parts_mut((*this as *mut Self).cast(), length);
    }
}

impl Deref for AccountDataSection {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for AccountDataSection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
