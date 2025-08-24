use std::{cmp::Reverse, fmt::Display};

use star_frame::{
    anyhow::{ensure, Context as _},
    prelude::*,
};

create_unit_system!(pub struct MarketplaceUnitSystem<Currency>);

use marketplace_unit_system_units::{Currency, Unitless};
use star_frame_spl::token::state::MintAccount;

pub type Price = UnitVal<PackedValue<u64>, Currency>;
pub type Quantity = UnitVal<PackedValue<u64>, Unitless>;

pub const ZERO_PRICE: Price = Price::new(PackedValue(0));
pub const ZERO_QUANTITY: Quantity = Quantity::new(PackedValue(0));

pub const ASK_ID_MASK: u64 = 1 << 63;

#[derive(Eq, Debug, Pod, PartialEq, Zeroable, Copy, Clone, Ord, PartialOrd, TypeToIdl, Align1)]
#[repr(C, packed)]
pub struct OrderInfo {
    /// The price in currency (set on the market)
    pub price: Price,
    /// The quantity of market tokens being sold
    pub quantity: Quantity,
    /// A unique (for the market) id for this order
    pub order_id: u64,
    /// The key of the maker who placed the order
    pub maker: Pubkey,
}

#[derive(Eq, Debug, PartialEq, Pod, Zeroable, Copy, Clone, TypeToIdl, Default)]
#[repr(C, packed)]
pub struct OrderTotals {
    /// currency either escrowed from buy orders or released from completed sell orders
    pub currency: Price,
    /// Market tokens either escrowed from sell orders or released from completed buy orders
    pub market_tokens: Quantity,
}

impl OrderTotals {
    /// Updates the totals when an existing order is filled or partially filled
    pub fn update_existing(&mut self, price: Price, quantity: Quantity, fill_side: OrderSide) {
        match fill_side {
            OrderSide::Bid => {
                // Buy orders escrow currency and release market_tokens
                self.currency -= price * quantity;
                self.market_tokens += quantity;
            }
            OrderSide::Ask => {
                // Sell orders escrow market_tokens and release currency
                self.currency += price * quantity;
                self.market_tokens -= quantity;
            }
        }
    }

    pub fn combine(&self, other: &Self) -> Self {
        Self {
            currency: self.currency + other.currency,
            market_tokens: self.market_tokens + other.market_tokens,
        }
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, NoUninit, Zeroable, CheckedBitPattern, Align1, TypeToIdl,
)]
#[repr(u8)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn order_matches(&self, limit_price: Price, book_price: Price) -> bool {
        match self {
            OrderSide::Bid => limit_price >= book_price,
            OrderSide::Ask => limit_price <= book_price,
        }
    }

    pub fn reverse(&self) -> Self {
        match self {
            OrderSide::Bid => OrderSide::Ask,
            OrderSide::Ask => OrderSide::Bid,
        }
    }

    #[inline]
    pub fn from_id(id: u64) -> Self {
        if id & ASK_ID_MASK == ASK_ID_MASK {
            OrderSide::Ask
        } else {
            OrderSide::Bid
        }
    }
}

borsh_with_bytemuck!(OrderSide);

#[derive(Eq, Debug, PartialEq, Pod, Zeroable, Default, Copy, Clone, TypeToIdl, Align1)]
#[repr(C, packed)]
pub struct MakerInfo {
    pub totals: OrderTotals,
    /// Total open orders for this maker
    pub order_count: u16,
}

impl MakerInfo {
    /// Mark an order as completely filled. This reduces the order count and updates rent bytes owned.
    pub fn mark_order_filled(&mut self) {
        self.order_count -= 1;
    }

    #[must_use]
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            totals: self.totals.combine(&other.totals),
            order_count: self.order_count + other.order_count,
        }
    }

    #[must_use]
    pub fn total_currency(&self) -> Price {
        self.totals.currency
    }

    #[must_use]
    pub fn total_market_tokens(&self) -> Quantity {
        self.totals.market_tokens
    }

    #[must_use]
    fn maybe_combine(left: Option<&Self>, right: Option<&Self>) -> Option<Self> {
        match (left, right) {
            (Some(left), Some(right)) => Some(left.combine(right)),
            (Some(left), None) => Some(*left),
            (None, Some(right)) => Some(*right),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct OrderBookResult {
    pub order_id: Option<u64>,
    pub executed_cost: Price,
    pub executed_quantity: Quantity,
    pub remaining_cost: Price,
    pub remaining_quantity: Quantity,
}

impl Display for OrderBookResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrderBookResult")
            .field("order_id", &self.order_id)
            .field("executed_cost", &{ self.executed_cost.val().0 })
            .field("executed_quantity", &{ self.executed_quantity.val().0 })
            .field("remaining_cost", &{ self.remaining_cost.val().0 })
            .field("remaining_quantity", &{ self.remaining_quantity.val().0 })
            .finish()
    }
}

impl OrderBookResult {
    pub fn total_cost(&self) -> Price {
        self.executed_cost + self.remaining_cost
    }

    pub fn total_quantity(&self) -> Quantity {
        self.executed_quantity + self.remaining_quantity
    }
}

#[unsized_type]
pub struct OrderBookSide {
    /// An incrememnting counter for each order id. The first bit is set to 1 for asks.
    pub id_counter: u64,
    #[unsized_start]
    pub makers: Map<Pubkey, MakerInfo>,
    pub orders: List<OrderInfo>,
}

#[unsized_impl]
impl OrderBookSide {
    pub fn is_empty(&self) -> bool {
        self.makers.is_empty() && self.orders.is_empty()
    }

    #[exclusive]
    pub fn remove_maker(&mut self, maker: &Pubkey) -> Result<Option<MakerInfo>> {
        let maker = self.makers().remove(maker)?;
        Ok(maker)
    }

    #[inline]
    pub fn find_bid_index(&self, price: Price, order_id: u64) -> Result<usize, usize> {
        // Bids are sorted by price descending, then order id ascending
        self.orders
            .binary_search_by_key(&(Reverse(price), order_id), |o| {
                (Reverse(o.price), o.order_id)
            })
    }

    #[inline]
    pub fn find_ask_index(&self, price: Price, order_id: u64) -> Result<usize, usize> {
        // Asks are sorted by price ascending, then order id ascending
        self.orders
            .binary_search_by_key(&(price, order_id), |o| (o.price, o.order_id))
    }

    #[inline]
    pub fn find_order_index(
        &self,
        price: Price,
        order_id: u64,
        side: OrderSide,
    ) -> Result<usize, usize> {
        match side {
            OrderSide::Bid => self.find_bid_index(price, order_id),
            OrderSide::Ask => self.find_ask_index(price, order_id),
        }
    }

    /// Returns the new order id
    #[exclusive]
    fn add_order(
        &mut self,
        price: Price,
        quantity: Quantity,
        side: OrderSide,
        maker: Pubkey,
    ) -> Result<u64> {
        let order_id = self.id_counter;
        self.id_counter += 1;
        let order_index = self.find_order_index(price, order_id, side);
        // The search should fail (and give the insertion index) because we're searching with a new order id
        let insertion_index = order_index.err().context(
            "Order book with same price and order id already exists. This should never happen.",
        )?;
        self.orders().insert(
            insertion_index,
            OrderInfo {
                price,
                quantity,
                order_id,
                maker,
            },
        )?;

        let maker_info = self.get_or_insert_maker(&maker)?;
        maker_info.order_count += 1;

        match side {
            OrderSide::Bid => {
                maker_info.totals.currency += price * quantity;
            }
            OrderSide::Ask => {
                maker_info.totals.market_tokens += quantity;
            }
        }

        Ok(order_id)
    }

    #[exclusive]
    fn process_order_inner(
        &mut self,
        price: Price,
        quantity: Quantity,
        side: OrderSide,
    ) -> Result<OrderBookResult> {
        let self_mut = &mut **self;
        let mut orders_consumed = 0usize;
        let mut remaining_quantity = quantity;
        let mut executed_cost: Price = ZERO_PRICE;
        for book_order in self_mut.orders.iter_mut() {
            if remaining_quantity.val() == 0 || !side.order_matches(price, book_order.price) {
                break;
            }

            let book_order_maker = self_mut
                .makers
                .get_mut(&book_order.maker)
                .context("Missing order maker. This should never happen.")?;

            let quantity_consumed = if { book_order.quantity } >= remaining_quantity {
                // Book order is partially filled
                remaining_quantity
            } else {
                // Book order is fully filled
                book_order_maker.mark_order_filled();
                orders_consumed += 1;

                book_order.quantity
            };

            book_order_maker.totals.update_existing(
                book_order.price,
                quantity_consumed,
                // The fill side is the opposite side of the order, so we need to reverse the order side here!
                side.reverse(),
            );
            remaining_quantity -= quantity_consumed;

            // This should never overflow since spl token total supply is a u64.
            // (and if it does it will panic, so we don't need to do any checked math here)
            executed_cost += book_order.price * quantity_consumed;
            // Always reduce quantity on the order, even if it's being cleaned up (just in case it doesn't get cleaned up somehow!)
            book_order.quantity -= quantity_consumed;
        }
        self.orders().remove_range(0..orders_consumed)?;

        Ok(OrderBookResult {
            order_id: None,
            executed_cost,
            executed_quantity: quantity - remaining_quantity,
            remaining_cost: remaining_quantity * price,
            remaining_quantity,
        })
    }

    /// Returns the maker info and the additional rent bytes used
    #[exclusive]
    pub fn get_or_insert_maker(&mut self, maker: &Pubkey) -> Result<&mut MakerInfo> {
        if !self.makers.contains_key(maker) {
            self.makers().insert(
                *maker,
                MakerInfo {
                    totals: OrderTotals::default(),
                    order_count: 0,
                },
            )?;
        }
        Ok(self.makers.get_mut(maker).expect("Maker was just inserted"))
    }
}

#[derive(Debug, Copy, Clone, GetSeeds)]
#[get_seeds(seed_const = b"market")]
pub struct MarketSeeds {
    pub currency: KeyFor<MintAccount>,
    pub market_token: KeyFor<MintAccount>,
}

#[derive(Debug, Copy, Clone)]
pub struct CreateMarketArgs {
    pub authority: Pubkey,
    pub currency: KeyFor<MintAccount>,
    pub market_token: KeyFor<MintAccount>,
    pub bump: u8,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, NoUninit, Zeroable, CheckedBitPattern, Align1, TypeToIdl,
)]
#[repr(C, packed)]
pub struct ProcessOrderArgs {
    pub side: OrderSide,
    pub price: Price,
    pub quantity: Quantity,
    pub fill_or_kill: bool,
}

borsh_with_bytemuck!(ProcessOrderArgs);

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, NoUninit, Zeroable, CheckedBitPattern, Align1, TypeToIdl,
)]
#[repr(C, packed)]
pub struct CancelOrderArgs {
    pub order_id: u64,
    pub price: Price,
}

borsh_with_bytemuck!(CancelOrderArgs);

#[derive(Debug, Copy, Clone)]
pub struct CancelOrdersResult {
    pub totals: OrderTotals,
    pub cancelled_count: usize,
}

#[unsized_type(program_account, seeds = MarketSeeds)]
pub struct Market {
    /// The version flag of this account type
    pub version: u8,
    /// The bump for the PDA
    pub bump: u8,
    pub authority: Pubkey,
    pub currency: KeyFor<MintAccount>,
    pub market_token: KeyFor<MintAccount>,
    #[unsized_start]
    /// The bids for this market, with orders sorted by price from highest to lowest
    pub bids: OrderBookSide,
    /// The asks for this market, with orders sorted by price from lowest to highest
    pub asks: OrderBookSide,
}

pub struct ValidateMarketToken<'a>(pub &'a KeyFor<MintAccount>);
pub struct ValidateCurrency<'a>(pub &'a KeyFor<MintAccount>);

impl<'a> AccountValidate<ValidateMarketToken<'a>> for Market {
    fn validate_account(self_ref: &Self::Ref<'_>, arg: ValidateMarketToken<'a>) -> Result<()> {
        ensure!(&self_ref.market_token == arg.0, "Market token mismatch");
        Ok(())
    }
}

impl<'a> AccountValidate<ValidateCurrency<'a>> for Market {
    fn validate_account(self_ref: &Self::Ref<'_>, arg: ValidateCurrency<'a>) -> Result<()> {
        ensure!(&self_ref.currency == arg.0, "Currency mismatch");
        Ok(())
    }
}

#[unsized_impl]
impl Market {
    pub fn initialize(&mut self, args: CreateMarketArgs) {
        let CreateMarketArgs {
            currency,
            market_token,
            bump,
            authority,
        } = args;
        **self = MarketSized {
            version: 0,
            bump,
            authority,
            currency,
            market_token,
        };
        self.asks.id_counter = ASK_ID_MASK; // Set the first bit to a 1 for asks to tell the difference between asks and bids
    }

    #[exclusive]
    fn clear_orders_for_cleanup(&mut self) -> Result<()> {
        self.bids().orders().clear()?;
        self.asks().orders().clear()?;
        Ok(())
    }

    pub fn get_combined_maker_info(&self, maker: &Pubkey) -> Option<MakerInfo> {
        let bid_maker = self.bids.makers.get(maker);
        let ask_maker = self.asks.makers.get(maker);
        MakerInfo::maybe_combine(bid_maker, ask_maker)
    }

    #[exclusive]
    pub fn remove_maker_for_cleanup(&mut self, maker: &Pubkey) -> Result<Option<MakerInfo>> {
        self.clear_orders_for_cleanup()?;
        let bid_maker = self.bids().remove_maker(maker)?;
        let ask_maker = self.asks().remove_maker(maker)?;
        let combined_maker = MakerInfo::maybe_combine(bid_maker.as_ref(), ask_maker.as_ref());
        Ok(combined_maker)
    }

    #[exclusive]
    pub fn process_order(
        &mut self,
        args: ProcessOrderArgs,
        maker: Pubkey,
    ) -> Result<OrderBookResult> {
        let ProcessOrderArgs {
            side,
            price,
            quantity,
            fill_or_kill,
        } = args;
        let (mut order_result, mut maker_book) = match args.side {
            // Bids are processed against asks
            OrderSide::Bid => (
                // TODO: Charge fees if we really need to for sellers
                self.asks().process_order_inner(price, quantity, side)?,
                self.bids(),
            ),
            // Asks are processed against bids
            OrderSide::Ask => (
                self.bids().process_order_inner(price, quantity, side)?,
                self.asks(),
            ),
        };

        if order_result.remaining_quantity > ZERO_QUANTITY {
            if fill_or_kill {
                bail!("Fill or kill order was not filled");
            }
            let order_id =
                maker_book.add_order(price, order_result.remaining_quantity, side, maker)?;

            order_result.order_id = Some(order_id);
        }

        Ok(order_result)
    }

    #[exclusive]
    pub fn cancel_orders(
        &mut self,
        maker: &Pubkey,
        orders_to_cancel: &[CancelOrderArgs],
    ) -> Result<OrderTotals> {
        let mut cancelled_currency = ZERO_PRICE;
        let mut cancelled_market_tokens = ZERO_QUANTITY;
        let mut cancelled_bids = 0;
        let mut cancelled_asks = 0;
        let mut filled_market_tokens = ZERO_QUANTITY;
        let mut filled_currency = ZERO_PRICE;

        for order in orders_to_cancel {
            let side = OrderSide::from_id(order.order_id);
            let book = match side {
                OrderSide::Bid => &mut self.bids(),
                OrderSide::Ask => &mut self.asks(),
            };
            let Ok(index) = book.find_order_index(order.price, order.order_id, side) else {
                // Silently ignore orders that don't exist
                continue;
            };
            let order_to_remove = book.orders[index];
            book.orders().remove(index)?;
            ensure!(
                &order_to_remove.maker == maker,
                "Order maker does not match order to remove"
            );

            match side {
                OrderSide::Bid => {
                    // Bid maker stores currency from pending bids
                    cancelled_currency += order_to_remove.price * order_to_remove.quantity;
                    cancelled_bids += 1;
                }
                OrderSide::Ask => {
                    // Ask maker stores market_tokens from pending asks
                    cancelled_market_tokens += order_to_remove.quantity;
                    cancelled_asks += 1;
                }
            }
        }

        let mut remove_bid_maker = false;
        let mut remove_ask_maker = false;

        if let Some(bid_maker) = self.bids.makers.get_mut(maker) {
            bid_maker.totals.currency -= cancelled_currency;
            bid_maker.order_count -= cancelled_bids;
            // Withdraw filled order market_tokens
            filled_market_tokens = core::mem::take(&mut bid_maker.totals.market_tokens);
            remove_bid_maker = bid_maker.order_count == 0;
        } else if cancelled_bids > 0 {
            bail!("Bids cancelled but no bid maker found");
        }

        if let Some(ask_maker) = self.asks.makers.get_mut(maker) {
            ask_maker.totals.market_tokens -= cancelled_market_tokens;
            ask_maker.order_count -= cancelled_asks;
            // Withdraw filled order currency
            filled_currency = core::mem::take(&mut ask_maker.totals.currency);
            remove_ask_maker = ask_maker.order_count == 0;
        } else if cancelled_asks > 0 {
            bail!("Asks cancelled but no ask maker found");
        }

        if remove_bid_maker {
            self.bids().makers().remove(maker)?;
        }
        if remove_ask_maker {
            self.asks().makers().remove(maker)?;
        }

        Ok(OrderTotals {
            currency: cancelled_currency + filled_currency,
            market_tokens: cancelled_market_tokens + filled_market_tokens,
        })
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;

    use pretty_assertions::assert_eq;
    use star_frame::unsize::ModifyOwned;

    use crate::test_utils::{new_price, new_quantity};

    use super::*;

    pub fn default_market() -> MarketOwned {
        MarketOwned {
            version: 0,
            bump: 0,
            bids: OrderBookSideOwned {
                id_counter: 0,
                makers: BTreeMap::from_iter([]),
                orders: vec![],
            },
            asks: OrderBookSideOwned {
                id_counter: ASK_ID_MASK,
                makers: BTreeMap::from_iter([]),
                orders: vec![],
            },
            authority: Pubkey::new_unique(),
            currency: KeyFor::new(Pubkey::new_unique()),
            market_token: KeyFor::new(Pubkey::new_unique()),
        }
    }

    #[test]
    fn unsized_test_place_orders() -> Result<()> {
        let mut market = default_market();
        let maker = Pubkey::new_unique();
        let price = new_price(10);
        let quantity = new_quantity(10);
        let side = OrderSide::Bid;
        let fill_or_kill = false;

        let mut order_result = OrderBookResult::default();

        let mut expected_market = market.clone();

        Market::modify_owned(&mut market, |market| {
            order_result = market.process_order(
                ProcessOrderArgs {
                    side,
                    price,
                    quantity,
                    fill_or_kill,
                },
                maker,
            )?;
            Ok(())
        })?;

        expected_market.bids.orders.push(OrderInfo {
            price,
            quantity,
            order_id: 0,
            maker,
        });
        expected_market.bids.id_counter += 1;
        expected_market.bids.makers.insert(
            maker,
            MakerInfo {
                totals: OrderTotals {
                    currency: price * quantity,
                    market_tokens: ZERO_QUANTITY,
                },
                order_count: 1,
            },
        );

        assert_eq!(market, expected_market);

        assert_eq!(
            order_result,
            OrderBookResult {
                order_id: Some(0),
                executed_cost: ZERO_PRICE,
                executed_quantity: ZERO_QUANTITY,
                remaining_cost: price * quantity,
                remaining_quantity: quantity,
            }
        );

        let price = new_price(20);
        let quantity = new_quantity(15);
        Market::modify_owned(&mut market, |market| {
            order_result = market.process_order(
                ProcessOrderArgs {
                    side,
                    price,
                    quantity,
                    fill_or_kill,
                },
                maker,
            )?;
            Ok(())
        })?;

        // Insert the new order at the beginning of the list since the price is higher (so it should be at the top of the book)
        expected_market.bids.orders.insert(
            0,
            OrderInfo {
                price,
                quantity,
                // Order id is 1 higher
                order_id: 1,
                maker,
            },
        );
        expected_market.bids.id_counter += 1;
        let expected_maker_info = expected_market
            .bids
            .makers
            .get_mut(&maker)
            .expect("Maker info should exist");
        expected_maker_info.totals.currency += price * quantity;
        expected_maker_info.order_count += 1;

        assert_eq!(market, expected_market);

        assert_eq!(
            order_result,
            OrderBookResult {
                order_id: Some(1),
                executed_cost: ZERO_PRICE,
                executed_quantity: ZERO_QUANTITY,
                remaining_cost: price * quantity,
                remaining_quantity: quantity,
            }
        );

        // Now add some to the sell side
        let maker = Pubkey::new_unique();
        let price = new_price(25);
        let quantity = new_quantity(10);
        let side = OrderSide::Ask;
        let fill_or_kill = false;

        Market::modify_owned(&mut market, |market| {
            order_result = market.process_order(
                ProcessOrderArgs {
                    side,
                    price,
                    quantity,
                    fill_or_kill,
                },
                maker,
            )?;
            Ok(())
        })?;

        expected_market.asks.orders.push(OrderInfo {
            price,
            quantity,
            order_id: ASK_ID_MASK,
            maker,
        });
        expected_market.asks.id_counter += 1;
        expected_market.asks.makers.insert(
            maker,
            MakerInfo {
                totals: OrderTotals {
                    currency: ZERO_PRICE,
                    market_tokens: quantity,
                },
                order_count: 1,
            },
        );

        assert_eq!(market, expected_market);

        assert_eq!(
            order_result,
            OrderBookResult {
                order_id: Some(ASK_ID_MASK),
                executed_cost: ZERO_PRICE,
                executed_quantity: ZERO_QUANTITY,
                remaining_cost: price * quantity,
                remaining_quantity: quantity,
            }
        );

        Ok(())
    }

    #[test]
    fn unsized_test_match_orders() -> Result<()> {
        let buyer1 = Pubkey::new_unique();
        let buyer1_price = new_price(10);
        let buyer1_quantity = new_quantity(10);

        let buyer2 = Pubkey::new_unique();
        let buyer2_price = new_price(8);
        let buyer2_quantity = new_quantity(8);

        let buyer3 = Pubkey::new_unique();
        let buyer3_price = new_price(6);
        let buyer3_quantity = new_quantity(6);

        let seller1 = Pubkey::new_unique();
        let seller1_price = new_price(12);
        let seller1_quantity = new_quantity(10);

        let seller2 = Pubkey::new_unique();
        let seller2_price = new_price(14);
        let seller2_quantity = new_quantity(8);

        let seller3 = Pubkey::new_unique();
        let seller3_price = new_price(16);
        let seller3_quantity = new_quantity(6);

        let mut market = MarketOwned {
            bids: OrderBookSideOwned {
                id_counter: 3,
                makers: BTreeMap::from_iter([
                    (
                        buyer1,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: buyer1_price * buyer1_quantity,
                                market_tokens: ZERO_QUANTITY,
                            },
                            order_count: 1,
                        },
                    ),
                    (
                        buyer2,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: buyer2_price * buyer2_quantity,
                                market_tokens: ZERO_QUANTITY,
                            },
                            order_count: 1,
                        },
                    ),
                    (
                        buyer3,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: buyer3_price * buyer3_quantity,
                                market_tokens: ZERO_QUANTITY,
                            },
                            order_count: 1,
                        },
                    ),
                ]),
                orders: vec![
                    OrderInfo {
                        price: buyer1_price,
                        quantity: buyer1_quantity,
                        order_id: 0,
                        maker: buyer1,
                    },
                    OrderInfo {
                        price: buyer2_price,
                        quantity: buyer2_quantity,
                        order_id: 1,
                        maker: buyer2,
                    },
                    OrderInfo {
                        price: buyer3_price,
                        quantity: buyer3_quantity,
                        order_id: 2,
                        maker: buyer3,
                    },
                ],
            },
            asks: OrderBookSideOwned {
                id_counter: ASK_ID_MASK + 3,
                makers: BTreeMap::from_iter([
                    (
                        seller1,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: ZERO_PRICE,
                                market_tokens: seller1_quantity,
                            },
                            order_count: 1,
                        },
                    ),
                    (
                        seller2,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: ZERO_PRICE,
                                market_tokens: seller2_quantity,
                            },
                            order_count: 1,
                        },
                    ),
                    (
                        seller3,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: ZERO_PRICE,
                                market_tokens: seller3_quantity,
                            },
                            order_count: 1,
                        },
                    ),
                ]),
                orders: vec![
                    OrderInfo {
                        price: seller1_price,
                        quantity: seller1_quantity,
                        order_id: ASK_ID_MASK,
                        maker: seller1,
                    },
                    OrderInfo {
                        price: seller2_price,
                        quantity: seller2_quantity,
                        order_id: ASK_ID_MASK + 1,
                        maker: seller2,
                    },
                    OrderInfo {
                        price: seller3_price,
                        quantity: seller3_quantity,
                        order_id: ASK_ID_MASK + 2,
                        maker: seller3,
                    },
                ],
            },
            ..default_market()
        };

        let market_maker_buyer = Pubkey::new_unique();
        let mm_buy_price = new_price(15);
        let mm_buy_quantity = new_quantity(15);
        let mut order_result = OrderBookResult::default();

        let mut expected_market = market.clone();

        Market::modify_owned(&mut market, |market| {
            order_result = market.process_order(
                ProcessOrderArgs {
                    side: OrderSide::Bid,
                    price: mm_buy_price,
                    quantity: mm_buy_quantity,
                    fill_or_kill: false,
                },
                market_maker_buyer,
            )?;
            Ok(())
        })?;

        // The buy order for 15 units at price 15 will match against the asks:
        // 1. seller1's order of 10 units at price 12 is fully filled.
        // 2. seller2's order of 8 units at price 14 is partially filled by 5 units.
        let executed_cost_s1 = seller1_price * seller1_quantity; // 120
        let executed_quantity_s1 = seller1_quantity; // 10

        let executed_quantity_s2 = new_quantity(5);
        let executed_cost_s2 = seller2_price * executed_quantity_s2; // 70

        // Update seller1's state
        let seller1_maker = expected_market.asks.makers.get_mut(&seller1).unwrap();
        seller1_maker.order_count -= 1;
        seller1_maker.totals.currency += executed_cost_s1;
        seller1_maker.totals.market_tokens -= executed_quantity_s1;

        // Update seller2's state
        let seller2_maker = expected_market.asks.makers.get_mut(&seller2).unwrap();
        seller2_maker.totals.currency += executed_cost_s2;
        seller2_maker.totals.market_tokens -= executed_quantity_s2;
        expected_market.asks.orders[1].quantity -= executed_quantity_s2;

        // Remove filled order
        expected_market.asks.orders.remove(0);

        assert_eq!(market, expected_market);

        assert_eq!(
            order_result,
            OrderBookResult {
                order_id: None,
                executed_cost: executed_cost_s1 + executed_cost_s2,
                executed_quantity: executed_quantity_s1 + executed_quantity_s2,
                remaining_cost: ZERO_PRICE,
                remaining_quantity: ZERO_QUANTITY,
            }
        );

        // Now, a market maker places a sell order that partially fills and then rests on the book
        let market_maker_seller = Pubkey::new_unique();
        let mm_sell_price = new_price(7);
        let mm_sell_quantity = new_quantity(20);

        Market::modify_owned(&mut market, |market| {
            order_result = market.process_order(
                ProcessOrderArgs {
                    side: OrderSide::Ask,
                    price: mm_sell_price,
                    quantity: mm_sell_quantity,
                    fill_or_kill: false,
                },
                market_maker_seller,
            )?;
            Ok(())
        })?;

        // The sell order for 20 units at price 7 will match against bids:
        // 1. buyer1's order of 10 units at price 10 is fully filled.
        // 2. buyer2's order of 8 units at price 8 is fully filled.
        // The remaining 2 units will be placed as a new ask order.
        let executed_cost_b1 = buyer1_price * buyer1_quantity; // 100
        let executed_quantity_b1 = buyer1_quantity; // 10

        let executed_cost_b2 = buyer2_price * buyer2_quantity; // 64
        let executed_quantity_b2 = buyer2_quantity; // 8

        // Update and remove buyer1's state
        let buyer1_maker = expected_market.bids.makers.get_mut(&buyer1).unwrap();
        buyer1_maker.order_count = 0;
        buyer1_maker.totals.currency = ZERO_PRICE;
        buyer1_maker.totals.market_tokens = buyer1_quantity;

        // Update and remove buyer2's state
        let buyer2_maker = expected_market.bids.makers.get_mut(&buyer2).unwrap();
        buyer2_maker.order_count = 0;
        buyer2_maker.totals.currency = ZERO_PRICE;
        buyer2_maker.totals.market_tokens = buyer2_quantity;

        // The third bid order and its maker remain unchanged
        // Remove filled orders (first two)
        expected_market.bids.orders.remove(0);
        expected_market.bids.orders.remove(0);

        let remaining_sell_quantity =
            mm_sell_quantity - executed_quantity_b1 - executed_quantity_b2; // 2
        let new_ask_order_id = expected_market.asks.id_counter;
        expected_market.asks.id_counter += 1;
        expected_market.asks.orders.insert(
            0,
            OrderInfo {
                price: mm_sell_price,
                quantity: remaining_sell_quantity,
                order_id: new_ask_order_id,
                maker: market_maker_seller,
            },
        );
        expected_market.asks.makers.insert(
            market_maker_seller,
            MakerInfo {
                totals: OrderTotals {
                    currency: ZERO_PRICE,
                    market_tokens: remaining_sell_quantity,
                },
                order_count: 1,
            },
        );

        assert_eq!(market, expected_market);

        let expected_executed_quantity = executed_quantity_b1 + executed_quantity_b2;
        let expected_executed_cost = executed_cost_b1 + executed_cost_b2;

        assert_eq!(
            order_result,
            OrderBookResult {
                order_id: Some(new_ask_order_id),
                executed_cost: expected_executed_cost,
                executed_quantity: expected_executed_quantity,
                remaining_cost: mm_sell_price * remaining_sell_quantity,
                remaining_quantity: remaining_sell_quantity,
            }
        );

        Ok(())
    }

    #[test]
    fn unsized_test_cancel_orders() -> Result<()> {
        let buyer1 = Pubkey::new_unique();
        let buyer1_price = new_price(10);
        let buyer1_quantity = new_quantity(5);
        let buyer1_filled_quantity = new_quantity(5);
        let buyer1_id = 0u64;

        let buyer2 = Pubkey::new_unique();
        let buyer2_price = new_price(7);
        let buyer2_quantity = new_quantity(10);
        let buyer2_id = 1u64;

        let seller = Pubkey::new_unique();
        let seller_price = new_price(14);
        let seller_quantity = new_quantity(8);
        let seller_filled_price = new_price(5);
        let seller_id = ASK_ID_MASK;

        let maker = Pubkey::new_unique();
        let maker_first_buy_price = new_price(6);
        let maker_first_buy_quantity = new_quantity(12);
        let maker_first_buy_id = 2u64;
        let maker_bids_filled_market_tokens = new_quantity(10);
        let maker_second_buy_price = new_price(7); // another buy for this price
        let maker_second_buy_quantity = new_quantity(8);
        let maker_second_buy_id = 3u64;

        let maker_first_sell_price = new_price(12);
        let maker_first_sell_quantity = new_quantity(10);
        let maker_first_sell_id = ASK_ID_MASK + 1;
        let maker_asks_filled_currency = new_price(3);
        let maker_second_sell_price = new_price(16);
        let maker_second_sell_quantity = new_quantity(8);
        let maker_second_sell_id = ASK_ID_MASK + 2;

        let mut local_market = MarketOwned {
            bids: OrderBookSideOwned {
                id_counter: 4,
                makers: BTreeMap::from_iter([
                    (
                        buyer1,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: buyer1_price * buyer1_quantity,
                                market_tokens: buyer1_filled_quantity,
                            },
                            order_count: 1,
                        },
                    ),
                    (
                        buyer2,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: buyer2_price * buyer2_quantity,
                                market_tokens: ZERO_QUANTITY,
                            },
                            order_count: 1,
                        },
                    ),
                    (
                        maker,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: maker_first_buy_price * maker_first_buy_quantity
                                    + maker_second_buy_price * maker_second_buy_quantity,
                                market_tokens: maker_bids_filled_market_tokens,
                            },
                            order_count: 2,
                        },
                    ),
                ]),
                orders: vec![
                    OrderInfo {
                        price: buyer1_price,
                        quantity: buyer1_quantity,
                        order_id: buyer1_id,
                        maker: buyer1,
                    },
                    OrderInfo {
                        price: buyer2_price,
                        quantity: buyer2_quantity,
                        order_id: buyer2_id,
                        maker: buyer2,
                    },
                    OrderInfo {
                        price: maker_second_buy_price,
                        quantity: maker_second_buy_quantity,
                        order_id: maker_second_buy_id,
                        maker,
                    },
                    OrderInfo {
                        price: maker_first_buy_price,
                        quantity: maker_first_buy_quantity,
                        order_id: maker_first_buy_id,
                        maker,
                    },
                ],
            },
            asks: OrderBookSideOwned {
                id_counter: ASK_ID_MASK + 2,
                makers: BTreeMap::from_iter([
                    (
                        maker,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: maker_asks_filled_currency,
                                market_tokens: maker_first_sell_quantity
                                    + maker_second_sell_quantity,
                            },
                            order_count: 2,
                        },
                    ),
                    (
                        seller,
                        MakerInfo {
                            totals: OrderTotals {
                                currency: seller_filled_price,
                                market_tokens: seller_quantity,
                            },
                            order_count: 1,
                        },
                    ),
                ]),
                orders: vec![
                    OrderInfo {
                        price: maker_first_sell_price,
                        quantity: maker_first_sell_quantity,
                        order_id: maker_first_sell_id,
                        maker,
                    },
                    OrderInfo {
                        price: seller_price,
                        quantity: seller_quantity,
                        order_id: seller_id,
                        maker: seller,
                    },
                    OrderInfo {
                        price: maker_second_sell_price,
                        quantity: maker_second_sell_quantity,
                        order_id: maker_second_sell_id,
                        maker,
                    },
                ],
            },
            ..default_market()
        };

        let mut expected_market = local_market.clone();

        // Test cancelling a single bid
        let mut cancel_result = None;
        Market::modify_owned(&mut local_market, |market| {
            cancel_result = Some(market.cancel_orders(
                &buyer1,
                &[CancelOrderArgs {
                    order_id: buyer1_id,
                    price: buyer1_price,
                }],
            )?);
            Ok(())
        })?;

        // Update expected market - buyer1's bid order should be removed
        expected_market.bids.orders.remove(0);
        expected_market.bids.makers.remove(&buyer1);
        assert_eq!(local_market, expected_market);

        // Verify cancel result
        let cancel_result = cancel_result.unwrap();
        assert_eq!(
            cancel_result,
            OrderTotals {
                currency: buyer1_price * buyer1_quantity,
                market_tokens: buyer1_filled_quantity,
            }
        );

        // Test cancelling multiple orders from the same maker
        let mut cancel_result = None;
        Market::modify_owned(&mut local_market, |market| {
            cancel_result = Some(market.cancel_orders(
                &maker,
                &[
                    CancelOrderArgs {
                        order_id: maker_first_buy_id,
                        price: maker_first_buy_price,
                    },
                    CancelOrderArgs {
                        order_id: maker_second_sell_id,
                        price: maker_second_sell_price,
                    },
                    CancelOrderArgs {
                        order_id: maker_first_sell_id,
                        price: maker_first_sell_price,
                    },
                ],
            )?);
            Ok(())
        })?;

        // Update expected market - 3 of maker's orders should be removed, leaving 1 buy order
        // Remove maker's first buy order (last in bids.orders)
        expected_market.bids.orders.pop();
        // Remove maker's ask orders (indices 0 and 2 in asks.orders)
        expected_market.asks.orders.remove(2); // Remove second sell order first (higher index)
        expected_market.asks.orders.remove(0); // Remove first sell order

        // Update maker's bid info - reduce order count and currency, but keep maker since 1 order remains
        let maker_bid_info = expected_market.bids.makers.get_mut(&maker).unwrap();
        maker_bid_info.order_count = 1; // Only second buy order remains
        maker_bid_info.totals.currency = maker_second_buy_price * maker_second_buy_quantity;
        maker_bid_info.totals.market_tokens = ZERO_QUANTITY;

        // Remove maker from asks side since all ask orders are cancelled
        expected_market.asks.makers.remove(&maker);

        assert_eq!(local_market, expected_market);

        // Verify cancel result
        let cancel_result = cancel_result.unwrap();
        let expected_cancelled_currency =
            maker_first_buy_price * maker_first_buy_quantity + maker_asks_filled_currency;
        let expected_cancelled_market_tokens = maker_first_sell_quantity
            + maker_second_sell_quantity
            + maker_bids_filled_market_tokens;
        assert_eq!(
            cancel_result,
            OrderTotals {
                currency: expected_cancelled_currency,
                market_tokens: expected_cancelled_market_tokens,
            }
        );
        // 3 orders cancelled + 1 maker entry removed (from asks side)
        // Test cancelling non-existent order (should be silently ignored)
        let mut cancel_result = None;
        Market::modify_owned(&mut local_market, |market| {
            cancel_result = Some(market.cancel_orders(
                &seller,
                &[CancelOrderArgs {
                    order_id: 999,
                    price: new_price(100),
                }],
            )?);
            Ok(())
        })?;
        // the only thing that should change is the maker should get their filled values returned

        // Market should remain unchanged
        let seller_maker_info = expected_market.asks.makers.get_mut(&seller).unwrap();
        let expected_currency_return = core::mem::take(&mut seller_maker_info.totals.currency);

        assert_eq!(local_market, expected_market);

        // Verify cancel result - should be zero since no orders were cancelled
        let cancel_result = cancel_result.unwrap();
        assert_eq!(
            cancel_result,
            OrderTotals {
                currency: expected_currency_return,
                market_tokens: ZERO_QUANTITY,
            }
        );

        // Test cancelling order with wrong maker (should fail)
        let result = Market::modify_owned(&mut local_market, |market| {
            market.cancel_orders(
                &maker,
                &[CancelOrderArgs {
                    order_id: buyer2_id,
                    price: buyer2_price,
                }],
            )?;
            Ok(())
        });

        assert!(result.is_err());

        Ok(())
    }
}
