//! Custom clock settings for development.

#![warn(missing_debug_implementations, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::wildcard_imports,
    clippy::result_large_err
)]
use common_utils::prelude::*;
//     program, AccountInfo, AccountLoader, Context, Program, Pubkey, Result, System,
// };
use anchor_lang::solana_program::log::*;
// use anchor_lang::{account, zero_copy};
// use borsh::{BorshDeserialize, BorshSerialize};
use common_utils::ExclusiveMap;

declare_id!("SoRt3CQHw4RuakqngB6X3ZgBV5nya3c2NECg3rS8wZS");

#[program]
#[allow(clippy::needless_pass_by_value)]
pub mod exclusive_map {
    use super::*;

    /// Initializes a new exclusive map account.
    pub fn create_map(ctx: Context<CreateMap>) -> Result<()> {
        let list_account = &mut ctx.accounts.map_account.load_init()?;
        **list_account = ExclusiveMapAccount {
            version: 0,
            authority: *ctx.accounts.authority.key,
        };
        Ok(())
    }

    /// Initializes a new exclusive map account.
    pub fn insert_items(ctx: Context<UpdateItems>, items: InsertItemsList) -> Result<()> {
        let mut wrapper = ZeroCopyWrapper::from(&ctx.accounts.map_account);
        let items = items
            .items
            .into_iter()
            .map(|InsertItems { key, value }| (key, value.pack()))
            .collect::<Vec<_>>();
        sol_log_compute_units();
        for item in items {
            wrapper.insert(item.0, item.1)?;
        }
        sol_log_compute_units();
        normalize_rent(
            ctx.accounts.map_account.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            &ctx.accounts.system_program,
            None,
        )?;
        let item_len = wrapper.remaining()?.len();
        msg!("Total items: {}", item_len);
        Ok(())
    }

    pub fn delete_items(ctx: Context<UpdateItems>, keys: Vec<Pubkey>) -> Result<()> {
        let mut wrapper = ZeroCopyWrapper::from(&ctx.accounts.map_account);
        sol_log_compute_units();
        for key in keys {
            wrapper.remove(&key)?;
        }
        sol_log_compute_units();
        Ok(())
    }

    pub fn contains_item(ctx: Context<UpdateItems>, keys: Vec<Pubkey>) -> Result<()> {
        let wrapper = ZeroCopyWrapper::from(&ctx.accounts.map_account);
        sol_log_compute_units();
        let list_items = wrapper.remaining()?;
        let mut contains = Vec::with_capacity(keys.len());
        for key in keys {
            contains.push(list_items.index_of(&key));
        }
        sol_log_compute_units();
        msg!("contains: {:?}", contains);
        if contains.iter().any(|x| x.is_err()) {
            return err!(ErrorCode::AccountNotEnoughKeys);
        }

        Ok(())
    }
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize)]
pub struct InsertItems {
    pub key: Pubkey,
    pub value: ListValueUnpacked,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize)]
pub struct InsertItemsList {
    pub items: Vec<InsertItems>,
}

#[derive(Debug, Accounts)]
pub struct CreateMap<'info> {
    #[account(init, space = 8 + 1 + 32 + 4, payer = authority)]
    map_account: AccountLoader<'info, ExclusiveMapAccount>,
    /// CHECK: fine
    #[account(mut, signer)]
    authority: AccountInfo<'info>,
    system_program: Program<'info, System>,
}

#[derive(Debug, Accounts)]
pub struct UpdateItems<'info> {
    #[account(has_one = authority, mut)]
    map_account: AccountLoader<'info, ExclusiveMapAccount>,
    /// CHECK: fine
    #[account(mut, signer)]
    authority: AccountInfo<'info>,
    system_program: Program<'info, System>,
}

#[safe_zero_copy_account]
#[account(zero_copy)]
pub struct ExclusiveMapAccount {
    version: u8,
    authority: Pubkey,
}

#[safe_zero_copy]
#[zero_copy]
#[derive(Eq, PartialEq, Unpackable)]
pub struct ListValue {
    pub pubkey: Pubkey,
    pub byte: u8,
    pub long: u64,
}

impl WrappableAccount for ExclusiveMapAccount {
    type RemainingData = ExclusiveMap<Pubkey, ListValue, u32>;
}
