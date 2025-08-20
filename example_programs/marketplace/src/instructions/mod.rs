mod cancel_orders;
mod initialize;
mod place_order;

pub use cancel_orders::*;
pub use initialize::*;
pub use place_order::*;

use star_frame::prelude::*;
#[cfg(feature = "idl")]
use star_frame_spl::associated_token::FindAtaSeeds;
use star_frame_spl::{
    associated_token::state::{AssociatedTokenAccount, ValidateAta},
    token::{
        instructions::{Transfer, TransferCpiAccounts},
        state::{MintAccount, TokenAccount, ValidateToken},
        Token,
    },
};

#[cfg(feature = "idl")]
use crate::state::FindMarketSeeds;
use crate::state::{
    Market, MarketSeeds, OrderTotals, ValidateCurrency, ValidateMarketToken, ZERO_PRICE,
    ZERO_QUANTITY,
};

/// Accounts for managing market orders. Used in [`PlaceOrder`] and [`CancelOrders`] instructions.
#[derive(AccountSet, Debug)]
pub struct ManageOrderAccounts {
    #[validate(funder)]
    pub funder: Mut<Signer<SystemAccount>>,
    pub user: Signer<AccountInfo>,
    #[idl(arg = Seeds(FindMarketSeeds {
        currency: seed_path("currency"),
        market_token: seed_path("market_token")
    }))]
    #[validate(arg = (
        ValidateCurrency(self.currency.key_for()),
        ValidateMarketToken(self.market_token.key_for())
    ))]
    #[cleanup(arg = NormalizeRent(()))]
    pub market: Mut<ValidatedAccount<Market>>,
    pub currency: MintAccount,
    pub market_token: MintAccount,
    #[validate(arg = ValidateAta { mint: self.market_token.key_for(), wallet: self.market.pubkey()})]
    #[idl(arg = Seeds(FindAtaSeeds{ mint: seed_path("market_token"), wallet: seed_path("market") }))]
    pub market_token_vault: Mut<AssociatedTokenAccount>,
    #[validate(arg = ValidateAta { mint: self.currency.key_for(), wallet: self.market.pubkey()})]
    #[idl(arg = Seeds(FindAtaSeeds{ mint: seed_path("currency"), wallet: seed_path("market") }))]
    pub currency_vault: Mut<AssociatedTokenAccount>,
    #[validate(arg = ValidateToken { mint: Some(*self.market_token.key_for()), owner: Some(*self.user.pubkey())})]
    #[idl(arg = Seeds(FindAtaSeeds{ mint: seed_path("market_token"), wallet: seed_path("user") }))]
    pub user_market_token_vault: Mut<TokenAccount>,
    #[validate(arg = ValidateToken { mint: Some(*self.currency.key_for()), owner: Some(*self.user.pubkey())})]
    #[idl(arg = Seeds(FindAtaSeeds{ mint: seed_path("currency"), wallet: seed_path("user") }))]
    pub user_currency_vault: Mut<TokenAccount>,
    pub token_program: Program<Token>,
}

impl ManageOrderAccounts {
    fn withdraw(&self, totals: OrderTotals, ctx: &Context) -> Result<()> {
        let OrderTotals {
            market_tokens,
            currency,
        } = totals;
        let signer_seeds = if market_tokens > ZERO_QUANTITY || currency > ZERO_PRICE {
            let market = self.market.data()?;
            let seeds_with_bump = SeedsWithBump {
                seeds: MarketSeeds {
                    currency: *self.currency.key_for(),
                    market_token: *self.market_token.key_for(),
                },
                bump: market.bump,
            };
            Some(seeds_with_bump)
        } else {
            None
        };
        let signer_seeds = signer_seeds.as_ref().map(|seeds| seeds.seeds_with_bump());
        if market_tokens > ZERO_QUANTITY {
            Token::cpi(
                &Transfer {
                    amount: market_tokens.val().0,
                },
                TransferCpiAccounts {
                    source: *self.market_token_vault.account_info(),
                    destination: *self.user_market_token_vault.account_info(),
                    owner: *self.market.account_info(),
                },
                ctx,
            )?
            .invoke_signed(&[signer_seeds.as_ref().unwrap().as_slice()])?;
        }
        if currency > ZERO_PRICE {
            Token::cpi(
                &Transfer {
                    amount: currency.val().0,
                },
                TransferCpiAccounts {
                    source: *self.currency_vault.account_info(),
                    destination: *self.user_currency_vault.account_info(),
                    owner: *self.market.account_info(),
                },
                ctx,
            )?
            .invoke_signed(&[signer_seeds.as_ref().unwrap().as_slice()])?;
        }
        Ok(())
    }

    fn deposit(&self, totals: OrderTotals, ctx: &Context) -> Result<()> {
        let OrderTotals {
            market_tokens,
            currency,
        } = totals;
        if market_tokens > ZERO_QUANTITY {
            Token::cpi(
                &Transfer {
                    amount: market_tokens.val().0,
                },
                TransferCpiAccounts {
                    source: *self.user_market_token_vault.account_info(),
                    destination: *self.market_token_vault.account_info(),
                    owner: *self.user.account_info(),
                },
                ctx,
            )?
            .invoke()?;
        }
        if currency > ZERO_PRICE {
            Token::cpi(
                &Transfer {
                    amount: currency.val().0,
                },
                TransferCpiAccounts {
                    source: *self.user_currency_vault.account_info(),
                    destination: *self.currency_vault.account_info(),
                    owner: *self.user.account_info(),
                },
                ctx,
            )?
            .invoke()?;
        }
        Ok(())
    }
}
