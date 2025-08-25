use star_frame::prelude::*;
use star_frame_spl::token::{state::MintAccount, Token};

use crate::state::{CreateMarketArgs, Market, MarketSeeds};

#[cfg(feature = "idl")]
use crate::state::FindMarketSeeds;

/// Initializes a marketplace for a given currency and market token
#[derive(InstructionArgs, BorshSerialize, BorshDeserialize, Copy, Clone, Debug)]
#[borsh(crate = "star_frame::borsh")]
pub struct Initialize;

#[derive(AccountSet, Debug)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub authority: Signer,
    pub currency: MintAccount,
    pub market_token: MintAccount,
    #[validate(arg = (
      Create(()),
      Seeds(MarketSeeds {
        currency: *self.currency.key_for(),
        market_token: *self.market_token.key_for()
      })
    ))]
    #[idl(
      arg = Seeds(FindMarketSeeds {
        currency: seed_path("currency"),
        market_token: seed_path("market_token")
      })
    )]
    pub market_account: Init<Seeded<Account<Market>>>,
    pub system_program: Program<System>,
    pub token_program: Program<Token>,
}

impl StarFrameInstruction for Initialize {
    type Accounts<'b, 'c> = InitializeAccounts;
    type ReturnType = ();

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        _: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        accounts
            .market_account
            .data_mut()?
            .initialize(CreateMarketArgs {
                authority: *accounts.authority.pubkey(),
                currency: *accounts.currency.key_for(),
                market_token: *accounts.market_token.key_for(),
                bump: accounts.market_account.access_seeds().bump,
            });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        state::{tests::default_market, MarketOwned},
        test_utils::new_mint_account,
    };
    use mollusk_svm::{result::Check, Mollusk};
    use solana_account::Account as SolanaAccount;
    use star_frame::{client::SerializeAccount, solana_pubkey::Pubkey};
    use std::{collections::HashMap, env};

    #[test]
    fn initialize_creates_market_account() -> Result<()> {
        if env::var("SBF_OUT_DIR").is_err() {
            println!("SBF_OUT_DIR is not set, skipping test");
            return Ok(());
        }

        let mut mollusk: Mollusk = Mollusk::new(&crate::Marketplace::ID, "marketplace");
        mollusk_svm_programs_token::token::add_program(&mut mollusk);

        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let currency = KeyFor::new(Pubkey::new_unique());
        let market_token = KeyFor::new(Pubkey::new_unique());

        let (market_pda, bump) =
            crate::state::Market::find_program_address(&crate::state::MarketSeeds {
                currency,
                market_token,
            });

        let account_store = HashMap::from_iter([
            (payer, SolanaAccount::new(1_000_000_000, 0, &System::ID)),
            (authority, SolanaAccount::new(1_000_000_000, 0, &System::ID)),
            (market_pda, SolanaAccount::default()),
            new_mint_account(currency),
            new_mint_account(market_token),
            mollusk_svm::program::keyed_account_for_system_program(),
        ]);
        let mollusk = mollusk.with_context(account_store);

        let ix = crate::Marketplace::instruction(
            &Initialize,
            InitializeClientAccounts {
                payer,
                authority,
                currency: *currency.pubkey(),
                market_token: *market_token.pubkey(),
                market_account: market_pda,
                token_program: None,
                system_program: None,
            },
        )?;

        let expected = MarketOwned {
            bump,
            authority,
            currency,
            market_token,
            ..default_market()
        };

        mollusk.process_and_validate_instruction(
            &ix,
            &[
                Check::success(),
                Check::account(&market_pda)
                    .owner(&crate::Marketplace::ID)
                    .data(&crate::state::Market::serialize_account(expected)?)
                    .build(),
            ],
        );

        Ok(())
    }
}
