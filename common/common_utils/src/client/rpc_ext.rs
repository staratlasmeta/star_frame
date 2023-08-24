use crate::client::ParsedAccount;
use crate::prelude::build_transaction;
use crate::{BoxedAnchorError, RemainingDataWithArg, SafeZeroCopyAccount, WrappableAccount};
use array_init::array_init;
use async_trait::async_trait;
use bytemuck::PodCastError;
use common_utils::client::InstructionWithSigners;
use common_utils::prelude::{AccountWithRemaining, DynSigner};
use futures::future::try_join_all;
use solana_account_decoder::UiAccountEncoding;
use solana_client::client_error::ClientError;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{
    RpcAccountInfoConfig, RpcSendTransactionConfig, RpcTransactionConfig,
};
use solana_client::rpc_request::RpcError;
use solana_program::clock::Slot;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::{Transaction, TransactionError};
use solana_transaction_status::{TransactionConfirmationStatus, TransactionStatus};
use std::collections::HashMap;
use std::ops::Deref;
use std::time::Duration;
use thiserror::Error;
use tokio::time::{interval, MissedTickBehavior};

/// Config for [`RpcClientExt::confirm_transactions_with_config`]
#[derive(Debug)]
pub struct ConfirmTransactionConfig {
    /// How often to check for confirmations
    pub loop_rate: Duration,
    /// The minimum confirmation status to wait for
    pub min_confirmation: TransactionConfirmationStatus,
}
impl Default for ConfirmTransactionConfig {
    fn default() -> Self {
        Self {
            loop_rate: Duration::from_secs(1),
            min_confirmation: TransactionConfirmationStatus::Confirmed,
        }
    }
}

/// Config for [`RpcClientExt::build_send_and_check_with_config`]
#[derive(Debug)]
pub struct SendTransactionConfig {
    /// Config for confirming the transaction
    pub confirm_config: ConfirmTransactionConfig,
    /// Whether to skip preflight simulations
    pub skip_preflight: bool,
}
impl Default for SendTransactionConfig {
    fn default() -> Self {
        Self {
            confirm_config: ConfirmTransactionConfig::default(),
            skip_preflight: true,
        }
    }
}

/// Config for [`RpcClientExt::get_account_with_config`]
#[derive(Debug)]
pub struct GetAccountConfig {
    /// The commitment to use
    pub commitment: TransactionConfirmationStatus,
}
impl Default for GetAccountConfig {
    fn default() -> Self {
        Self {
            commitment: TransactionConfirmationStatus::Confirmed,
        }
    }
}

/// Config for [`RpcClientExt::create_funded_keys_with_config`]
#[derive(Debug)]
pub struct CreateFundedKeyConfig {
    /// Config for confirming the transaction
    pub confirm: ConfirmTransactionConfig,
    /// The amount of lamports to send
    pub lamports: u64,
}
impl Default for CreateFundedKeyConfig {
    fn default() -> Self {
        Self {
            confirm: ConfirmTransactionConfig::default(),
            lamports: LAMPORTS_PER_SOL,
        }
    }
}

fn confirmation_at_least(
    control: &TransactionConfirmationStatus,
    test: &TransactionConfirmationStatus,
) -> bool {
    matches!(
        (control, test),
        (TransactionConfirmationStatus::Processed, _)
            | (
                TransactionConfirmationStatus::Confirmed,
                TransactionConfirmationStatus::Confirmed | TransactionConfirmationStatus::Finalized,
            )
            | (
                TransactionConfirmationStatus::Finalized,
                TransactionConfirmationStatus::Finalized
            )
    )
}

/// Convert a [`TransactionConfirmationStatus`] into a [`CommitmentConfig`]
#[must_use]
pub fn tc_into_commitment(confirmation: &TransactionConfirmationStatus) -> CommitmentConfig {
    match confirmation {
        TransactionConfirmationStatus::Processed => CommitmentConfig::processed(),
        TransactionConfirmationStatus::Confirmed => CommitmentConfig::confirmed(),
        TransactionConfirmationStatus::Finalized => CommitmentConfig::finalized(),
    }
}

/// The result of a transaction
pub type TransactionResult<T> = Result<T, TxError>;

/// Error from [`RpcClientExt`]
#[derive(Debug, Error)]
pub enum RpcExtError {
    /// A client error from solana
    #[error("Client error: {0}")]
    ClientError(#[from] ClientError),
    /// An error from the rpc
    #[error("Rpc error: {0}")]
    RpcError(#[from] RpcError),
    /// An error from a transaction
    #[error("Transaction error: {0}")]
    TransactionError(#[from] TxError),
    /// An error from anchor
    #[error("Anchor error: {0}")]
    AnchorError(#[from] BoxedAnchorError),
    /// An error from a Solana Program
    #[error("Solana Program error: {0}")]
    ProgramError(#[from] ProgramError),
    /// An error from bytemuck
    #[error("PodCast error: {0}")]
    PodCastError(#[from] PodCastError),
    /// Account is invalid for provided reason
    #[error("Account not found")]
    AccountNotFound,
    /// Invalid account owner
    #[error("Invalid owner, expected: {expected}, received: {received}")]
    InvalidOwner {
        /// The expected owner
        expected: Pubkey,
        /// The owner we received
        received: Pubkey,
    },
    /// Not enough data in the account
    #[error("Account did not have enough data")]
    NotEnoughAccountData,
    /// The discriminant did not match
    #[error("Discriminant mismatch, expected: {expected:?}, received: {received:?}")]
    DiscriminantMismatch {
        /// The expected discriminant
        expected: [u8; 8],
        /// The discriminant we received
        received: [u8; 8],
    },
}

/// Error received when confirming a transaction
#[derive(Debug, Error)]
pub enum TxError {
    /// The transaction was confirmed with an error
    #[error("Transaction confirmed in slot `{slot}` with error: {error}.\nLogs: {logs:#?}")]
    TxError {
        /// The slot we confirmed the transaction in
        slot: Slot,
        /// The error that occurred
        error: TransactionError,
        /// The transaction logs.
        logs: Option<Vec<String>>,
    },
    /// The transaction was dropped
    #[error("Transaction was dropped")]
    Dropped,
}

/// An extension to [`RpcClient`] that adds some useful methods
#[async_trait]
pub trait RpcClientExt {
    /// Confirm a list of transactions using a config.
    async fn confirm_transactions_with_config(
        &self,
        sig_and_block_height: impl IntoIterator<Item = (Signature, u64)> + Send,
        config: ConfirmTransactionConfig,
    ) -> Result<HashMap<Signature, TransactionResult<Slot>>, RpcExtError>;
    /// Confirm a list of transactions
    async fn confirm_transactions(
        &self,
        sig_and_block_height: impl IntoIterator<Item = (Signature, u64)> + Send,
    ) -> Result<HashMap<Signature, TransactionResult<Slot>>, RpcExtError> {
        self.confirm_transactions_with_config(
            sig_and_block_height,
            ConfirmTransactionConfig::default(),
        )
        .await
    }
    /// Request an airdrop
    async fn request_airdrop(&self, key: Pubkey, lamports: u64) -> Result<(), RpcExtError> {
        self.request_airdrops([(key, lamports)]).await
    }
    /// Request a list of airdrops
    async fn request_airdrops<I>(&self, keys: I) -> Result<(), RpcExtError>
    where
        I: IntoIterator<Item = (Pubkey, u64)> + Send,
        I::IntoIter: Send,
    {
        self.request_airdrops_with_config(keys, ConfirmTransactionConfig::default())
            .await
    }
    /// Request a list of airdrops using a config
    async fn request_airdrops_with_config<I>(
        &self,
        keys: I,
        config: ConfirmTransactionConfig,
    ) -> Result<(), RpcExtError>
    where
        I: IntoIterator<Item = (Pubkey, u64)> + Send,
        I::IntoIter: Send;

    /// Build, send, and confirm a transaction
    async fn build_send_and_check<'a>(
        &self,
        ixs: impl IntoIterator<Item = InstructionWithSigners<'a>> + Send,
        funder: &'a DynSigner,
    ) -> Result<(Signature, Slot), RpcExtError> {
        self.build_send_and_check_with_config(ixs, funder, SendTransactionConfig::default())
            .await
    }

    /// Build, send, and confirm a transaction using a config
    async fn build_send_and_check_with_config<'a>(
        &self,
        ixs: impl IntoIterator<Item = InstructionWithSigners<'a>> + Send,
        funder: &'a DynSigner,
        config: SendTransactionConfig,
    ) -> Result<(Signature, Slot), RpcExtError>;

    /// Gets a parsed account
    async fn get_parsed_account<A>(&self, pubkey: Pubkey) -> Result<ParsedAccount<A>, RpcExtError>
    where
        A: SafeZeroCopyAccount,
    {
        self.get_parsed_account_with_config(pubkey, GetAccountConfig::default())
            .await
    }

    /// Gets a parsed account using a config
    async fn get_parsed_account_with_config<A>(
        &self,
        pubkey: Pubkey,
        config: GetAccountConfig,
    ) -> Result<ParsedAccount<A>, RpcExtError>
    where
        A: SafeZeroCopyAccount;

    /// Gets a wrapped account
    async fn get_wrapped_account<A, R>(
        &self,
        pubkey: Pubkey,
    ) -> Result<AccountWithRemaining<A, R>, RpcExtError>
    where
        A: SafeZeroCopyAccount + WrappableAccount<()>,
        for<'a> <A::RemainingData as RemainingDataWithArg<'a, ()>>::Data: Deref,
        for<'a> <<A::RemainingData as RemainingDataWithArg<'a, ()>>::Data as Deref>::Target:
            ToOwned<Owned = R>,
    {
        self.get_wrapped_account_with_arg(pubkey, ()).await
    }

    /// Gets a wrapped account using a config
    async fn get_wrapped_account_with_config<A, R>(
        &self,
        pubkey: Pubkey,
        config: GetAccountConfig,
    ) -> Result<AccountWithRemaining<A, R>, RpcExtError>
    where
        A: SafeZeroCopyAccount + WrappableAccount<()>,
        for<'a> <A::RemainingData as RemainingDataWithArg<'a, ()>>::Data: Deref,
        for<'a> <<A::RemainingData as RemainingDataWithArg<'a, ()>>::Data as Deref>::Target:
            ToOwned<Owned = R>,
    {
        self.get_wrapped_account_with_arg_and_config(pubkey, (), config)
            .await
    }

    /// Get a wrapped account using an arg
    async fn get_wrapped_account_with_arg<A, AA, R>(
        &self,
        pubkey: Pubkey,
        arg: AA,
    ) -> Result<AccountWithRemaining<A, R, AA>, RpcExtError>
    where
        A: SafeZeroCopyAccount + WrappableAccount<AA>,
        AA: Send,
        for<'a> <A::RemainingData as RemainingDataWithArg<'a, AA>>::Data: Deref,
        for<'a> <<A::RemainingData as RemainingDataWithArg<'a, AA>>::Data as Deref>::Target:
            ToOwned<Owned = R>,
    {
        self.get_wrapped_account_with_arg_and_config(pubkey, arg, GetAccountConfig::default())
            .await
    }

    /// Get a wrapped account using an arg and config
    async fn get_wrapped_account_with_arg_and_config<A, AA, R>(
        &self,
        pubkey: Pubkey,
        arg: AA,
        config: GetAccountConfig,
    ) -> Result<AccountWithRemaining<A, R, AA>, RpcExtError>
    where
        A: SafeZeroCopyAccount + WrappableAccount<AA>,
        AA: Send,
        for<'a> <A::RemainingData as RemainingDataWithArg<'a, AA>>::Data: Deref,
        for<'a> <<A::RemainingData as RemainingDataWithArg<'a, AA>>::Data as Deref>::Target:
            ToOwned<Owned = R>;

    /// Create a funded key
    async fn create_funded_key(&self) -> Result<Keypair, RpcExtError> {
        self.create_funded_key_with_config(CreateFundedKeyConfig::default())
            .await
    }

    /// Create a funded key using a config
    async fn create_funded_key_with_config(
        &self,
        config: CreateFundedKeyConfig,
    ) -> Result<Keypair, RpcExtError> {
        self.create_funded_keys_with_config(config)
            .await
            .map(|[key]| key)
    }

    /// Create funded keys
    async fn create_funded_keys<const N: usize>(&self) -> Result<[Keypair; N], RpcExtError> {
        self.create_funded_keys_with_config(CreateFundedKeyConfig::default())
            .await
    }

    /// Create funded keys using a config
    async fn create_funded_keys_with_config<const N: usize>(
        &self,
        config: CreateFundedKeyConfig,
    ) -> Result<[Keypair; N], RpcExtError>;

    /// Build a transaction fetching the recent blockhash
    async fn build_transaction<'a>(
        &self,
        instructions: impl IntoIterator<Item = InstructionWithSigners<'a>> + Send,
        funder: &'a DynSigner,
    ) -> Result<(Transaction, u64), RpcExtError> {
        self.build_transaction_with_config(
            instructions,
            funder,
            TransactionConfirmationStatus::Confirmed,
        )
        .await
    }

    /// Build a transaction using a config fetching the recent blockhash
    async fn build_transaction_with_config<'a>(
        &self,
        instructions: impl IntoIterator<Item = InstructionWithSigners<'a>> + Send,
        funder: &'a DynSigner,
        commitment: TransactionConfirmationStatus,
    ) -> Result<(Transaction, u64), RpcExtError>;
}

#[async_trait]
impl RpcClientExt for RpcClient {
    async fn confirm_transactions_with_config(
        &self,
        sig_and_block_height: impl IntoIterator<Item = (Signature, u64)> + Send,
        config: ConfirmTransactionConfig,
    ) -> Result<HashMap<Signature, TransactionResult<Slot>>, RpcExtError> {
        let mut sigs = Vec::new();
        let mut block_heights = Vec::new();
        for (sig, block_height) in sig_and_block_height {
            sigs.push(sig);
            block_heights.push(block_height);
        }
        let mut out = HashMap::new();
        let mut interval = interval(config.loop_rate);
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        while !sigs.is_empty() {
            interval.tick().await;
            let block_height = self
                .get_block_height_with_commitment(tc_into_commitment(&config.min_confirmation))
                .await?;
            let statues = self.get_signature_statuses(&sigs).await?;
            for (index, status) in statues.value.into_iter().enumerate().rev() {
                match status {
                    Some(TransactionStatus {
                        slot,
                        err,
                        confirmation_status: Some(confirmation_status),
                        ..
                    }) if confirmation_at_least(&config.min_confirmation, &confirmation_status) => {
                        let sig = sigs[index];
                        out.insert(
                            sig,
                            match err {
                                None => Ok(slot),
                                Some(error) => {
                                    let tx = self
                                        .get_transaction_with_config(
                                            &sig,
                                            RpcTransactionConfig {
                                                encoding: None,
                                                commitment: Some(tc_into_commitment(
                                                    &config.min_confirmation,
                                                )),
                                                max_supported_transaction_version: None,
                                            },
                                        )
                                        .await?;
                                    Err(TxError::TxError {
                                        slot,
                                        error,
                                        logs: tx
                                            .transaction
                                            .meta
                                            .and_then(|meta| meta.log_messages.into()),
                                    })
                                }
                            },
                        );
                        sigs.swap_remove(index);
                        block_heights.swap_remove(index);
                    }
                    _ => {}
                }
            }

            for (index, last_block_height) in block_heights.clone().into_iter().enumerate().rev() {
                if last_block_height < block_height {
                    out.insert(sigs[index], Err(TxError::Dropped));
                    sigs.swap_remove(index);
                    block_heights.swap_remove(index);
                }
            }
        }
        Ok(out)
    }

    async fn request_airdrops_with_config<I>(
        &self,
        keys: I,
        config: ConfirmTransactionConfig,
    ) -> Result<(), RpcExtError>
    where
        I: IntoIterator<Item = (Pubkey, u64)> + Send,
        I::IntoIter: Send,
    {
        let sigs = try_join_all(keys.into_iter().map(|(key, lamports)| async move {
            let (blockhash, valid_block_height) = self
                .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
                .await?;
            let sig = self
                .request_airdrop_with_blockhash(&key, lamports, &blockhash)
                .await?;
            Result::<_, RpcExtError>::Ok((sig, valid_block_height))
        }))
        .await?;

        self.confirm_transactions_with_config(sigs, config).await?;
        Ok(())
    }

    async fn build_send_and_check_with_config<'a>(
        &self,
        ixs: impl IntoIterator<Item = InstructionWithSigners<'a>> + Send,
        funder: &'a DynSigner,
        config: SendTransactionConfig,
    ) -> Result<(Signature, Slot), RpcExtError> {
        let (transaction, valid_block_height) = self.build_transaction(ixs, funder).await?;

        let sig = self
            .send_transaction_with_config(
                &transaction,
                RpcSendTransactionConfig {
                    skip_preflight: config.skip_preflight,
                    preflight_commitment: Some(
                        tc_into_commitment(&config.confirm_config.min_confirmation).commitment,
                    ),
                    encoding: None,
                    max_retries: None,
                    min_context_slot: None,
                },
            )
            .await?;

        let slot = self
            .confirm_transactions_with_config([(sig, valid_block_height)], config.confirm_config)
            .await?
            .remove(&sig)
            .unwrap()?;

        Ok((sig, slot))
    }

    async fn get_parsed_account_with_config<A>(
        &self,
        pubkey: Pubkey,
        config: GetAccountConfig,
    ) -> Result<ParsedAccount<A>, RpcExtError>
    where
        A: SafeZeroCopyAccount,
    {
        let account = self
            .get_account_with_config(
                &pubkey,
                RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: Some(tc_into_commitment(&config.commitment)),
                    min_context_slot: None,
                },
            )
            .await?
            .value
            .ok_or(RpcExtError::AccountNotFound)?;

        ParsedAccount::from_account(pubkey, &account)
    }

    async fn get_wrapped_account_with_arg_and_config<A, AA, R>(
        &self,
        pubkey: Pubkey,
        arg: AA,
        config: GetAccountConfig,
    ) -> Result<AccountWithRemaining<A, R, AA>, RpcExtError>
    where
        A: SafeZeroCopyAccount + WrappableAccount<AA>,
        AA: Send,
        for<'a> <A::RemainingData as RemainingDataWithArg<'a, AA>>::Data: Deref,
        for<'a, 'b> <<A::RemainingData as RemainingDataWithArg<'a, AA>>::Data as Deref>::Target:
            ToOwned<Owned = R>,
    {
        let parsed_account = self
            .get_parsed_account_with_config::<A>(pubkey, config)
            .await?;

        AccountWithRemaining::from_parsed_account_with_arg(parsed_account, arg)
    }

    async fn create_funded_keys_with_config<const N: usize>(
        &self,
        config: CreateFundedKeyConfig,
    ) -> Result<[Keypair; N], RpcExtError> {
        let keys = array_init(|_| Keypair::new());

        self.request_airdrops_with_config(
            keys.iter().map(|k| (k.pubkey(), config.lamports)),
            config.confirm,
        )
        .await?;

        Ok(keys)
    }

    async fn build_transaction_with_config<'a>(
        &self,
        instructions: impl IntoIterator<Item = InstructionWithSigners<'a>> + Send,
        funder: &'a DynSigner,
        commitment: TransactionConfirmationStatus,
    ) -> Result<(Transaction, u64), RpcExtError> {
        let (rbh, valid_block_height) = self
            .get_latest_blockhash_with_commitment(tc_into_commitment(&commitment))
            .await?;

        Ok((
            build_transaction(instructions, funder, rbh),
            valid_block_height,
        ))
    }
}
