use crate::EscrowProgram;
use ::solana_program::pubkey::Pubkey;
use star_frame::prelude::*;

#[derive(Align1, Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable)]
#[repr(C, packed)]
pub struct EscrowAccount {
    pub version: u8,
    pub maker: Pubkey,
    pub maker_deposit_token_account: Pubkey,
    pub maker_receive_token_account: Pubkey,
    pub escrow_token_account: Pubkey,
    pub exchange_mint: Pubkey,
    pub maker_amount: u64,
    pub taker_amount: u64,
    pub bump: u8,
}

impl ProgramAccount for EscrowAccount {
    type OwnerProgram = EscrowProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant = [0; 8];
}

#[derive(Debug, GetSeeds)]
#[seed_const(b"ESCROW")]
pub struct EscrowAccountSeeds {
    pub maker: Pubkey,
    pub maker_deposit_token_account: Pubkey,
    pub exchange_mint: Pubkey,
}

impl SeededAccountData for EscrowAccount {
    type Seeds = EscrowAccountSeeds;
}
