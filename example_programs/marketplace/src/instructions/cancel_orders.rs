use crate::state::{CancelOrderArgs, MarketExclusiveImpl};
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

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

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        orders_to_cancel: Self::RunArg<'_>,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let cancelled_totals = account_set
            .market
            .data_mut()?
            .cancel_orders(account_set.user.pubkey(), orders_to_cancel)?;

        account_set.withdraw(cancelled_totals, ctx)?;

        Ok(())
    }
}
