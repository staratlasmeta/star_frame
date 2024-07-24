use solana_program::pubkey::Pubkey;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

#[derive(Copy, Clone, Align1, Debug, Pod, Zeroable)]
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
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant =
        [47, 44, 255, 15, 103, 77, 139, 247];
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
// #[decode(arg = usize)]
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
    pub profile: SeededInitAccount<'info, CounterAccount>,
    pub system_program: Program<'info, SystemProgram>,
}

impl StarFrameInstruction for CreateCounterIx {
    type SelfData<'a> = Self;
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = &'a Option<u64>;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = CreateCounterAccounts<'info>
    where
        'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        Self::deserialize(bytes).map_err(Into::into)
    }

    fn split_to_args(
        r: &Self,
    ) -> (
        Self::DecodeArg<'_>,
        Self::ValidateArg<'_>,
        Self::RunArg<'_>,
        Self::CleanupArg<'_>,
    ) {
        ((), (), &r.start_at, ())
    }

    fn run_instruction<'b, 'info>(
        start_at: Self::RunArg<'_>,
        _program_id: &Pubkey,
        _account_set: &mut Self::Accounts<'b, '_, 'info>,
        _sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
        msg!("start_at >> {:?}", start_at);
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateCounterIx {}

#[derive(AccountSet, Debug)]
pub struct UpdateCounterAccounts<'info> {
    pub old_owner: Signer<Writable<AccountInfo<'info>>>,
    pub new_owner: AccountInfo<'info>,
    pub profile: Writable<DataAccount<'info, CounterAccount>>,
}

#[derive(Debug)]
pub struct CountIx {
    pub amount: Option<u64>,
    pub subtract: bool,
}

#[derive(AccountSet, Debug)]
pub struct CountAccounts<'info> {
    pub owner: Signer<Writable<AccountInfo<'info>>>,
    pub profile: Writable<DataAccount<'info, CounterAccount>>,
}

#[star_frame_instruction_set]
pub enum CounterInstructionSet {
    CreateCounter(CreateCounterIx),
}

#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet,
    id =  "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG",
)]
pub struct CounterProgram {}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    star_frame::entrypoint::try_star_frame_entrypoint::<CounterProgram>(
        program_id,
        accounts,
        instruction_data,
    )
    .map_err(star_frame::errors::handle_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::instruction::Instruction as SolanaInstruction;
    use solana_sdk::signature::Signer;
    use solana_sdk::system_program;
    use solana_sdk::transaction::Transaction;
    use star_frame::solana_program::instruction::AccountMeta;

    #[tokio::test]
    async fn test_validate() {
        // Initialize the program test environment
        let program_test = ProgramTest::new(
            "counter",
            CounterProgram::PROGRAM_ID,
            processor!(process_instruction),
        );

        // Add accounts to the context
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Prepare the instruction data
        let account_key = Pubkey::new_unique();
        let _account_key2 = Pubkey::new_unique();
        let start_at = Some(2u64);
        // Create the instruction
        let seeds = CounterAccountSeeds {
            owner: account_key,
        };
        let (counter_account, _bump) =
            Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::PROGRAM_ID);
        let _ix_data = CreateCounterIx { start_at };
        let instruction = SolanaInstruction::new_with_borsh(
            CounterProgram::PROGRAM_ID,
            &[0, 1],
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(account_key, false),
                AccountMeta::new(counter_account, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
        );

        // Create and send the transaction
        let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
        transaction.sign(&[&payer], recent_blockhash);

        // Process the transaction
        banks_client.process_transaction(transaction).await.unwrap();
        // let xxx = banks_client.process_transaction(transaction).await;
        // msg!("xxx >> {:?}", xxx);
    }
}
