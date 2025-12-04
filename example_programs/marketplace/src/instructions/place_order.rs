use star_frame::{pinocchio::log::sol_log, prelude::*};

use crate::{
    instructions::ManageOrderAccounts,
    state::{MarketExclusiveImpl, OrderSide, OrderTotals, ProcessOrderArgs},
};

/// Places (and/or fills) an order for a marketplace
///
/// For simplicity, we don't track rent, so the user that placed an order won't neccesarily get back that rent when it's filled
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
#[borsh(crate = "star_frame::borsh")]
pub struct PlaceOrder {
    #[ix_args(run)]
    pub args: ProcessOrderArgs,
}

#[zero_copy]
#[derive(Debug, PartialEq, Eq, TypeToIdl)]
pub struct PlaceOrderReturn {
    pub placed: bool,
    pub order_id: u64,
}

impl PlaceOrderReturn {
    pub fn new_order_id(&self) -> Option<u64> {
        if self.placed {
            Some(self.order_id)
        } else {
            None
        }
    }
}

impl From<Option<u64>> for PlaceOrderReturn {
    fn from(order_id: Option<u64>) -> Self {
        Self {
            placed: order_id.is_some(),
            order_id: order_id.unwrap_or(0),
        }
    }
}

#[star_frame_instruction]
fn PlaceOrder(
    accounts: &mut ManageOrderAccounts,
    process_order_args: ProcessOrderArgs,
) -> Result<PlaceOrderReturn> {
    let order_result = accounts
        .market
        .data_mut()?
        .process_order(process_order_args, *accounts.user.pubkey())?;

    let mut withdraw_totals = OrderTotals::default();
    let mut deposit_totals = OrderTotals::default();

    match process_order_args.side {
        OrderSide::Bid => {
            // Bids lock up currency and return market tokens
            deposit_totals.currency = order_result.total_cost();
            withdraw_totals.market_tokens = order_result.executed_quantity;
        }
        OrderSide::Ask => {
            // Asks lock up market tokens and return currency
            deposit_totals.market_tokens = order_result.total_quantity();
            withdraw_totals.currency = order_result.executed_cost;
        }
    }

    sol_log(order_result.to_string().as_str());

    accounts.withdraw(withdraw_totals)?;
    accounts.deposit(deposit_totals)?;

    Ok(order_result.order_id.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instructions::ManageOrderClientAccounts,
        state::{
            tests::default_market, MakerInfo, Market, MarketOwned, MarketSeeds, OrderBookSideOwned,
            OrderInfo, ASK_ID_MASK, ZERO_PRICE,
        },
        test_utils::{
            new_mint_account, new_price, new_quantity, new_token_account, token_account_data,
            LAMPORTS_PER_SOL,
        },
        Marketplace,
    };
    use mollusk_svm::result::Check;
    use solana_account::Account as SolanaAccount;
    use star_frame::{itertools::Itertools, solana_pubkey::Pubkey};
    use star_frame_spl::associated_token::AssociatedToken;
    use std::{collections::HashMap, env};
    const STARTING_USER_CURRENCY_BALANCE: u64 = 1_000_000_000;
    const STARTING_USER_MARKET_TOKEN_BALANCE: u64 = 1_000_000;

    #[test]
    fn place_bid() -> Result<()> {
        if env::var("SBF_OUT_DIR").is_err() {
            std::println!("SBF_OUT_DIR is not set, skipping test");
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
        const NUM_MAKERS: usize = 1000;
        let makers = (0..NUM_MAKERS)
            .map(|_| {
                (
                    Pubkey::new_unique(),
                    MakerInfo {
                        totals: OrderTotals {
                            currency: ZERO_PRICE,
                            market_tokens: new_quantity(10),
                        },
                        order_count: 1,
                    },
                )
            })
            .collect_vec();

        let orders = makers
            .iter()
            .enumerate()
            .map(|(i, (maker, _))| OrderInfo {
                price: new_price(i as u64),
                quantity: new_quantity(10),
                order_id: ASK_ID_MASK + i as u64,
                maker: *maker,
            })
            .collect_vec();

        let market_owned = MarketOwned {
            bump,
            authority,
            currency: currency_mint,
            market_token: market_token_mint,
            asks: OrderBookSideOwned {
                id_counter: ASK_ID_MASK + NUM_MAKERS as u64,
                makers: makers.into_iter().collect(),
                orders,
            },
            ..default_market()
        };

        let market_data = Market::serialize_account(market_owned.clone())?;
        let market_lamports = mollusk.sysvars.rent.minimum_balance(market_data.len());
        let user_currency_vault = Pubkey::new_unique();
        let user_market_token_vault = Pubkey::new_unique();
        // Accounts
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
                    lamports: market_lamports,
                    data: market_data,
                    owner: Marketplace::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            new_mint_account(currency_mint),
            new_mint_account(market_token_mint),
            new_token_account(currency_vault, market_pda, currency_mint, 0),
            new_token_account(
                market_token_vault,
                market_pda,
                market_token_mint,
                NUM_MAKERS as u64 * 10,
            ),
            new_token_account(
                user_currency_vault,
                user,
                currency_mint,
                STARTING_USER_CURRENCY_BALANCE,
            ),
            new_token_account(
                user_market_token_vault,
                user,
                market_token_mint,
                STARTING_USER_MARKET_TOKEN_BALANCE,
            ),
            mollusk_svm::program::keyed_account_for_system_program(),
        ]);

        let mollusk = mollusk.with_context(account_store);

        // Call instruction directly
        const PRICE_U64: u64 = 999;
        const ORDERS_TO_FILL: u64 = PRICE_U64 + 1;
        let quantity_u64 = 100_000;
        let price = new_price(PRICE_U64); // this should consume the first 1000 orders
        let quantity = new_quantity(quantity_u64);

        let executed_cost = (0..ORDERS_TO_FILL).sum::<u64>() * 10; // The first 1000 filled orders
        let remaining_cost = (quantity_u64 - ORDERS_TO_FILL * 10) * PRICE_U64; // the remaining new order quantity
        let consumed_cost = executed_cost + remaining_cost;

        mollusk.process_and_validate_instruction(
            &Marketplace::instruction(
                &PlaceOrder {
                    args: ProcessOrderArgs {
                        side: OrderSide::Bid,
                        price,
                        quantity,
                        fill_or_kill: false,
                    },
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
                Check::account(&currency_vault)
                    .data(&token_account_data(
                        market_pda,
                        currency_mint,
                        consumed_cost,
                    ))
                    .build(),
                Check::account(&market_token_vault)
                    .data(&token_account_data(
                        market_pda,
                        market_token_mint,
                        NUM_MAKERS as u64 * 10 - 10 * ORDERS_TO_FILL, // consumed 1000 orders of 10 market tokens each
                    ))
                    .build(),
                Check::account(&user_currency_vault)
                    .data(&token_account_data(
                        user,
                        currency_mint,
                        STARTING_USER_CURRENCY_BALANCE - consumed_cost,
                    ))
                    .build(),
                Check::account(&user_market_token_vault)
                    .data(&token_account_data(
                        user,
                        market_token_mint,
                        STARTING_USER_MARKET_TOKEN_BALANCE + 10 * ORDERS_TO_FILL, // filled 1000 orders of 10 market tokens each
                    ))
                    .build(),
            ],
        );

        let market_data = Market::deserialize_account(
            mollusk
                .account_store
                .try_borrow()?
                .get(&market_pda)
                .unwrap()
                .data
                .as_slice(),
        )?;

        assert_eq!(
            market_data.asks.orders.len(),
            NUM_MAKERS - ORDERS_TO_FILL as usize
        );
        assert_eq!(market_data.bids.orders.len(), 1); // the remaining quantity from the order being placed that was not filled

        Ok(())
    }

    // TODO: Perhaps test for asks, but it should be basically symmetrical and we have unit tests for that already
}
