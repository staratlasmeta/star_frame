//! Test helpers

use crate::{Advance, AdvanceArray};
use bytemuck::{cast_slice, cast_slice_mut, from_bytes, from_bytes_mut};
use num_traits::ToPrimitive;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::MAX_PERMITTED_DATA_INCREASE;
use solana_program::system_instruction::MAX_PERMITTED_DATA_LENGTH;
use std::cell::RefCell;
use std::rc::Rc;

/// A testing account info.
/// Creates valid account infos for testing.
#[derive(Debug, Clone)]
pub struct TestAccountInfo {
    data: Vec<u64>,
}
impl TestAccountInfo {
    const PADDING_BYTE_OFFSET: usize = 0;
    const ORIGINAL_DATA_LENGTH_BYTE_OFFSET: usize = Self::PADDING_BYTE_OFFSET + 4;
    const KEY_BYTE_OFFSET: usize = Self::ORIGINAL_DATA_LENGTH_BYTE_OFFSET + 4;
    const OWNER_BYTE_OFFSET: usize = Self::KEY_BYTE_OFFSET + 32;
    const LAMPORTS_BYTE_OFFSET: usize = Self::OWNER_BYTE_OFFSET + 32;
    const DATA_LENGTH_BYTE_OFFSET: usize = Self::LAMPORTS_BYTE_OFFSET + 8;
    const DATA_BYTE_OFFSET: usize = Self::DATA_LENGTH_BYTE_OFFSET + 8;

    const fn calc_number_u64(bytes: usize) -> usize {
        (bytes / 8) + (bytes % 8 != 0) as usize
    }

    /// Gets the raw bytes of the account info.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        cast_slice(&self.data)
    }

    /// Gets the data bytes of the account info.
    #[must_use]
    pub fn data_bytes(&self) -> &[u8] {
        &self.bytes()[Self::DATA_BYTE_OFFSET..][..self.data_length()]
    }

    /// Gets the raw bytes of the account info.
    #[must_use]
    pub fn bytes_mut(&mut self) -> &mut [u8] {
        cast_slice_mut(&mut self.data)
    }

    /// Gets the data bytes of the account info.
    #[must_use]
    pub fn data_bytes_mut(&mut self) -> &mut [u8] {
        let data_length = self.data_length();
        &mut self.bytes_mut()[Self::DATA_BYTE_OFFSET..][..data_length]
    }

    /// Gets the data length of the account info.
    #[must_use]
    pub fn data_length(&self) -> usize {
        u64::from_le_bytes(
            self.bytes()[Self::DATA_LENGTH_BYTE_OFFSET..][..8]
                .try_into()
                .unwrap(),
        )
        .to_usize()
        .unwrap()
    }

    /// Sets the data length of the account info.
    pub fn set_data_length(&mut self, length: usize) {
        *from_bytes_mut(&mut self.bytes_mut()[Self::DATA_LENGTH_BYTE_OFFSET..][..8]) =
            length as u64;
        self.refresh_data_increase();
    }

    /// Creates a new test account info.
    /// Allocates the data length of bytes with 0s.
    #[must_use]
    pub fn new(data_length: usize) -> Self {
        let bytes_length = Self::DATA_BYTE_OFFSET
            + (data_length + MAX_PERMITTED_DATA_INCREASE)
                .min(MAX_PERMITTED_DATA_LENGTH.to_usize().unwrap());

        let mut out = Self {
            data: vec![0; Self::calc_number_u64(bytes_length)],
        };
        out.set_data_length(data_length);
        out
    }

    /// Meant to be called as if on a transaction boundary
    pub fn refresh_data_increase(&mut self) {
        let length = self.data_length();
        let bytes_length = Self::DATA_BYTE_OFFSET
            + (length + MAX_PERMITTED_DATA_INCREASE)
                .min(MAX_PERMITTED_DATA_LENGTH.to_usize().unwrap());
        self.data.resize(Self::calc_number_u64(bytes_length), 0);
        self.bytes_mut()[Self::ORIGINAL_DATA_LENGTH_BYTE_OFFSET..][..4]
            .copy_from_slice(&u32::to_le_bytes(length.to_u32().unwrap()));
        for byte in &mut self.bytes_mut()[Self::DATA_BYTE_OFFSET + length..] {
            *byte = 0;
        }
    }

    /// Gets the valid account info.
    pub fn account_info(&mut self) -> AccountInfo {
        let mut bytes = self.bytes_mut();
        let _padding: &mut [_; 4] = bytes.advance_array();
        let _original_data_length: &mut [_; 4] = bytes.advance_array();
        let key: &mut [_; 32] = bytes.advance_array();
        let owner: &mut [_; 32] = bytes.advance_array();
        let lamports: &mut [_; 8] = bytes.advance_array();
        let data_length_bytes: &mut [_; 8] = bytes.advance_array();
        let length = *from_bytes_mut::<u64>(data_length_bytes);
        let data_bytes = bytes.advance(length.to_usize().unwrap());

        AccountInfo {
            key: from_bytes(key),
            lamports: Rc::new(RefCell::new(from_bytes_mut(lamports))),
            data: Rc::new(RefCell::new(data_bytes)),
            owner: from_bytes(owner),
            rent_epoch: 0,
            is_signer: false,
            is_writable: false,
            executable: false,
        }
    }
}
