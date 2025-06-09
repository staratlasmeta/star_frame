use star_frame::anyhow::bail;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::derive_more::{self, Deref, DerefMut};
use star_frame::empty_star_frame_instruction;
use star_frame::prelude::*;
use star_frame::solana_pubkey::Pubkey;

#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet,
    id = "Coux9zxTFKZpRdFpE4F7Fs5RZ6FdaURdckwS61BUTMG"
)]
pub struct CounterProgram;

#[derive(InstructionSet)]
pub enum CounterInstructionSet {
    CreateCounter(CreateCounterIx),
    UpdateSigner(UpdateCounterSignerIx),
    Count(CountIx),
    CloseCounter(CloseCounterIx),
}

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
pub struct WrappedCounter(#[single_account_set] Account<CounterAccount>);

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"COUNTER")]
pub struct CounterAccountSeeds {
    pub owner: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct CreateCounterIx {
    #[ix_args(&run)]
    pub start_at: Option<u64>,
}

#[derive(AccountSet)]
pub struct CreateCounterAccounts {
    #[account_set(funder)]
    pub funder: Signer<Mut<SystemAccount>>,
    pub owner: SystemAccount,
    #[validate(arg = (
        CreateIfNeeded(()),
        Seeds(CounterAccountSeeds { owner: *self.owner.pubkey() }),
    ))]
    #[idl(arg = Seeds(FindCounterAccountSeeds { owner: seed_path("owner") }))]
    pub counter: Init<Seeded<WrappedCounter>>,
    pub system_program: Program<System>,
}

impl StarFrameInstruction for CreateCounterIx {
    type ReturnType = ();
    type Accounts<'b, 'c> = CreateCounterAccounts;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        start_at: Self::RunArg<'_>,
        _ctx: &mut impl Context,
    ) -> Result<Self::ReturnType> {
        **account_set.counter.data_mut()? = CounterAccount {
            version: 0,
            signer: *account_set.owner.pubkey(),
            owner: *account_set.owner.pubkey(),
            bump: account_set.counter.access_seeds().bump,
            count: start_at.unwrap_or(0),
            data: Default::default(),
        };

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
#[ix_args(&run)]
pub struct UpdateCounterSignerIx;

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct UpdateCounterSignerAccounts {
    pub signer: Signer<SystemAccount>,
    pub new_signer: SystemAccount,
    pub counter: Mut<Account<CounterAccount>>,
}

impl UpdateCounterSignerAccounts {
    fn validate(&self) -> Result<()> {
        if *self.signer.pubkey() != self.counter.data()?.signer {
            bail!("Incorrect signer");
        }
        Ok(())
    }
}

impl StarFrameInstruction for UpdateCounterSignerIx {
    type ReturnType = ();
    type Accounts<'b, 'c> = UpdateCounterSignerAccounts;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        _run_args: Self::RunArg<'_>,
        _ctx: &mut impl Context,
    ) -> Result<Self::ReturnType> {
        let mut counter = account_set.counter.data_mut()?;
        counter.signer = *account_set.new_signer.pubkey();

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
#[ix_args(run)]
pub struct CountIx {
    pub amount: u64,
    pub subtract: bool,
}

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct CountAccounts {
    pub owner: Signer<SystemAccount>,
    pub counter: Mut<Account<CounterAccount>>,
}

impl CountAccounts {
    fn validate(&self) -> Result<()> {
        if *self.owner.pubkey() != self.counter.data()?.owner {
            bail!("Incorrect owner");
        }
        Ok(())
    }
}

impl StarFrameInstruction for CountIx {
    type ReturnType = ();
    type Accounts<'b, 'c> = CountAccounts;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        CountIx { amount, subtract }: Self::RunArg<'_>,
        _ctx: &mut impl Context,
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
pub struct CloseCounterAccounts {
    #[validate(address = &self.counter.data()?.signer)]
    pub signer: Signer<SystemAccount>,
    #[account_set(recipient)]
    pub funds_to: Mut<SystemAccount>,
    #[cleanup(arg = CloseAccount(()))]
    pub counter: Mut<WrappedCounter>,
}
empty_star_frame_instruction!(CloseCounterIx, CloseCounterAccounts);

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::collections::HashMap;
    use std::env;

    use super::*;
    use mollusk_svm::account_store::AccountStore;
    use mollusk_svm::program::keyed_account_for_system_program;
    use mollusk_svm::result::Check;
    use mollusk_svm::*;
    use solana_account::Account as SolanaAccount;
    use star_frame::client::{DeserializeAccount, SerializeAccount};
    use star_frame::solana_instruction::Instruction;

    #[cfg(feature = "idl")]
    #[test]
    fn generate_idl() -> Result<()> {
        use codama_nodes::{NodeTrait, ProgramNode};
        let idl = StarFrameDeclaredProgram::program_to_idl()?;
        let codama_idl: ProgramNode = idl.try_into()?;
        let idl_json = codama_idl.to_json()?;
        std::fs::write("idl.json", &idl_json)?;
        println!("{idl_json}");
        Ok(())
    }

    #[test]
    fn program_test() -> Result<()> {
        if env::var("SBF_OUT_DIR").is_err() {
            println!("SBF_OUT_DIR is not set, skipping test");
            return Ok(());
        }
        let mollusk = Mollusk::new(&CounterProgram::ID, "counter");

        let owner = Pubkey::new_unique();
        let signer2 = Pubkey::new_unique();
        let funder = Pubkey::new_unique();
        let funds_to = Pubkey::new_unique();

        let start_at = Some(2u64);
        let seeds = CounterAccountSeeds { owner };
        let (counter_account, bump) =
            Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::ID);

        let mollusk = mollusk.with_context(HashMap::from_iter([
            (funder, SolanaAccount::new(1_000_000_000, 0, &System::ID)),
            (owner, SolanaAccount::new(0, 0, &System::ID)),
            (counter_account, SolanaAccount::new(0, 0, &System::ID)),
            (signer2, SolanaAccount::default()),
            (funds_to, SolanaAccount::default()),
            keyed_account_for_system_program(),
        ]));

        let mut expected_counter = CounterAccount {
            version: 0,
            owner,
            signer: owner,
            count: 2,
            bump,
            data: Default::default(),
        };
        // Init a new counter
        mollusk.process_and_validate_instruction(
            &CounterProgram::instruction(
                &CreateCounterIx { start_at },
                CreateCounterClientAccounts {
                    funder,
                    owner,
                    counter: counter_account,
                    system_program: System::ID,
                },
            )?,
            &[
                Check::success(),
                Check::account(&counter_account)
                    .data(&CounterAccount::serialize_account(expected_counter)?)
                    .owner(&CounterProgram::ID)
                    .build(),
            ],
        );

        // Update a counter signer
        expected_counter.signer = signer2;
        mollusk.process_and_validate_instruction(
            &CounterProgram::instruction(
                &UpdateCounterSignerIx,
                UpdateCounterSignerClientAccounts {
                    signer: owner,
                    new_signer: signer2,
                    counter: counter_account,
                },
            )?,
            &[
                Check::success(),
                Check::account(&counter_account)
                    .data(&CounterAccount::serialize_account(expected_counter)?)
                    .build(),
            ],
        );

        const COUNT_ADD: u64 = 7;

        expected_counter.count += COUNT_ADD;
        // Update count
        mollusk.process_and_validate_instruction(
            &CounterProgram::instruction(
                &CountIx {
                    amount: COUNT_ADD,
                    subtract: false,
                },
                CountClientAccounts {
                    owner,
                    counter: counter_account,
                },
            )?,
            &[
                Check::success(),
                Check::account(&counter_account)
                    .data(&CounterAccount::serialize_account(expected_counter)?)
                    .build(),
            ],
        );
        const COUNT_SUB: u64 = 4;
        expected_counter.count -= COUNT_SUB;
        mollusk.process_and_validate_instruction(
            &CounterProgram::instruction(
                &CountIx {
                    amount: COUNT_SUB,
                    subtract: true,
                },
                CountClientAccounts {
                    owner,
                    counter: counter_account,
                },
            )?,
            &[
                Check::success(),
                Check::account(&counter_account)
                    .data(&CounterAccount::serialize_account(expected_counter)?)
                    .build(),
            ],
        );

        let counter_lamports = mollusk
            .account_store
            .borrow()
            .get_account(&counter_account)
            .unwrap()
            .lamports;

        // Close counter
        mollusk.process_and_validate_instruction(
            &CounterProgram::instruction(
                &CloseCounterIx,
                CloseCounterClientAccounts {
                    signer: signer2,
                    funds_to,
                    counter: counter_account,
                },
            )?,
            &[
                Check::success(),
                Check::account(&counter_account)
                    .lamports(0)
                    .data(&[u8::MAX; 8])
                    .build(),
                Check::account(&funds_to).lamports(counter_lamports).build(),
            ],
        );

        Ok(())
    }
}
