#![warn(missing_debug_implementations, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::wildcard_imports,
    clippy::result_large_err
)]

use anchor_lang::prelude::*;

declare_id!("CLockX54wfiQwQoRGcMAxitSPKdNcSRousbcHEJoecou");

#[program]
#[allow(clippy::needless_pass_by_value)]
pub mod custom_clock {
    use super::*;

    pub fn init_clock(ctx: Context<InitClock>, slot: u64, timestamp: i64) -> Result<()> {
        *ctx.accounts.clock.load_init()? = CustomClock {
            version: 0,
            slot,
            timestamp,
        };
        Ok(())
    }

    pub fn set_clock(ctx: Context<SetClock>, slot: u64, timestamp: i64) -> Result<()> {
        *ctx.accounts.clock.load_mut()? = CustomClock {
            version: 0,
            slot,
            timestamp,
        };
        Ok(())
    }
}

#[derive(Debug)]
#[account(zero_copy)]
pub struct CustomClock {
    pub version: u8,
    pub slot: u64,
    pub timestamp: i64,
}

#[derive(Accounts, Debug)]
pub struct SetClock<'info> {
    #[account(mut)]
    pub clock: AccountLoader<'info, CustomClock>,
}

#[derive(Accounts, Debug)]
pub struct InitClock<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,
    #[account(
        init,
        payer = funder,
        space = 8 + 1 + 8 * 2,
    )]
    pub clock: AccountLoader<'info, CustomClock>,
    pub system_program: Program<'info, System>,
}
