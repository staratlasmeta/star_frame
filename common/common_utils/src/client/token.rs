use crate::client::{RpcClientExt, RpcExtError};
use crate::prelude::{ConfirmTransactionConfig, InstructionWithSigners};
use anchor_lang::Id;
use anchor_spl::token::{Mint, Token, TokenAccount};
use async_trait::async_trait;
use common_utils::prelude::DynSigner;
use futures::future::try_join_all;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_instruction::create_account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signer};
use spl_token::instruction::{initialize_account3, initialize_mint2, mint_to};
use spl_token::state::Account;

/// Config for [`RpcClientTokenExt::create_mints_with_config`]
#[derive(Debug)]
pub struct CreateMintConfig {
    /// The funder for the mint
    pub funder: Keypair,
    /// The mint
    pub mint: Keypair,
    /// The mint authority
    pub mint_authority: Keypair,
    /// The freeze authority
    pub freeze_authority: Option<Keypair>,
    /// The number of decimals for each mint
    pub decimals: u8,
}

impl Default for CreateMintConfig {
    fn default() -> Self {
        Self {
            funder: Keypair::new(),
            mint: Keypair::new(),
            mint_authority: Keypair::new(),
            freeze_authority: None,
            decimals: 8,
        }
    }
}

/// The result of [`RpcClientTokenExt::create_mints_with_config`]
#[derive(Debug)]
pub struct CreateMintResult {
    /// The mint keypair
    pub mint: Keypair,
    /// The mint authority keypair
    pub mint_authority: Keypair,
    /// The freeze authority keypair if present
    pub freeze_authority: Option<Keypair>,
    /// The decimals on the mint
    pub decimals: u8,
}

/// Config for [`RpcClientTokenExt::create_token_account_with_config`]
#[derive(Debug)]
pub struct CreateTokenAccountConfig {
    /// The mint of the token account
    pub mint: Pubkey,
    /// The owner of the token account
    pub owner: Pubkey,
}

/// An extension to [`RpcClient`] that provides methods for interacting with the token program
#[async_trait]
pub trait RpcClientTokenExt
where
    Self: RpcClientExt,
{
    /// Create a mint with the default config
    async fn create_mint(&self) -> Result<CreateMintResult, RpcExtError> {
        let funder = Keypair::new();
        // Ensure the funder has SOL for funding mint
        self.request_airdrop(funder.pubkey(), LAMPORTS_PER_SOL)
            .await?;

        self.create_mint_with_config(&funder as &DynSigner, CreateMintConfig::default())
            .await
    }

    /// Create a mint
    async fn create_mint_with_config<'a>(
        &self,
        funder: &'a DynSigner,
        config: CreateMintConfig,
    ) -> Result<CreateMintResult, RpcExtError>;

    /// Create a token account
    async fn create_token_account<'a>(
        &self,
        funder: &'a DynSigner,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Keypair, RpcExtError>;

    /// Mint to a token account
    async fn mint_to_token_account<'a>(
        &self,
        funder: &'a DynSigner,
        mint: &Pubkey,
        token_account: Pubkey,
        token_amount: u64,
        mint_authority: &'a DynSigner,
    ) -> Result<(), RpcExtError> {
        self.mint_to_token_accounts(
            funder,
            mint,
            [(token_account, token_amount)],
            mint_authority,
        )
        .await
    }

    /// Mint tokens to token accounts
    async fn mint_to_token_accounts<'a, I>(
        &self,
        funder: &'a DynSigner,
        mint: &Pubkey,
        token_accounts: I,
        mint_authority: &'a DynSigner,
    ) -> Result<(), RpcExtError>
    where
        I: IntoIterator<Item = (Pubkey, u64)> + Send,
        I::IntoIter: Send,
    {
        self.mint_to_token_accounts_with_config(
            funder,
            mint,
            token_accounts,
            mint_authority,
            ConfirmTransactionConfig::default(),
        )
        .await
    }

    /// Mint tokens to token accounts using a transaction config
    async fn mint_to_token_accounts_with_config<'a, I>(
        &self,
        funder: &'a DynSigner,
        mint: &Pubkey,
        token_accounts: I,
        mint_authority: &'a DynSigner,
        config: ConfirmTransactionConfig,
    ) -> Result<(), RpcExtError>
    where
        I: IntoIterator<Item = (Pubkey, u64)> + Send,
        I::IntoIter: Send;

    /// Returns the deserialized Token Account info
    async fn get_token_account_info(&self, token_account: &Pubkey) -> Result<Account, RpcExtError> {
        self.get_token_account_info_with_config(token_account, CommitmentConfig::confirmed())
            .await
    }

    /// Returns the deserialized Token Account info with a commitment config
    async fn get_token_account_info_with_config(
        &self,
        token_account: &Pubkey,
        config: CommitmentConfig,
    ) -> Result<Account, RpcExtError>;
}

#[async_trait]
impl RpcClientTokenExt for RpcClient {
    async fn create_mint_with_config<'a>(
        &self,
        funder: &'a DynSigner,
        config: CreateMintConfig,
    ) -> Result<CreateMintResult, RpcExtError> {
        // Create account for the mint
        let create_ix = InstructionWithSigners::build(|funder| {
            (
                create_account(
                    &funder,
                    &config.mint.pubkey(),
                    Rent::default().minimum_balance(Mint::LEN),
                    Mint::LEN as u64,
                    &Token::id(),
                ),
                [&config.mint as &DynSigner],
            )
        });

        // Initialize the mint
        let freeze_auth_pubkey = config.freeze_authority.as_ref().map(Signer::pubkey);
        let freeze_auth = freeze_auth_pubkey.as_ref();

        let init_ix = initialize_mint2(
            &Token::id(),
            &config.mint.pubkey(),
            &config.mint_authority.pubkey(),
            freeze_auth,
            config.decimals,
        )?;

        let init_ix = InstructionWithSigners::build(|_| (init_ix, vec![]));

        self.build_send_and_check([create_ix, init_ix], funder)
            .await?;

        Ok(CreateMintResult {
            mint: config.mint,
            mint_authority: config.mint_authority,
            freeze_authority: config.freeze_authority,
            decimals: config.decimals,
        })
    }

    async fn create_token_account<'a>(
        &self,
        funder: &'a DynSigner,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Result<Keypair, RpcExtError> {
        // Find keypair for token account
        let token_account = Keypair::new();

        // Create token account
        let create_ix = InstructionWithSigners::build(|funder| {
            (
                create_account(
                    &funder,
                    &token_account.pubkey(),
                    Rent::default().minimum_balance(TokenAccount::LEN),
                    TokenAccount::LEN as u64,
                    &Token::id(),
                ),
                [&token_account as &DynSigner],
            )
        });

        // Initialize token account
        let init_ix = initialize_account3(&Token::id(), &token_account.pubkey(), mint, owner)?;

        let init_ix = InstructionWithSigners::build(|_| (init_ix, vec![]));

        self.build_send_and_check([create_ix, init_ix], funder)
            .await?;

        Ok(token_account)
    }

    async fn mint_to_token_accounts_with_config<'a, I>(
        &self,
        funder: &'a DynSigner,
        mint: &Pubkey,
        token_accounts: I,
        mint_authority: &'a DynSigner,
        config: ConfirmTransactionConfig,
    ) -> Result<(), RpcExtError>
    where
        I: IntoIterator<Item = (Pubkey, u64)> + Send,
        I::IntoIter: Send,
    {
        let sigs = try_join_all(token_accounts.into_iter().map(|(key, tokens)| async move {
            let ix = mint_to(
                &Token::id(),
                mint,
                &key,
                &mint_authority.pubkey(),
                &[],
                tokens,
            )?;
            let ix = InstructionWithSigners::build(|_| (ix, vec![mint_authority]));
            let (sig, slot) = self.build_send_and_check([ix], funder).await?;
            Result::<_, RpcExtError>::Ok((sig, slot))
        }))
        .await?;
        self.confirm_transactions_with_config(sigs, config).await?;
        Ok(())
    }

    async fn get_token_account_info_with_config(
        &self,
        token_account: &Pubkey,
        config: CommitmentConfig,
    ) -> Result<Account, RpcExtError> {
        let info = match self
            .get_account_with_commitment(token_account, config)
            .await?
            .value
        {
            Some(value) => value,
            None => return Err(RpcExtError::AccountNotFound),
        };
        let token_info = Account::unpack(&info.data).expect("Failed to unpack token account");

        Ok(token_info)
    }
}
