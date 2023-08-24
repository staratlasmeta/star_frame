use crate::client::{ParsedAccount, RpcExtError};
use crate::RemainingDataWithArg;
use common_utils::{SafeZeroCopyAccount, WrappableAccount};
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use std::cell::{Ref, RefCell};
use std::marker::PhantomData;
use std::ops::Deref;

/// A parsed account with remaining data.
#[derive(Debug)]
pub struct AccountWithRemaining<A, R, AA = ()>
where
    A: SafeZeroCopyAccount + WrappableAccount<AA>,
    for<'a> <A::RemainingData as RemainingDataWithArg<'a, AA>>::Data: Deref,
    for<'a> <<A::RemainingData as RemainingDataWithArg<'a, AA>>::Data as Deref>::Target:
        ToOwned<Owned = R>,
{
    /// The account's key.
    pub key: Pubkey,
    /// The account's lamports.
    pub lamports: u64,
    /// The account's header.
    pub header: A,
    /// The account's remaining data.
    pub remaining: R,
    /// The account's extra data.
    pub extra: Vec<u8>,
    /// Phantom for the arg of the remaining data.
    pub phantom: PhantomData<fn() -> AA>,
}
impl<A, R, AA> AccountWithRemaining<A, R, AA>
where
    A: SafeZeroCopyAccount + WrappableAccount<AA>,
    for<'a> <A::RemainingData as RemainingDataWithArg<'a, AA>>::Data: Deref,
    for<'a> <<A::RemainingData as RemainingDataWithArg<'a, AA>>::Data as Deref>::Target:
        ToOwned<Owned = R>,
{
    /// Create a new [`AccountWithRemaining`] from the given key, account, and arg.
    pub fn from_parsed_account_with_arg(
        parsed_account: ParsedAccount<A>,
        arg: AA,
    ) -> Result<Self, RpcExtError> {
        let data = RefCell::new(parsed_account.extra);
        let (account_remaining, extra_data) =
            A::RemainingData::remaining_data_with_arg(Ref::map(data.borrow(), |r| &**r), arg)?;

        Ok(Self {
            key: parsed_account.key,
            lamports: parsed_account.lamports,
            header: parsed_account.header,
            remaining: account_remaining.deref().to_owned(),
            extra: extra_data.to_vec(),
            phantom: PhantomData,
        })
    }

    /// Create a new [`AccountWithRemaining`] from the given key, account, and arg.
    #[inline]
    pub fn from_account_with_arg(
        key: Pubkey,
        account: &Account,
        arg: AA,
    ) -> Result<Self, RpcExtError> {
        Self::from_parsed_account_with_arg(ParsedAccount::from_account(key, account)?, arg)
    }
}

impl<A, R> AccountWithRemaining<A, R, ()>
where
    A: SafeZeroCopyAccount + WrappableAccount,
    for<'a> <A::RemainingData as RemainingDataWithArg<'a, ()>>::Data: Deref,
    for<'a> <<A::RemainingData as RemainingDataWithArg<'a, ()>>::Data as Deref>::Target:
        ToOwned<Owned = R>,
{
    /// Create a new [`AccountWithRemaining`] from the given key and account.
    #[inline]
    pub fn from_parsed_account(parsed_account: ParsedAccount<A>) -> Result<Self, RpcExtError> where
    {
        Self::from_parsed_account_with_arg(parsed_account, ())
    }

    /// Create a new [`AccountWithRemaining`] from the given key and account.
    #[inline]
    pub fn from_account(key: Pubkey, account: &Account) -> Result<Self, RpcExtError> {
        Self::from_account_with_arg(key, account, ())
    }
}
