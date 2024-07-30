//! An example of an escrow program, inspired by PaulX tutorial seen here
//! https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/
//!
//! User (Initializer) constructs an escrow deal:
//! - SPL token (X) they will offer and amount
//! - SPL Mint (Y) count they want in return and amount
//! - Program will hold in escrow the maker token amount
//!
//! Once this escrow is initialised, either:
//! 1. User (Taker) can call the exchange function to exchange their Y for X
//! - This will close the escrow account and no longer be usable
//! OR
//! 2. If no one has exchanged, the maker can close the escrow account
//! - Initializer will get back the maker token amount

use star_frame::prelude::*;

mod instructions;
mod state;

#[derive(StarFrameProgram)]
#[program(
    instruction_set = instructions::EscrowInstructionSet<'static>,
    id =  "EScwddmALCPeQU8o4cNKjT6GSVGrHesvj3Xamc1fnErY",
)]
pub struct EscrowProgram {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::EscrowAccount;
    use borsh::to_vec;
    use bytemuck::checked::try_from_bytes;
    use instructions::{CancelIx, ExchangeIx, InitEscrowIx};
    use solana_program_test::*;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction as SolanaInstruction},
        signature::{Keypair, Signer},
        system_program,
        transaction::Transaction,
    };
    use spl_associated_token_account::get_associated_token_address;
    use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
    use star_frame::itertools::Itertools;
    use star_frame::solana_program::program_pack::Pack;

    async fn _create_mint(
        mint_authority: &Pubkey,
        funder: &Pubkey,
        mint_rent: u64,
    ) -> Result<(Keypair, [SolanaInstruction; 2])> {
        const MINT_LEN: u64 = 82;
        let mint_keypair = Keypair::new();
        let create_mint_account_instruction = solana_sdk::system_instruction::create_account(
            funder,
            &mint_keypair.pubkey(),
            mint_rent,
            MINT_LEN,
            &spl_token::ID,
        );
        let initialize_mint_instruction = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint_keypair.pubkey(),
            &mint_authority,
            None,
            9,
        )
        .unwrap();

        Ok((
            mint_keypair,
            [create_mint_account_instruction, initialize_mint_instruction],
        ))
    }

    async fn _load_token(
        token_address: &Pubkey,
        banks_client: &mut BanksClient,
    ) -> Result<spl_token::state::Account> {
        let account_info = banks_client
            .get_account(*token_address)
            .await
            .unwrap()
            .unwrap();
        Ok(spl_token::state::Account::unpack(&account_info.data)?)
    }

    async fn _load_escrow(
        escrow_address: &Pubkey,
        banks_client: &mut BanksClient,
    ) -> Result<EscrowAccount> {
        let escrow_info = banks_client
            .get_account(*escrow_address)
            .await
            .unwrap()
            .unwrap();
        Ok(*try_from_bytes::<EscrowAccount>(&escrow_info.data[8..])?)
    }

    #[tokio::test]
    async fn test_escrow() {
        // Initialize the test environment
        let program_test = ProgramTest::new(
            "escrow",
            EscrowProgram::PROGRAM_ID,
            processor!(EscrowProgram::processor),
        );
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let maker = Keypair::new();
        let taker = Keypair::new();

        // Create mints
        const MINT_LEN: u64 = 82;
        let mint_rent = banks_client
            .get_rent()
            .await
            .unwrap()
            .minimum_balance(MINT_LEN as usize);
        let mint_result1 = _create_mint(&payer.pubkey(), &payer.pubkey(), mint_rent)
            .await
            .unwrap();
        let mint_result2 = _create_mint(&payer.pubkey(), &payer.pubkey(), mint_rent)
            .await
            .unwrap();
        let mut mint_ixs: Vec<SolanaInstruction> = Vec::new();
        mint_ixs.extend_from_slice(&mint_result1.1);
        mint_ixs.extend_from_slice(&mint_result2.1);
        let transaction = Transaction::new_signed_with_payer(
            &mint_ixs,
            Some(&payer.pubkey()),
            &[&payer, &mint_result1.0, &mint_result2.0],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // get escrow address
        let maker_token1 = get_associated_token_address(&maker.pubkey(), &mint_result1.0.pubkey());
        let maker_token2 = get_associated_token_address(&maker.pubkey(), &mint_result2.0.pubkey());
        let seeds = state::EscrowAccountSeeds {
            maker: maker.pubkey(),
            maker_deposit_token_account: maker_token1,
            exchange_mint: mint_result2.0.pubkey(),
        };
        let (escrow_key, bump) =
            Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::PROGRAM_ID);

        // create token accounts
        let transaction = Transaction::new_signed_with_payer(
            &[
                create_associated_token_account_idempotent(
                    &payer.pubkey(), // funder
                    &maker.pubkey(), // owner
                    &mint_result1.0.pubkey(),
                    &spl_token::ID,
                ),
                create_associated_token_account_idempotent(
                    &payer.pubkey(), // funder
                    &escrow_key,     // owner
                    &mint_result1.0.pubkey(),
                    &spl_token::ID,
                ),
                create_associated_token_account_idempotent(
                    &payer.pubkey(), // funder
                    &taker.pubkey(), // owner
                    &mint_result1.0.pubkey(),
                    &spl_token::ID,
                ),
                create_associated_token_account_idempotent(
                    &payer.pubkey(), // funder
                    &maker.pubkey(), // owner
                    &mint_result2.0.pubkey(),
                    &spl_token::ID,
                ),
                create_associated_token_account_idempotent(
                    &payer.pubkey(), // funder
                    &taker.pubkey(), // owner
                    &mint_result2.0.pubkey(),
                    &spl_token::ID,
                ),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // mint 100 to both maker and taker
        let taker_token2 = get_associated_token_address(&taker.pubkey(), &mint_result2.0.pubkey());
        let transaction = Transaction::new_signed_with_payer(
            &[
                spl_token::instruction::mint_to(
                    &spl_token::ID,
                    &mint_result2.0.pubkey(),
                    &taker_token2,
                    &payer.pubkey(),
                    &[&payer.pubkey()],
                    100,
                )
                .unwrap(),
                spl_token::instruction::mint_to(
                    &spl_token::ID,
                    &mint_result1.0.pubkey(),
                    &maker_token1,
                    &payer.pubkey(),
                    &[&payer.pubkey()],
                    100,
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();

        // init escrow
        let escrow_token1 = get_associated_token_address(&escrow_key, &mint_result1.0.pubkey());
        let maker_amount = 41;
        let taker_amount = 66;
        let maker_token1_data0 = _load_token(&maker_token1, &mut banks_client).await.unwrap();
        let escrow_token1_data0 = _load_token(&escrow_token1, &mut banks_client)
            .await
            .unwrap();
        let ix_data = [
            InitEscrowIx::DISCRIMINANT.to_vec(),
            to_vec(&InitEscrowIx {
                maker_amount,
                taker_amount,
            })
            .unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let instruction = SolanaInstruction::new_with_bytes(
            EscrowProgram::PROGRAM_ID,
            &ix_data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(maker.pubkey(), true),
                AccountMeta::new(maker_token1, false),
                AccountMeta::new_readonly(maker_token2, false),
                AccountMeta::new(escrow_token1, false),
                AccountMeta::new_readonly(mint_result2.0.pubkey(), false),
                AccountMeta::new(escrow_key, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
        );
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer, &maker],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();
        let expected = EscrowAccount {
            version: 0,
            maker: maker.pubkey(),
            maker_deposit_token_account: maker_token1,
            maker_receive_token_account: maker_token2,
            escrow_token_account: escrow_token1,
            exchange_mint: mint_result2.0.pubkey(),
            maker_amount,
            taker_amount,
            bump,
        };
        let escrow_account = _load_escrow(&escrow_key, &mut banks_client).await.unwrap();
        assert_eq!(expected, escrow_account);
        let maker_token1_data1 = _load_token(&maker_token1, &mut banks_client).await.unwrap();
        let escrow_token1_data1 = _load_token(&escrow_token1, &mut banks_client)
            .await
            .unwrap();
        assert_eq!(
            maker_token1_data0.amount - maker_token1_data1.amount,
            maker_amount
        );
        assert_eq!(
            escrow_token1_data1.amount - escrow_token1_data0.amount,
            maker_amount
        );
        let maker_token2_data1 = _load_token(&maker_token2, &mut banks_client).await.unwrap();

        // make the exchange
        let taker_token1 = get_associated_token_address(&taker.pubkey(), &mint_result1.0.pubkey());
        let taker_token1_data1 = _load_token(&taker_token1, &mut banks_client).await.unwrap();
        let taker_token2_data1 = _load_token(&taker_token2, &mut banks_client).await.unwrap();
        let ix_data2 = [
            ExchangeIx::DISCRIMINANT.to_vec(),
            to_vec(&ExchangeIx {}).unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let instruction2 = SolanaInstruction::new_with_bytes(
            EscrowProgram::PROGRAM_ID,
            &ix_data2,
            vec![
                AccountMeta::new(maker.pubkey(), false),
                AccountMeta::new(maker_token2, false),
                AccountMeta::new_readonly(taker.pubkey(), true),
                AccountMeta::new(taker_token2, false),
                AccountMeta::new(taker_token1, false),
                AccountMeta::new(escrow_key, false),
                AccountMeta::new(escrow_token1, false),
                AccountMeta::new_readonly(mint_result2.0.pubkey(), false),
                AccountMeta::new_readonly(spl_token::ID, false),
            ],
        );
        let transaction = Transaction::new_signed_with_payer(
            &[instruction2],
            Some(&payer.pubkey()),
            &[&payer, &taker],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();
        let escrow_info2 = banks_client.get_account(escrow_key).await.unwrap();
        assert!(escrow_info2.is_none());
        let escrow_token_info = banks_client.get_account(escrow_token1).await.unwrap();
        assert!(escrow_token_info.is_none());
        let maker_token1_data2 = _load_token(&maker_token1, &mut banks_client).await.unwrap();
        let maker_token2_data2 = _load_token(&maker_token2, &mut banks_client).await.unwrap();
        let taker_token1_data2 = _load_token(&taker_token1, &mut banks_client).await.unwrap();
        let taker_token2_data2 = _load_token(&taker_token2, &mut banks_client).await.unwrap();
        assert_eq!(maker_token1_data1.amount, maker_token1_data2.amount);
        assert_eq!(
            maker_token2_data2.amount - maker_token2_data1.amount,
            taker_amount
        );
        assert_eq!(
            taker_token1_data2.amount - taker_token1_data1.amount,
            maker_amount
        );
        assert_eq!(
            taker_token2_data1.amount - taker_token2_data2.amount,
            taker_amount
        );

        // init escrow then cancel
        let maker_token1_data3 = _load_token(&maker_token1, &mut banks_client).await.unwrap();
        let instruction0 = SolanaInstruction::new_with_bytes(
            EscrowProgram::PROGRAM_ID,
            &ix_data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(maker.pubkey(), true),
                AccountMeta::new(maker_token1, false),
                AccountMeta::new_readonly(maker_token2, false),
                AccountMeta::new(escrow_token1, false),
                AccountMeta::new_readonly(mint_result2.0.pubkey(), false),
                AccountMeta::new(escrow_key, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
        );
        let ix_data3 = [
            CancelIx::DISCRIMINANT.to_vec(),
            to_vec(&CancelIx {}).unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let instruction3 = SolanaInstruction::new_with_bytes(
            EscrowProgram::PROGRAM_ID,
            &ix_data3,
            vec![
                AccountMeta::new(maker.pubkey(), true),
                AccountMeta::new(maker_token1, false),
                AccountMeta::new(escrow_key, false),
                AccountMeta::new(escrow_token1, false),
                AccountMeta::new_readonly(mint_result2.0.pubkey(), false),
                AccountMeta::new_readonly(spl_token::ID, false),
            ],
        );
        let transaction = Transaction::new_signed_with_payer(
            &[
                create_associated_token_account_idempotent(
                    &payer.pubkey(), // funder
                    &escrow_key,     // owner
                    &mint_result1.0.pubkey(),
                    &spl_token::ID,
                ),
                instruction0,
                instruction3,
            ],
            Some(&payer.pubkey()),
            &[&payer, &maker],
            recent_blockhash,
        );
        banks_client.process_transaction(transaction).await.unwrap();
        let maker_token1_data4 = _load_token(&maker_token1, &mut banks_client).await.unwrap();
        assert_eq!(maker_token1_data4.amount, maker_token1_data3.amount);
        let escrow_info2 = banks_client.get_account(escrow_key).await.unwrap();
        assert!(escrow_info2.is_none());
        let escrow_token_info = banks_client.get_account(escrow_token1).await.unwrap();
        assert!(escrow_token_info.is_none());
    }
}
