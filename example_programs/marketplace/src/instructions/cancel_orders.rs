use crate::state::{CancelOrderArgs, MarketExclusiveImpl};
use star_frame::{
    borsh::{BorshDeserialize, BorshSerialize},
    prelude::*,
};

use crate::instructions::ManageOrderAccounts;

/// Cancels orders for a marketplace
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, InstructionArgs)]
#[borsh(crate = "star_frame::borsh")]
pub struct CancelOrders {
    #[ix_args(&run)]
    pub args: Vec<CancelOrderArgs>,
}

impl StarFrameInstruction for CancelOrders {
    type ReturnType = ();
    type Accounts<'b, 'c> = ManageOrderAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        orders_to_cancel: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let cancelled_totals = accounts
            .market
            .data_mut()?
            .cancel_orders(accounts.user.pubkey(), orders_to_cancel)?;

        accounts.withdraw(cancelled_totals, ctx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instructions::ManageOrderClientAccounts,
        state::{
            MakerInfo, Market, MarketOwned, MarketSeeds, OrderBookSideOwned, OrderInfo,
            OrderTotals, Price, Quantity, ASK_ID_MASK,
        },
        test_utils::{new_mint_account, new_token_account, token_account_data, LAMPORTS_PER_SOL},
        Marketplace,
    };
    use mollusk_svm::result::Check;
    use solana_account::Account as SolanaAccount;
    use star_frame::{
        client::SerializeAccount as _, data_types::PackedValue, solana_pubkey::Pubkey,
    };
    use star_frame_spl::associated_token::AssociatedToken;
    use std::{collections::HashMap, env};

    fn price(v: u64) -> Price {
        Price::new(PackedValue(v))
    }
    fn qty(v: u64) -> Quantity {
        Quantity::new(PackedValue(v))
    }

    #[test]
    fn cancel_orders() -> Result<()> {
        if env::var("SBF_OUT_DIR").is_err() {
            println!("SBF_OUT_DIR is not set, skipping test");
            return Ok(());
        }
        let mollusk = crate::test_utils::new_mollusk();

        // Keys
        let payer = Pubkey::new_unique();
        let user = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let currency_mint = KeyFor::new(Pubkey::new_unique());
        let market_token_mint = KeyFor::new(Pubkey::new_unique());
        let (market_pda, bump) = Market::find_program_address(&MarketSeeds {
            currency: currency_mint,
            market_token: market_token_mint,
        });

        // Vault addresses
        let currency_vault = AssociatedToken::find_address(&market_pda, &currency_mint);
        let market_token_vault = AssociatedToken::find_address(&market_pda, &market_token_mint);
        let user_currency_vault = Pubkey::new_unique();
        let user_market_token_vault = Pubkey::new_unique();
        // Seed market with a single resting bid and ask owned by user
        let bid_price = price(10);
        let bid_qty = qty(5);
        let ask_price = price(11);
        let ask_qty = qty(6);
        let bids_filled_qty = qty(3);
        let asks_filled_price = price(12);
        let market_owned = MarketOwned {
            version: 0,
            bump,
            authority,
            currency: currency_mint,
            market_token: market_token_mint,
            bids: OrderBookSideOwned {
                id_counter: 1,
                makers: [(
                    user,
                    MakerInfo {
                        totals: OrderTotals {
                            currency: bid_price * bid_qty,
                            market_tokens: bids_filled_qty,
                        },
                        order_count: 1,
                    },
                )]
                .into_iter()
                .collect(),
                orders: vec![OrderInfo {
                    price: bid_price,
                    quantity: bid_qty,
                    order_id: 0,
                    maker: user,
                }],
            },
            asks: OrderBookSideOwned {
                id_counter: ASK_ID_MASK + 1,
                makers: [(
                    user,
                    MakerInfo {
                        totals: OrderTotals {
                            currency: asks_filled_price,
                            market_tokens: ask_qty,
                        },
                        order_count: 1,
                    },
                )]
                .into_iter()
                .collect(),
                orders: vec![OrderInfo {
                    price: ask_price,
                    quantity: ask_qty,
                    order_id: ASK_ID_MASK,
                    maker: user,
                }],
            },
        };
        let market_data = Market::serialize_account(market_owned.clone())?;
        let account_store = HashMap::from_iter([
            (
                payer,
                SolanaAccount::new(
                    LAMPORTS_PER_SOL,
                    0,
                    &star_frame::program::system::System::ID,
                ),
            ),
            (
                user,
                SolanaAccount::new(
                    LAMPORTS_PER_SOL,
                    0,
                    &star_frame::program::system::System::ID,
                ),
            ),
            (
                authority,
                SolanaAccount::new(
                    LAMPORTS_PER_SOL,
                    0,
                    &star_frame::program::system::System::ID,
                ),
            ),
            (
                market_pda,
                SolanaAccount {
                    lamports: LAMPORTS_PER_SOL,
                    data: market_data,
                    owner: Marketplace::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            new_mint_account(currency_mint),
            new_mint_account(market_token_mint),
            new_token_account(currency_vault, market_pda, currency_mint, 1_000_000),
            new_token_account(market_token_vault, market_pda, market_token_mint, 100_000),
            new_token_account(user_currency_vault, user, currency_mint, 0),
            new_token_account(user_market_token_vault, user, market_token_mint, 0),
            mollusk_svm::program::keyed_account_for_system_program(),
        ]);

        let mollusk = mollusk.with_context(account_store);

        // Cancel the order
        mollusk.process_and_validate_instruction(
            &crate::Marketplace::instruction(
                &CancelOrders {
                    args: vec![
                        CancelOrderArgs {
                            order_id: 0,
                            price: bid_price,
                        },
                        CancelOrderArgs {
                            order_id: ASK_ID_MASK,
                            price: ask_price,
                        },
                    ],
                },
                ManageOrderClientAccounts {
                    funder: payer,
                    user,
                    market: market_pda,
                    currency: *currency_mint.pubkey(),
                    market_token: *market_token_mint.pubkey(),
                    market_token_vault,
                    currency_vault,
                    user_market_token_vault,
                    user_currency_vault,
                    token_program: None,
                },
            )?,
            &[
                Check::success(),
                Check::account(&user_currency_vault)
                    .data(&token_account_data(
                        user,
                        currency_mint,
                        (bid_price * bid_qty + asks_filled_price).val().0,
                    ))
                    .build(),
                Check::account(&user_market_token_vault)
                    .data(&token_account_data(
                        user,
                        market_token_mint,
                        (ask_qty + bids_filled_qty).val().0,
                    ))
                    .build(),
            ],
        );

        Ok(())
    }
}
