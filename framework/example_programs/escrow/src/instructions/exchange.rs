use crate::state::EscrowAccount;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;
use star_frame::solana_program::program_pack::Pack;
use star_frame::solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(crate = "borsh")]
pub struct ExchangeIx {}

#[derive(AccountSet)]
pub struct ExchangeAccounts<'info> {
    pub maker: Writable<AccountInfo<'info>>,
    pub maker_receive_token_account: Writable<AccountInfo<'info>>,
    pub taker: Signer<AccountInfo<'info>>,
    pub taker_deposit_token_account: Writable<AccountInfo<'info>>,
    pub taker_receive_token_account: Writable<AccountInfo<'info>>,
    #[cleanup( arg = CloseAccount {
        recipient: &self.maker,
    })]
    pub escrow: Writable<DataAccount<'info, EscrowAccount>>,
    pub escrow_token_account: Writable<AccountInfo<'info>>,
    pub token_mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

impl StarFrameInstruction for ExchangeIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = ();
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = ExchangeAccounts<'info>
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
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
        let escrow_data = account_set.escrow.data()?;

        // transfer to maker
        sys_calls.invoke(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                account_set.taker_deposit_token_account.key(),
                account_set.maker_receive_token_account.key(),
                account_set.taker.key(),
                &[],
                escrow_data.taker_amount,
            )?,
            &[
                account_set
                    .taker_deposit_token_account
                    .account_info_cloned(),
                account_set
                    .maker_receive_token_account
                    .account_info_cloned(),
                account_set.taker.account_info_cloned(),
                account_set.token_program.account_info_cloned(),
            ],
        )?;

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
        sys_calls.invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                account_set.escrow_token_account.key(),
                account_set.taker_receive_token_account.key(),
                account_set.escrow.key(),
                &[],
                token_data.amount,
            )?,
            &[
                account_set.escrow_token_account.account_info_cloned(),
                account_set
                    .taker_receive_token_account
                    .account_info_cloned(),
                account_set.escrow.account_info_cloned(),
                account_set.token_program.account_info_cloned(),
            ],
            &[&signer_seeds],
        )?;

        // close escrow token
        sys_calls.invoke_signed(
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
