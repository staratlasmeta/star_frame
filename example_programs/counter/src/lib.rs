use star_frame::anyhow::bail;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::derive_more::{self, Deref, DerefMut};
use star_frame::empty_star_frame_instruction;
use star_frame::prelude::*;
use star_frame::solana_program::pubkey::Pubkey;

#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = CounterAccountSeeds)]
#[repr(C, packed)]
pub struct CounterAccount {
    pub version: u8,
    pub owner: Pubkey,
    pub signer: Pubkey,
    pub count: u64,
    pub bump: u8,
    pub data: CounterAccountData,
}

#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, TypeToIdl)]
#[repr(C, packed)]
pub struct CounterAccountData {
    pub version: u8,
    pub owner: Pubkey,
    pub signer: Pubkey,
    pub count: u64,
    pub bump: u8,
}

#[derive(AccountSet, Deref, DerefMut, Debug)]
pub struct WrappedCounter<'info>(#[single_account_set] Account<'info, CounterAccount>);

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
    pub system_program: Program<'info, System>,
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
        **account_set.counter.data_mut()? = CounterAccount {
            version: 0,
            signer: *account_set.owner.key(),
            owner: *account_set.owner.key(),
            bump: account_set.counter.access_seeds().bump,
            count: start_at.unwrap_or(0),
            data: Default::default(),
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
    pub counter: Mut<Account<'info, CounterAccount>>,
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
    pub counter: Mut<Account<'info, CounterAccount>>,
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
    #[validate(address = &self.counter.data()?.signer)]
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
    id = "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG"
)]
pub struct CounterProgram;

#[cfg(test)]
mod tests {
    use super::*;
    use codama_nodes::{NodeTrait, ProgramNode};
    use solana_program_test::*;
    use solana_sdk::signature::{Keypair, Signer};
    use solana_sdk::transaction::Transaction;
    use star_frame::client::DeserializeAccount;

    #[cfg(feature = "idl")]
    #[test]
    fn generate_idl() -> Result<()> {
        let idl = StarFrameDeclaredProgram::program_to_idl()?;
        let codama_idl: ProgramNode = idl.try_into()?;
        let idl_json = codama_idl.to_json()?;
        std::fs::write("idl.json", &idl_json)?;
        println!("{idl_json}");
        Ok(())
    }

    #[tokio::test]
    async fn test_that_it_works() -> Result<()> {
        let program_test = if option_env!("USE_BIN").is_some() {
            let target_dir = std::env::current_dir()?
                .join("../../target/deploy")
                .canonicalize()?;
            #[allow(unused_unsafe)]
            unsafe {
                std::env::set_var(
                    "BPF_OUT_DIR",
                    target_dir.to_str().expect("Failed to convert path to str"),
                );
            }
            ProgramTest::new("counter", StarFrameDeclaredProgram::ID, None)
        } else {
            ProgramTest::new(
                "counter",
                StarFrameDeclaredProgram::ID,
                processor!(CounterProgram::processor),
            )
        };
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Init a new counter
        let account_key = Keypair::new();
        let account_key2 = Keypair::new();
        let start_at = Some(2u64);
        let seeds = CounterAccountSeeds {
            owner: account_key.pubkey(),
        };
        let (counter_account, bump) =
            Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::ID);

        let instruction = CounterProgram::instruction(
            &CreateCounterIx { start_at },
            CreateCounterClientAccounts {
                funder: payer.pubkey(),
                owner: account_key.pubkey(),
                counter: counter_account,
                system_program: System::ID,
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
            data: Default::default(),
        };
        let acc = banks_client.get_account(counter_account).await?.unwrap();
        assert_eq!(expected, CounterAccount::deserialize_account(&acc.data)?);

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
        let acc2_data: CounterAccount = CounterAccount::deserialize_account(&acc2.data)?;
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
        let acc3_data: CounterAccount = CounterAccount::deserialize_account(&acc3.data)?;
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
