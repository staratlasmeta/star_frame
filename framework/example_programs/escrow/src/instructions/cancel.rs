use crate::state::EscrowAccount;
use crate::utils::validate_token_account;
use star_frame::anyhow::bail;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;
use star_frame::solana_program::program::invoke_signed;
use star_frame::solana_program::program_pack::Pack;
use star_frame::solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(crate = "borsh")]
pub struct CancelIx {}

#[derive(AccountSet)]
#[validate(extra_validation = self.validate())]
pub struct CancelAccounts<'info> {
    pub maker: Signer<Writable<AccountInfo<'info>>>,
    pub maker_deposit_token_account: Writable<AccountInfo<'info>>,
    #[cleanup( arg = CloseAccount {
        recipient: &self.maker,
    })]
    pub escrow: Writable<DataAccount<'info, EscrowAccount>>,
    pub escrow_token_account: Writable<AccountInfo<'info>>,
    pub token_program: AccountInfo<'info>,
}

impl<'info> CancelAccounts<'info> {
    fn validate(&self) -> Result<()> {
        let escrow_data = self.escrow.data()?;
        let token_program_id = spl_token::ID;
        if self.maker.key() != &escrow_data.maker {
            bail!("Incorrect maker");
        }
        if self.token_program.key() != &token_program_id {
            bail!("Incorrect token program");
        }
        validate_token_account(
            &self.maker_deposit_token_account,
            &token_program_id,
            Some(self.maker.key()),
            None,
            None,
        )?;
        Ok(())
    }
}

impl StarFrameInstruction for CancelIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = ();
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = CancelAccounts<'info>
    where
        'info: 'b;

    fn split_to_args<'a>(_r: &Self) -> SplitToArgsReturn<Self> {
        SplitToArgsReturn {
            decode: (),
            cleanup: (),
            run: (),
            validate: (),
        }
    }

    fn run_instruction<'b, 'info>(
        _run_args: Self::RunArg<'_>,
        _program_id: &Pubkey,
        account_set: &mut Self::Accounts<'b, '_, 'info>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
        let escrow_data = account_set.escrow.data()?;

        // let account_seeds = EscrowAccountSeeds {
        //     maker: escrow_data.maker,
        //     maker_deposit_token_account: escrow_data.maker_deposit_token_account,
        //     exchange_mint: escrow_data.exchange_mint,
        // }; ?????

        let signer_seeds = [
            b"ESCROW",
            escrow_data.maker.as_ref(),
            escrow_data.maker_deposit_token_account.as_ref(),
            escrow_data.exchange_mint.as_ref(),
            &[escrow_data.bump],
        ];

        // transfer to taker
        let token_data = spl_token::state::Account::unpack(
            &account_set.escrow_token_account.try_borrow_data()?,
        )?;
        assert!(
            token_data.amount >= escrow_data.maker_amount,
            "Insufficient maker amount"
        );
        invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                account_set.escrow_token_account.key(),
                account_set.maker_deposit_token_account.key(),
                account_set.escrow.key(),
                &[],
                token_data.amount,
            )?,
            &[
                account_set.escrow_token_account.account_info_cloned(),
                account_set
                    .maker_deposit_token_account
                    .account_info_cloned(),
                account_set.escrow.account_info_cloned(),
                account_set.token_program.account_info_cloned(),
            ],
            &[&signer_seeds],
        )?;

        // close escrow token
        invoke_signed(
            &spl_token::instruction::close_account(
                &spl_token::ID,
                account_set.escrow_token_account.key(),
                account_set.maker.key(),
                account_set.escrow.key(),
                &[],
            )?,
            &[
                account_set.escrow_token_account.account_info_cloned(),
                account_set.maker.account_info_cloned(),
                account_set.escrow.account_info_cloned(),
                account_set.token_program.account_info_cloned(),
            ],
            &[&signer_seeds],
        )?;

        Ok(())
    }
}
