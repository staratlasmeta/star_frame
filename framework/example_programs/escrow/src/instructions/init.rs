use crate::state::{EscrowAccount, EscrowAccountSeeds};
use crate::utils::{validate_token_account, validate_token_mint};
use star_frame::anyhow::bail;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct InitEscrowIx {
    pub maker_amount: u64,
    pub taker_amount: u64,
}

#[derive(AccountSet)]
#[validate(arg = u64, extra_validation = self.validate(arg))]
pub struct InitEscrowAccounts<'info> {
    pub funder: Signer<Writable<SystemAccount<'info>>>,
    pub maker: Signer<SystemAccount<'info>>,
    pub maker_deposit_token_account: Writable<AccountInfo<'info>>,
    pub maker_receive_token_account: AccountInfo<'info>,
    pub escrow_deposit_token_account: Writable<AccountInfo<'info>>,
    pub exchange_token_mint: AccountInfo<'info>,
    #[validate(
        arg = Create(
            SeededInit {
                seeds: EscrowAccountSeeds {
                    maker: *self.maker.key(),
                    maker_deposit_token_account: *self.maker_deposit_token_account.key(),
                    exchange_mint: *self.exchange_token_mint.key(),
                },
            init_create: CreateAccount::new(&self.system_program, &self.funder),
        })
    )]
    pub escrow: SeededInitAccount<'info, EscrowAccount>,
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, SystemProgram>,
}

impl<'info> InitEscrowAccounts<'info> {
    fn validate(&self, maker_amount: u64) -> Result<()> {
        let token_program_id = spl_token::ID;
        if self.token_program.key() != &token_program_id {
            bail!("Incorrect token program");
        }
        validate_token_account(
            &self.maker_deposit_token_account,
            &token_program_id,
            Some(self.maker.key()),
            Some(maker_amount),
            None,
        )?;
        validate_token_account(
            &self.maker_receive_token_account,
            &token_program_id,
            Some(self.maker.key()),
            None,
            Some(self.exchange_token_mint.key()),
        )?;
        validate_token_account(
            &self.escrow_deposit_token_account,
            &token_program_id,
            Some(self.escrow.key()),
            None,
            None,
        )?;
        validate_token_mint(&self.exchange_token_mint, &token_program_id)?;
        Ok(())
    }
}

impl StarFrameInstruction for InitEscrowIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = u64;
    type RunArg<'a> = (u64, u64);
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = InitEscrowAccounts<'info>
    where
        'info: 'b;

    fn split_to_args<'a>(r: &Self) -> SplitToArgsReturn<Self> {
        SplitToArgsReturn {
            decode: (),
            cleanup: (),
            run: (r.maker_amount, r.taker_amount),
            validate: r.maker_amount,
        }
    }

    fn run_instruction<'b, 'info>(
        account_set: &mut Self::Accounts<'b, '_, 'info>,
        (maker_amount, taker_amount): Self::RunArg<'_>,
        syscalls: &mut impl SyscallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
        *account_set.escrow.data_mut()? = EscrowAccount {
            version: 0,
            maker: *account_set.maker.key(),
            maker_deposit_token_account: *account_set.maker_deposit_token_account.key(),
            maker_receive_token_account: *account_set.maker_receive_token_account.key(),
            escrow_token_account: *account_set.escrow_deposit_token_account.key(),
            exchange_mint: *account_set.exchange_token_mint.key(),
            maker_amount,
            taker_amount,
            bump: account_set.escrow.access_seeds().bump,
        };

        syscalls.invoke(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                account_set.maker_deposit_token_account.key(),
                account_set.escrow_deposit_token_account.key(),
                account_set.maker.key(),
                &[],
                maker_amount,
            )?,
            &[
                account_set
                    .maker_deposit_token_account
                    .account_info_cloned(),
                account_set
                    .escrow_deposit_token_account
                    .account_info_cloned(),
                account_set.maker.account_info_cloned(),
                account_set.token_program.account_info_cloned(),
            ],
        )?;

        Ok(())
    }
}
