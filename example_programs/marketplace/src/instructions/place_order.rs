use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

use crate::instructions::ManageOrderAccounts;
use crate::state::{MarketExclusiveImpl, OrderSide, OrderTotals, ProcessOrderArgs};

/// Opens a new order for a marketplace
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
#[borsh(crate = "star_frame::borsh")]
pub struct PlaceOrder {
    #[ix_args(run)]
    pub args: ProcessOrderArgs,
}

impl StarFrameInstruction for PlaceOrder {
    type ReturnType = Option<u64>;
    type Accounts<'b, 'c> = ManageOrderAccounts;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        process_order_args: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let order_result = account_set
            .market
            .data_mut()?
            .process_order(process_order_args, *account_set.user.pubkey())?;

        let mut withdraw_totals = OrderTotals::default();
        let mut deposit_totals = OrderTotals::default();

        match process_order_args.side {
            OrderSide::Bid => {
                // // Bids lock up currency and return market tokens
                deposit_totals.currency = order_result.total_cost();
                withdraw_totals.market_tokens = order_result.executed_quantity;
            }
            OrderSide::Ask => {
                // Asks lock up market tokens and return currency
                deposit_totals.market_tokens = order_result.total_quantity();
                withdraw_totals.currency = order_result.executed_cost;
            }
        }

        account_set.withdraw(withdraw_totals, ctx)?;
        account_set.deposit(deposit_totals, ctx)?;

        Ok(order_result.order_id)
    }
}
