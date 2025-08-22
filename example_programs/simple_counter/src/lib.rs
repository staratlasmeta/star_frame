use star_frame::{
    anyhow::ensure,
    borsh::{BorshDeserialize, BorshSerialize},
    prelude::*,
    solana_pubkey::Pubkey,
};

#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet,
    id = "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG"
)]
pub struct CounterProgram;

#[derive(InstructionSet)]
pub enum CounterInstructionSet {
    Initialize(Initialize),
    Increment(Increment),
}

#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = CounterSeeds)]
#[repr(C, packed)]
pub struct CounterAccount {
    pub authority: Pubkey,
    pub count: u64,
}

impl AccountValidate<&Pubkey> for CounterAccount {
    fn validate(self_ref: &Self::Ref<'_>, arg: &Pubkey) -> Result<()> {
        ensure!(arg == &self_ref.authority, "Incorrect authority");
        Ok(())
    }
}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"COUNTER")]
pub struct CounterSeeds {
    pub authority: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize {
    #[ix_args(&run)]
    pub start_at: Option<u64>,
}

#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub authority: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(CounterSeeds { authority: *self.authority.pubkey() }),
    ))]
    #[idl(arg = Seeds(FindCounterSeeds { authority: seed_path("authority") }))]
    pub counter: Init<Seeded<Account<CounterAccount>>>,
    pub system_program: Program<System>,
}

impl StarFrameInstruction for Initialize {
    type ReturnType = ();
    type Accounts<'b, 'c> = InitializeAccounts;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        start_at: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        **account_set.counter.data_mut()? = CounterAccount {
            authority: *account_set.authority.pubkey(),
            count: start_at.unwrap_or(0),
        };

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
pub struct Increment {}

#[derive(AccountSet, Debug)]
pub struct IncrementAccounts {
    pub authority: Signer<SystemAccount>,
    #[validate(arg = self.authority.pubkey())]
    pub counter: Mut<ValidatedAccount<CounterAccount>>,
}

impl StarFrameInstruction for Increment {
    type ReturnType = ();
    type Accounts<'b, 'c> = IncrementAccounts;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        _run_args: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut counter = account_set.counter.data_mut()?;
        counter.count += 1;

        Ok(())
    }
}
