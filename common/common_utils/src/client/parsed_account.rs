use crate::client::RpcExtError;
use crate::{Advance, AdvanceArray};
use bytemuck::try_from_bytes;
use common_utils::SafeZeroCopyAccount;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use std::mem::size_of;

/// A parsed account.
#[derive(Debug)]
pub struct ParsedAccount<A> {
    /// The account's key.
    pub key: Pubkey,
    /// The account's lamports.
    pub lamports: u64,
    /// The account's header.
    pub header: A,
    /// The account's extra data.
    pub extra: Vec<u8>,
}
impl<A> ParsedAccount<A>
where
    A: SafeZeroCopyAccount,
{
    /// Create a new [`ParsedAccount`] from the given key and account.
    pub fn from_account(key: Pubkey, account: &Account) -> Result<Self, RpcExtError> {
        if account.owner != A::owner() {
            return Err(RpcExtError::InvalidOwner {
                expected: A::owner(),
                received: account.owner,
            });
        }

        let mut account_data = account.data.as_slice();

        let discriminant: [_; 8] = *account_data
            .try_advance_array()
            .map_err(|_| RpcExtError::NotEnoughAccountData)?;
        if discriminant != A::discriminator() {
            return Err(RpcExtError::DiscriminantMismatch {
                expected: A::discriminator(),
                received: discriminant,
            });
        }

        let account_header_data = account_data
            .try_advance(size_of::<A>())
            .map_err(|_| RpcExtError::NotEnoughAccountData)?;
        let account_header = try_from_bytes(account_header_data)?;

        Ok(Self {
            key,
            lamports: account.lamports,
            header: *account_header,
            extra: account_data.to_vec(),
        })
    }
}
