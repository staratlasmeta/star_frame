use star_frame::anyhow::bail;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::derive_more::{self, Deref, DerefMut};
use star_frame::empty_star_frame_instruction;
use star_frame::prelude::*;
use star_frame::solana_program::pubkey::Pubkey;

#[derive(Align1, Pod, Zeroable, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = CounterAccountSeeds)]
#[repr(C, packed)]
pub struct CounterAccount {
    pub version: u8,
    pub owner: Pubkey,
    pub signer: Pubkey,
    pub count: u64,
    pub bump: u8,
}

#[derive(AccountSet, Deref, DerefMut, Debug)]
pub struct WrappedCounter<'info>(#[single_account_set] DataAccount<'info, CounterAccount>);

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"COUNTER")]
pub struct CounterAccountSeeds {
    pub owner: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionToIdl)]
pub struct CreateCounterIx {
    pub start_at: Option<u64>,
}

#[derive(AccountSet)]
pub struct CreateCounterAccounts<'info> {
    #[account_set(funder)]
    pub funder: Signer<Mut<SystemAccount<'info>>>,
    pub owner: SystemAccount<'info>,
    #[validate(arg = (
        CreateIfNeeded(()),
        Seeds(CounterAccountSeeds { owner: *self.owner.key() }),
    ))]
    #[idl(arg = Seeds(FindCounterAccountSeeds { owner: seed_path("owner") }))]
    pub counter: Init<Seeded<WrappedCounter<'info>>>,
    pub system_program: Program<'info, SystemProgram>,
}

impl StarFrameInstruction for CreateCounterIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = &'a Option<u64>;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = CreateCounterAccounts<'info>;

    fn split_to_args<'a>(r: &mut Self) -> IxArgs<Self> {
        IxArgs::run(&r.start_at)
    }

    fn run_instruction<'info>(
        account_set: &mut Self::Accounts<'_, '_, 'info>,
        start_at: Self::RunArg<'_>,
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self::ReturnType> {
        *account_set.counter.data_mut()? = CounterAccount {
            version: 0,
            signer: *account_set.owner.key(),
            owner: *account_set.owner.key(),
            bump: account_set.counter.access_seeds().bump,
            count: start_at.unwrap_or(0),
        };

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionToIdl)]
pub struct UpdateCounterSignerIx;

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct UpdateCounterSignerAccounts<'info> {
    pub signer: Signer<SystemAccount<'info>>,
    pub new_signer: SystemAccount<'info>,
    pub counter: Mut<DataAccount<'info, CounterAccount>>,
}

impl<'info> UpdateCounterSignerAccounts<'info> {
    fn validate(&self) -> Result<()> {
        if *self.signer.key() != self.counter.data()?.signer {
            bail!("Incorrect signer");
        }
        Ok(())
    }
}

impl StarFrameInstruction for UpdateCounterSignerIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = ();
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = UpdateCounterSignerAccounts<'info>;

    fn split_to_args<'a>(_r: &mut Self) -> IxArgs<Self> {
        IxArgs::default()
    }

    fn run_instruction<'info>(
        account_set: &mut Self::Accounts<'_, '_, 'info>,
        _run_args: Self::RunArg<'_>,
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self::ReturnType> {
        let mut counter = account_set.counter.data_mut()?;
        counter.signer = *account_set.new_signer.key();

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionToIdl)]
pub struct CountIx {
    pub amount: u64,
    pub subtract: bool,
}

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct CountAccounts<'info> {
    pub owner: Signer<SystemAccount<'info>>,
    pub counter: Mut<DataAccount<'info, CounterAccount>>,
}

impl<'info> CountAccounts<'info> {
    fn validate(&self) -> Result<()> {
        if *self.owner.key() != self.counter.data()?.owner {
            bail!("Incorrect owner");
        }
        Ok(())
    }
}

impl StarFrameInstruction for CountIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = (u64, bool);
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = CountAccounts<'info>;

    fn split_to_args<'a>(r: &mut Self) -> IxArgs<Self> {
        IxArgs::run((r.amount, r.subtract))
    }

    fn run_instruction<'info>(
        account_set: &mut Self::Accounts<'_, '_, 'info>,
        (amount, subtract): Self::RunArg<'_>,
        _syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self::ReturnType> {
        let mut counter = account_set.counter.data_mut()?;
        let new_count: u64 = if subtract {
            counter.count - amount
        } else {
            counter.count + amount
        };
        counter.count = new_count;

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionToIdl)]
pub struct CloseCounterIx;

#[derive(AccountSet, Debug)]
pub struct CloseCounterAccounts<'info> {
    #[validate(arg = &self.counter.data()?.signer)]
    pub signer: Signer<SystemAccount<'info>>,
    #[account_set(recipient)]
    pub funds_to: Mut<SystemAccount<'info>>,
    #[cleanup(arg = CloseAccount(()))]
    pub counter: Mut<WrappedCounter<'info>>,
}
empty_star_frame_instruction!(CloseCounterIx, CloseCounterAccounts);

#[derive(InstructionSet)]
pub enum CounterInstructionSet {
    CreateCounter(CreateCounterIx),
    UpdateSigner(UpdateCounterSignerIx),
    Count(CountIx),
    CloseCounter(CloseCounterIx),
}

#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet,
    id = "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG",
)]
pub struct CounterProgram;

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::checked::try_from_bytes;
    use solana_program_test::*;
    use solana_sdk::signature::{Keypair, Signer};
    use solana_sdk::transaction::Transaction;
    use star_frame::serde_json;

    #[test]
    fn idl_test() {
        let idl = CounterProgram::program_to_idl().unwrap();
        println!("{}", serde_json::to_string_pretty(&idl).unwrap());
    }

    #[tokio::test]
    async fn test_that_it_works() -> Result<()> {
        let program_test = ProgramTest::new(
            "counter",
            CounterProgram::PROGRAM_ID,
            processor!(CounterProgram::processor),
        );
        let (banks_client, payer, recent_blockhash) = program_test.start().await;

        // Init a new counter
        let account_key = Keypair::new();
        let account_key2 = Keypair::new();
        let start_at = Some(2u64);
        let seeds = CounterAccountSeeds {
            owner: account_key.pubkey(),
        };
        let (counter_account, bump) =
            Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::PROGRAM_ID);

        let instruction = CounterProgram::instruction(
            &CreateCounterIx { start_at },
            CreateCounterClientAccounts {
                funder: payer.pubkey(),
                owner: account_key.pubkey(),
                counter: counter_account,
                system_program: SystemProgram::PROGRAM_ID,
            },
        )?;

        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await?;
        let expected = CounterAccount {
            version: 0,
            owner: account_key.pubkey(),
            signer: account_key.pubkey(),
            count: 2,
            bump,
        };
        let acc = banks_client.get_account(counter_account).await?.unwrap();
        assert_eq!(expected, *try_from_bytes(&acc.data[8..])?);

        // Update a counter signer
        let instruction2 = CounterProgram::instruction(
            &UpdateCounterSignerIx,
            UpdateCounterSignerClientAccounts {
                signer: account_key.pubkey(),
                new_signer: account_key2.pubkey(),
                counter: counter_account,
            },
        )?;
        let mut transaction2 = Transaction::new_with_payer(&[instruction2], Some(&payer.pubkey()));
        transaction2.sign(&[&payer, &account_key], recent_blockhash);
        banks_client.process_transaction(transaction2).await?;
        let acc2 = banks_client.get_account(counter_account).await?.unwrap();
        let acc2_data: CounterAccount = *try_from_bytes(&acc2.data[8..])?;
        assert_eq!(acc2_data.signer, account_key2.pubkey());

        // Update count
        let instruction3 = CounterProgram::instruction(
            &CountIx {
                amount: 7,
                subtract: false,
            },
            CountClientAccounts {
                owner: account_key.pubkey(),
                counter: counter_account,
            },
        )?;
        let instruction4 = CounterProgram::instruction(
            &CountIx {
                amount: 4,
                subtract: true,
            },
            CountClientAccounts {
                owner: account_key.pubkey(),
                counter: counter_account,
            },
        )?;

        let mut transaction3 =
            Transaction::new_with_payer(&[instruction3, instruction4], Some(&payer.pubkey()));
        transaction3.sign(&[&payer, &account_key], recent_blockhash);
        banks_client.process_transaction(transaction3).await?;
        let acc3 = banks_client.get_account(counter_account).await?.unwrap();
        let acc3_data: CounterAccount = *try_from_bytes(&acc3.data[8..])?;
        let old_count = acc2_data.count;
        let new_count = acc3_data.count;
        assert_eq!(new_count, old_count + 7 - 4);

        // Close counter
        let refund_acc = banks_client.get_account(account_key.pubkey()).await?;
        assert!(refund_acc.is_none());
        let instruction5 = CounterProgram::instruction(
            &CloseCounterIx,
            CloseCounterClientAccounts {
                signer: account_key2.pubkey(),
                funds_to: account_key.pubkey(),
                counter: counter_account,
            },
        )?;

        let mut transaction5 = Transaction::new_with_payer(&[instruction5], Some(&payer.pubkey()));
        transaction5.sign(&[&payer, &account_key2], recent_blockhash);
        banks_client.process_transaction(transaction5).await?;
        let acc5 = banks_client.get_account(counter_account).await?;
        assert!(acc5.is_none());
        let refund_acc2 = banks_client
            .get_account(account_key.pubkey())
            .await?
            .unwrap();
        assert_eq!(refund_acc2.lamports, acc3.lamports);
        Ok(())
    }
}
