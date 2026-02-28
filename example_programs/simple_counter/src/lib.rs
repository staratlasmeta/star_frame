use star_frame::prelude::*;

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

#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
#[program_account(seeds = CounterSeeds)]
pub struct CounterAccount {
    pub authority: Pubkey,
    pub count: u64,
}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"COUNTER")]
pub struct CounterSeeds {
    pub authority: Pubkey,
}

#[derive(Debug)]
pub struct Authority(Pubkey);

impl AccountValidate<Authority> for CounterAccount {
    fn validate_account(self_ref: &Self::Ptr, arg: Authority) -> Result<()> {
        ensure!(
            arg.0 == self_ref.authority,
            ProgramError::IncorrectAuthority
        );
        Ok(())
    }
}

/// Initialize the counter
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
        CounterSeeds { authority: *self.authority.pubkey() },
    ))]
    #[idl(arg = Seeds(FindCounterSeeds { authority: seed_path("authority") }))]
    pub counter: Init<Seeded<Account<CounterAccount>>>,
    pub system_program: Program<System>,
}

#[star_frame_instruction]
fn Initialize(account_set: &mut InitializeAccounts, start_at: &Option<u64>) -> Result<()> {
    **account_set.counter.data_mut()? = CounterAccount {
        authority: *account_set.authority.pubkey(),
        count: start_at.unwrap_or(0),
    };
    Ok(())
}

/// Increment the counter by 1
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
pub struct Increment;

#[derive(AccountSet, Debug)]
pub struct IncrementAccounts {
    pub authority: Signer,
    #[validate(arg = Authority(*self.authority.pubkey()))]
    pub counter: Mut<ValidatedAccount<CounterAccount>>,
}

#[star_frame_instruction]
fn Increment(account_set: &mut IncrementAccounts) -> Result<()> {
    let mut counter = account_set.counter.data_mut()?;
    counter.count += 1;
    Ok(())
}
