use solana_program::pubkey::Pubkey;
use star_frame::anyhow::bail;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

#[derive(Align1, Copy, Clone, Debug, Eq, PartialEq, Pod, Zeroable)]
#[repr(C, packed)]
pub struct CounterAccount {
    pub version: u8,
    pub owner: Pubkey,
    pub signer: Pubkey,
    pub count: u64,
    pub bump: u8,
}

impl ProgramAccount for CounterAccount {
    type OwnerProgram = CounterProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant = [0; 8];
}

impl SeededAccountData for CounterAccount {
    type Seeds = CounterAccountSeeds;
}

#[derive(Debug, GetSeeds)]
#[seed_const(b"COUNTER")]
pub struct CounterAccountSeeds {
    pub owner: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(crate = "borsh")]
pub struct CreateCounterIx {
    pub start_at: Option<u64>,
}

#[derive(AccountSet)]
pub struct CreateCounterAccounts<'info> {
    pub funder: Signer<Writable<AccountInfo<'info>>>,
    pub owner: AccountInfo<'info>,
    #[validate(
        arg = Create(
            SeededInit {
                seeds: CounterAccountSeeds {
                owner: *self.owner.key(),
            },
            init_create: CreateAccount::new(&self.system_program, &self.funder),
        })
    )]
    pub counter: SeededInitAccount<'info, CounterAccount>,
    pub system_program: Program<'info, SystemProgram>,
}

impl StarFrameInstruction for CreateCounterIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = &'a Option<u64>;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = CreateCounterAccounts<'info>
    where
        'info: 'b;

    fn split_to_args<'a>(r: &Self) -> SplitToArgsReturn<Self> {
        SplitToArgsReturn {
            decode: (),
            cleanup: (),
            run: &r.start_at,
            validate: (),
        }
    }

    fn run_instruction<'b, 'info>(
        start_at: Self::RunArg<'_>,
        _program_id: &Pubkey,
        account_set: &mut Self::Accounts<'b, '_, 'info>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(crate = "borsh")]
pub struct UpdateCounterSignerIx {}

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct UpdateCounterSignerAccounts<'info> {
    pub signer: Signer<AccountInfo<'info>>,
    pub new_signer: AccountInfo<'info>,
    pub counter: Writable<DataAccount<'info, CounterAccount>>,
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
    type Accounts<'b, 'c, 'info> = UpdateCounterSignerAccounts<'info>
    where
        'info: 'b;

    fn split_to_args<'a>(_r: &Self) -> SplitToArgsReturn<Self> {
        SplitToArgsReturn {
            ..Default::default()
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
        let mut counter = account_set.counter.data_mut()?;
        counter.signer = *account_set.new_signer.key();

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(crate = "borsh")]
pub struct CountIx {
    pub amount: u64,
    pub subtract: bool,
}

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct CountAccounts<'info> {
    pub owner: Signer<AccountInfo<'info>>,
    pub counter: Writable<DataAccount<'info, CounterAccount>>,
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
    type Accounts<'b, 'c, 'info> = CountAccounts<'info>
    where
        'info: 'b;

    fn split_to_args<'a>(r: &Self) -> SplitToArgsReturn<Self> {
        SplitToArgsReturn {
            run: (r.amount, r.subtract),
            ..Default::default()
        }
    }

    fn run_instruction<'b, 'info>(
        (amount, subtract): Self::RunArg<'_>,
        _program_id: &Pubkey,
        account_set: &mut Self::Accounts<'b, '_, 'info>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[borsh(crate = "borsh")]
pub struct CloseCounterIx {}

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct CloseCounterAccounts<'info> {
    pub signer: Signer<AccountInfo<'info>>,
    pub funds_to: Writable<AccountInfo<'info>>,
    #[cleanup( arg = CloseAccount {
        recipient: &self.funds_to,
    })]
    pub counter: Writable<DataAccount<'info, CounterAccount>>,
}

impl<'info> CloseCounterAccounts<'info> {
    fn validate(&self) -> Result<()> {
        if *self.signer.key() != self.counter.data()?.signer {
            println!(
                "e > {:?} | r > {:?}",
                *self.signer.key(),
                self.counter.data()?.signer
            );
            bail!("Incorrect signer");
        }
        Ok(())
    }
}

impl StarFrameInstruction for CloseCounterIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = ();
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = CloseCounterAccounts<'info>
    where
        'info: 'b;

    fn split_to_args<'a>(_r: &Self) -> SplitToArgsReturn<Self> {
        SplitToArgsReturn {
            ..Default::default()
        }
    }

    fn run_instruction<'b, 'info>(
        _run_args: Self::RunArg<'_>,
        _program_id: &Pubkey,
        _account_set: &mut Self::Accounts<'b, '_, 'info>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
        Ok(())
    }
}

#[star_frame_instruction_set]
// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// #[borsh(crate = "borsh", use_discriminant = false)]
pub enum CounterInstructionSet {
    CreateCounter(CreateCounterIx),
    UpdateSigner(UpdateCounterSignerIx),
    Count(CountIx),
    CloseCounter(CloseCounterIx),
}

#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet<'static>,
    id =  "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG",
)]
pub struct CounterProgram {}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::to_vec;
    use bytemuck::checked::try_from_bytes;
    use solana_program_test::*;
    use solana_sdk::instruction::Instruction as SolanaInstruction;
    use solana_sdk::signature::{Keypair, Signer};
    use solana_sdk::system_program;
    use solana_sdk::transaction::Transaction;
    use star_frame::itertools::Itertools;
    use star_frame::solana_program::instruction::AccountMeta;

    #[tokio::test]
    async fn test_that_it_works() {
        let program_test = ProgramTest::new(
            "counter",
            CounterProgram::PROGRAM_ID,
            processor!(CounterProgram::processor),
        );
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Init a new counter
        let account_key = Keypair::new();
        let account_key2 = Keypair::new();
        let start_at = Some(2u64);
        let seeds = CounterAccountSeeds {
            owner: account_key.pubkey(),
        };
        let (counter_account, bump) =
            Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::PROGRAM_ID);
        let ix_data = [
            CreateCounterIx::DISCRIMINANT.to_vec(),
            to_vec(&CreateCounterIx { start_at }).unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let instruction = SolanaInstruction::new_with_bytes(
            CounterProgram::PROGRAM_ID,
            &ix_data,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(account_key.pubkey(), false),
                AccountMeta::new(counter_account, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
        );
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();
        let expected = CounterAccount {
            version: 0,
            owner: account_key.pubkey(),
            signer: account_key.pubkey(),
            count: 2,
            bump,
        };
        let acc = banks_client
            .get_account(counter_account)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(expected, *try_from_bytes(&acc.data[8..]).unwrap());

        // Update a counter signer
        let ix_data2 = [
            UpdateCounterSignerIx::DISCRIMINANT.to_vec(),
            to_vec(&UpdateCounterSignerIx {}).unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let instruction2 = SolanaInstruction::new_with_bytes(
            CounterProgram::PROGRAM_ID,
            &ix_data2,
            vec![
                AccountMeta::new_readonly(account_key.pubkey(), true),
                AccountMeta::new_readonly(account_key2.pubkey(), false),
                AccountMeta::new(counter_account, false),
            ],
        );
        let mut transaction2 = Transaction::new_with_payer(&[instruction2], Some(&payer.pubkey()));
        transaction2.sign(&[&payer, &account_key], recent_blockhash);
        banks_client
            .process_transaction(transaction2)
            .await
            .unwrap();
        let acc2 = banks_client
            .get_account(counter_account)
            .await
            .unwrap()
            .unwrap();
        let acc2_data: CounterAccount = *try_from_bytes(&acc2.data[8..]).unwrap();
        assert_eq!(acc2_data.signer, account_key2.pubkey());

        // Update count
        let count_accounts: Vec<AccountMeta> = vec![
            AccountMeta::new_readonly(account_key.pubkey(), true),
            AccountMeta::new(counter_account, false),
        ];
        let ix_data3 = [
            CountIx::DISCRIMINANT.to_vec(),
            to_vec(&CountIx {
                amount: 7,
                subtract: false,
            })
            .unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let ix_data4 = [
            CountIx::DISCRIMINANT.to_vec(),
            to_vec(&CountIx {
                amount: 4,
                subtract: true,
            })
            .unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let instruction3 = SolanaInstruction::new_with_bytes(
            CounterProgram::PROGRAM_ID,
            &ix_data3,
            count_accounts.clone(),
        );
        let instruction4 = SolanaInstruction::new_with_bytes(
            CounterProgram::PROGRAM_ID,
            &ix_data4,
            count_accounts.clone(),
        );
        let mut transaction3 =
            Transaction::new_with_payer(&[instruction3, instruction4], Some(&payer.pubkey()));
        transaction3.sign(&[&payer, &account_key], recent_blockhash);
        banks_client
            .process_transaction(transaction3)
            .await
            .unwrap();
        let acc3 = banks_client
            .get_account(counter_account)
            .await
            .unwrap()
            .unwrap();
        let acc3_data: CounterAccount = *try_from_bytes(&acc3.data[8..]).unwrap();
        let old_count = acc2_data.count;
        let new_count = acc3_data.count;
        assert_eq!(new_count, old_count + 7 - 4);

        // Close counter
        let refund_acc = banks_client
            .get_account(account_key.pubkey())
            .await
            .unwrap();
        assert!(refund_acc.is_none());
        let ix_data5 = [
            CloseCounterIx::DISCRIMINANT.to_vec(),
            to_vec(&CloseCounterIx {}).unwrap(),
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let instruction5 = SolanaInstruction::new_with_bytes(
            CounterProgram::PROGRAM_ID,
            &ix_data5,
            vec![
                AccountMeta::new_readonly(account_key2.pubkey(), true),
                AccountMeta::new(account_key.pubkey(), false),
                AccountMeta::new(counter_account, false),
            ],
        );
        let mut transaction5 = Transaction::new_with_payer(&[instruction5], Some(&payer.pubkey()));
        transaction5.sign(&[&payer, &account_key2], recent_blockhash);
        banks_client
            .process_transaction(transaction5)
            .await
            .unwrap();
        let acc5 = banks_client.get_account(counter_account).await.unwrap();
        assert!(acc5.is_none());
        let refund_acc2 = banks_client
            .get_account(account_key.pubkey())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(refund_acc2.lamports, acc3.lamports);
    }
}
