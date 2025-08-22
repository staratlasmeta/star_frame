//! This program is used as a testing ground for on chain compute and unsized type behavior
use star_frame::{
    account_set::Account,
    borsh::{BorshDeserialize, BorshSerialize},
    pinocchio::syscalls::sol_remaining_compute_units,
    prelude::*,
};

#[allow(unused)]
fn remaining_compute() -> u64 {
    unsafe { sol_remaining_compute_units() }
}

const TEST_ID: Pubkey = Pubkey::new_from_array([1; 32]);
#[derive(StarFrameProgram)]
#[program(
    instruction_set = AccountTestInstructionSet,
    id = TEST_ID,
    skip_idl
)]
pub struct AccountTest;

#[derive(InstructionSet)]
#[ix_set(skip_idl)]
pub enum AccountTestInstructionSet {
    Run(RunIx),
}

#[derive(BorshSerialize, BorshDeserialize, InstructionArgs, Copy, Clone)]
#[instruction_args(skip_idl)]
#[ix_args(run)]
#[borsh(crate = "star_frame::borsh")]
pub struct RunIx {
    key_to_find: Pubkey,
    id_to_find: u64,
}

#[derive(AccountSet)]
#[account_set(skip_default_idl)]
pub struct RunAccounts {
    pub account: Account<AccountData>,
}

#[unsized_type(program_account, skip_idl)]
pub struct AccountData {
    #[unsized_start]
    list: List<ListInner>,
}

#[derive(Pod, Zeroable, Debug, PartialEq, Eq, PartialOrd, Ord, Align1, Copy, Clone)]
#[repr(C, packed)]
struct ListInner {
    id: u64,
    key: Pubkey,
}

impl StarFrameInstruction for RunIx {
    type ReturnType = ();

    type Accounts<'b, 'c> = RunAccounts;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        arg: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut data = account_set.account.data_mut()?;
        let before = remaining_compute();
        let mut list = data.list();
        let after = remaining_compute();
        msg!("compute units: {}", before - after - 100);

        list.insert(
            0,
            ListInner {
                id: 1,
                key: arg.key_to_find,
            },
        )?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
    use pretty_assertions::assert_eq;
    use solana_account::Account as SolanaAccount;
    use star_frame::client::{DeserializeAccount, SerializeAccount};
    use std::{collections::HashMap, env};

    #[test]
    fn test_ix() -> Result<()> {
        if env::var("SBF_OUT_DIR").is_err() {
            println!("SBF_OUT_DIR is not set, skipping test");
            return Ok(());
        }
        let mut mollusk = Mollusk::new(&AccountTest::ID, "account_test");
        mollusk_svm_programs_token::token::add_program(&mut mollusk);
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

        const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

        let account = Pubkey::new_unique();

        let list = std::iter::repeat_with(|| ListInner {
            id: 2,
            key: Pubkey::new_unique(),
            // key2: Pubkey::new_unique(),
        })
        .take(10000)
        .collect::<Vec<_>>();

        let account_data = AccountData::serialize_account(AccountDataOwned { list })?;

        let mut account_store: HashMap<Pubkey, SolanaAccount> = HashMap::from_iter([(
            account,
            SolanaAccount {
                lamports: LAMPORTS_PER_SOL * 10,
                data: account_data,
                owner: AccountTest::ID,
                executable: false,
                rent_epoch: 0,
            },
        )]);
        let mollusk = mollusk.with_context(account_store);

        let res = mollusk.process_and_validate_instruction(
            &AccountTest::instruction(
                &RunIx {
                    key_to_find: Pubkey::new_unique(),
                    id_to_find: 1,
                },
                RunClientAccounts { account },
            )?,
            &[Check::success()],
        );

        Ok(())
    }
}
