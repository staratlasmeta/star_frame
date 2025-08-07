use star_frame::{
    borsh::{BorshDeserialize, BorshSerialize},
    prelude::*,
};
use star_frame_spl::token::{state::MintAccount, Token};

use crate::state::{CreateMarketArgs, FindMarketSeeds, Market, MarketSeeds};

#[derive(InstructionArgs, BorshSerialize, BorshDeserialize, Copy, Clone, Debug)]
#[borsh(crate = "star_frame::borsh")]
pub struct Initialize;

#[derive(AccountSet, Debug)]
pub struct InitializeAccounts {
    #[account_set(funder)]
    pub payer: Mut<Signer<SystemAccount>>,
    pub authority: Signer<AccountInfo>,
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
    pub token_program: Program<Token>,
}

impl StarFrameInstruction for Initialize {
    type Accounts<'b, 'c> = InitializeAccounts;
    type ReturnType = ();

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        _: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        account_set
            .market_account
            .data_mut()?
            .initialize(CreateMarketArgs {
                authority: *account_set.authority.pubkey(),
                currency: *account_set.currency.key_for(),
                market_token: *account_set.market_token.key_for(),
                bump: account_set.market_account.access_seeds().bump,
            });
        Ok(())
    }
}
