use ::solana_program::pubkey::Pubkey;
use star_frame::anyhow::bail;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

// #[derive(Accounts)]
// pub struct CancelEscrow<'info> {
//     /// CHECK:
//     pub initializer: AccountInfo<'info>,
//     #[account(mut)]
//     pub pda_deposit_token_account: InterfaceAccount<'info, TokenAccount>,
//     /// CHECK:
//     pub pda_account: AccountInfo<'info>,
//     #[account(
//         mut,
//         constraint = escrow_account.initializer_key == *initializer.key,
//         constraint = escrow_account.initializer_deposit_token_account == *pda_deposit_token_account.to_account_info().key,
//         close = initializer
//     )]
//     pub escrow_account: Account<'info, EscrowAccount>,
//     pub token_program: Interface<'info, TokenInterface>,
// }