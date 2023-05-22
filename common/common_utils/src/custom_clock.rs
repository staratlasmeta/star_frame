//! Clock that can be set for testing and development.

#[cfg(feature = "development")]
use crate::AdvanceArray;
use common_utils::prelude::*;
use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;

/// Gets the custom clock if the `development` feature is enabled, otherwise gets the real clock.
#[cfg_attr(not(feature = "development"), allow(unused_variables))]
pub fn get_clock(remaining_accounts: &mut &[AccountInfo]) -> Result<Clock> {
    #[cfg(feature = "development")]
    {
        msg!("Development feature enabled!");
        if remaining_accounts
            .get(0)
            .map_or(false, |a| a.owner == &custom_clock::CustomClock::owner())
        {
            msg!("Getting Development Clock!");
            let custom_clock: &[_; 1] = remaining_accounts.try_advance_array()?;
            let custom_clock =
                AccountLoader::<custom_clock::CustomClock>::try_from(&custom_clock[0])?;
            let custom_clock = custom_clock.load()?;
            return Ok(Clock {
                slot: custom_clock.slot,
                epoch_start_timestamp: 0,
                epoch: 0,
                leader_schedule_epoch: 0,
                unix_timestamp: custom_clock.timestamp,
            });
        }
    }
    Ok(Clock::get()?)
}
