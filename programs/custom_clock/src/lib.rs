//! Custom clock settings for development.

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

    /// Initializes a new clock.
    pub fn init_clock(ctx: Context<InitClock>, slot: u64, timestamp: i64) -> Result<()> {
        *ctx.accounts.clock.load_init()? = CustomClock {
            version: 0,
            slot,
            timestamp,
        };
        Ok(())
    }

    /// Sets a clock.
    pub fn set_clock(ctx: Context<SetClock>, slot: u64, timestamp: i64) -> Result<()> {
        *ctx.accounts.clock.load_mut()? = CustomClock {
            version: 0,
            slot,
            timestamp,
        };
        Ok(())
    }
}

/// A Custom clock that can be set by the program.
#[derive(Debug)]
#[account(zero_copy)]
pub struct CustomClock {
    /// The data version of the account. Should always be `0`.
    pub version: u8,
    /// The slot for the clock.
    pub slot: u64,
    /// The timestamp for the clock.
    pub timestamp: i64,
}

/// Sets a clock.
#[derive(Accounts, Debug)]
pub struct SetClock<'info> {
    /// The clock to set.
    #[account(mut)]
    pub clock: AccountLoader<'info, CustomClock>,
}

/// Creates a new [`CustomClock`] account.
#[derive(Accounts, Debug)]
pub struct InitClock<'info> {
    /// The funder for the new clock.
    #[account(mut)]
    pub funder: Signer<'info>,
    /// The clock to create.
    #[account(
        init,
        payer = funder,
        space = 8 + 1 + 8 * 2,
    )]
    pub clock: AccountLoader<'info, CustomClock>,
    /// The system program.
    pub system_program: Program<'info, System>,
}
