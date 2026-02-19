//! This program is used as a testing ground for on chain compute and unsized type behavior

use star_frame::{
    account_set::{modifiers::MaybeMut, CheckKey as _},
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
)]
pub struct AccountTest;

#[derive(InstructionSet)]
pub enum AccountTestInstructionSet {
    Run(Run),
    #[cfg(feature = "probe_ix")]
    BorshProbeInner(BorshProbeInner),
    #[cfg(feature = "probe_ix")]
    BorshProbeInnerMut(BorshProbeInnerMut),
    #[cfg(feature = "probe_ix")]
    BorshProbeNonWritableMut(BorshProbeNonWritableMut),
}

#[derive(BorshSerialize, BorshDeserialize, InstructionArgs, Copy, Clone)]
#[ix_args(run)]
#[borsh(crate = "star_frame::borsh")]
pub struct Run {
    key_to_find: Pubkey,
    id_to_find: u64,
}

#[derive(AccountSet)]
pub struct RunAccounts<const MUT: bool> {
    #[validate(funder)]
    pub funder: Mut<Signer>,
    #[cleanup(arg = NormalizeRent(()))]
    pub account: MaybeMut<MUT, Account<AccountData>>,
    #[validate(arg = Create((|| MyBorshAccount::default(), &self.funder,)))]
    #[cleanup(arg = NormalizeRent(()))]
    pub borsh_account: Init<Signer<BorshAccount<MyBorshAccount>>>,
    pub system_program: Program<System>,
    pub inner: RunAccountsInner,
}

#[derive(AccountSet)]
pub struct RunAccountsInner {
    inner2: RunAccountsInnerInner,
}

#[derive(AccountSet, Debug)]
#[validate(extra_validation = self.validate())]
pub struct RunAccountsInnerInner(#[single_account_set] AccountInfo);

impl RunAccountsInnerInner {
    fn validate(&self) -> Result<()> {
        self.0
            .check_key(&System::ID)
            .with_ctx(|| format!("Key isnt system id!! {:?}", self))?;
        Ok(())
    }
}

#[unsized_type(program_account)]
pub struct AccountData {
    #[unsized_start]
    list: List<ListInner>,
}

#[derive(ProgramAccount, BorshSerialize, BorshDeserialize, Debug, Default)]
#[borsh(crate = "star_frame::borsh")]
pub struct MyBorshAccount {
    vec: Vec<u8>,
}

#[zero_copy(pod)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, TypeToIdl)]
struct ListInner {
    id: u64,
    key: Pubkey,
}

#[star_frame_instruction]
fn Run(accounts: &mut RunAccounts<true>, arg: Run) -> Result<()> {
    let mut data = accounts.account.data_mut()?;
    let before = remaining_compute();
    let mut list = data.list();
    let after = remaining_compute();
    msg!("compute units: {}", before - after - 100);

    accounts
        .borsh_account
        .set_inner(MyBorshAccount { vec: vec![1, 2, 3] })?;

    accounts.borsh_account.inner_mut()?.vec.push(4);

    list.insert(
        0,
        ListInner {
            id: 1,
            key: arg.key_to_find,
        },
    )?;

    Ok(())
}
#[cfg(feature = "probe_ix")]
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
#[borsh(crate = "star_frame::borsh")]
pub struct BorshProbeInner;

#[cfg(feature = "probe_ix")]
#[derive(AccountSet)]
pub struct BorshProbeInnerAccounts {
    pub borsh_account: Signer<BorshAccount<MyBorshAccount>>,
}

#[cfg(feature = "probe_ix")]
#[star_frame_instruction]
fn BorshProbeInner(accounts: &mut BorshProbeInnerAccounts) -> Result<()> {
    accounts.borsh_account.inner().map(|_| ())
}

#[cfg(feature = "probe_ix")]
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
#[borsh(crate = "star_frame::borsh")]
pub struct BorshProbeInnerMut;

#[cfg(feature = "probe_ix")]
#[derive(AccountSet)]
pub struct BorshProbeInnerMutAccounts {
    pub borsh_account: Mut<Signer<BorshAccount<MyBorshAccount>>>,
}

#[cfg(feature = "probe_ix")]
#[star_frame_instruction]
fn BorshProbeInnerMut(accounts: &mut BorshProbeInnerMutAccounts) -> Result<()> {
    accounts.borsh_account.inner_mut().map(|_| ())
}

#[cfg(feature = "probe_ix")]
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
#[borsh(crate = "star_frame::borsh")]
pub struct BorshProbeNonWritableMut;

#[cfg(feature = "probe_ix")]
#[derive(AccountSet)]
pub struct BorshProbeNonWritableMutAccounts {
    pub borsh_account: MaybeMut<false, Signer<BorshAccount<MyBorshAccount>>>,
}

#[cfg(feature = "probe_ix")]
#[star_frame_instruction]
fn BorshProbeNonWritableMut(accounts: &mut BorshProbeNonWritableMutAccounts) -> Result<()> {
    accounts.borsh_account.inner_mut().map(|_| ())
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use mollusk_svm::{result::Check, Mollusk};
    use pretty_assertions::assert_eq;
    use solana_account::Account as SolanaAccount;
    use solana_instruction::error::InstructionError;
    use star_frame::client::{DeserializeAccount, SerializeAccount};
    use star_frame::errors::{ErrorCode, StarFrameError as _};
    use std::{collections::HashMap, env};

    fn env_is_truthy(name: &str) -> bool {
        env::var(name)
            .map(|value| {
                let normalized = value.to_ascii_lowercase();
                normalized == "1" || normalized == "true" || normalized == "yes"
            })
            .unwrap_or(false)
    }

    fn should_run_sbf_test(test_name: &str, required_in_ci: bool) -> bool {
        if env::var_os("SBF_OUT_DIR").is_some() {
            return true;
        }

        let required_in_ci =
            required_in_ci && env_is_truthy("CI") && env_is_truthy("SF_REQUIRE_SBF_TESTS");

        assert!(
            !required_in_ci,
            "SBF_OUT_DIR must be set in CI for `{}`. Build the SBF artifact and export SBF_OUT_DIR.",
            test_name
        );

        println!("SBF_OUT_DIR is not set, skipping test `{}`", test_name);
        false
    }

    #[test]
    fn test_ix() -> Result<()> {
        if !should_run_sbf_test("test_ix", false) {
            return Ok(());
        }
        let mut mollusk = Mollusk::new(&AccountTest::ID, "account_test");
        mollusk_svm_programs_token::token::add_program(&mut mollusk);
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

        const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

        let account = Pubkey::new_unique();
        let borsh_account = Pubkey::new_unique();

        let list = std::iter::repeat_with(|| ListInner {
            id: 2,
            key: Pubkey::new_unique(),
        })
        .take(10000)
        .collect::<Vec<_>>();

        let account_data = AccountData::serialize_account(AccountDataOwned { list })?;

        let funder = Pubkey::new_unique();

        let mut account_store: HashMap<Pubkey, SolanaAccount> = HashMap::from_iter([
            (
                account,
                SolanaAccount {
                    lamports: 0,
                    data: account_data,
                    owner: AccountTest::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (borsh_account, SolanaAccount::default()),
            (
                funder,
                SolanaAccount {
                    lamports: LAMPORTS_PER_SOL * 10,
                    data: vec![],
                    owner: System::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ]);
        let mollusk = mollusk.with_context(account_store);

        let res = mollusk.process_and_validate_instruction(
            &AccountTest::instruction(
                &Run {
                    key_to_find: Pubkey::new_unique(),
                    id_to_find: 1,
                },
                RunClientAccounts {
                    account,
                    borsh_account,
                    funder,
                    system_program: None,
                    inner: RunAccountsInnerClientAccounts {
                        inner2: Pubkey::new_unique(),
                    },
                },
            )?,
            &[Check::success()],
        );

        let borsh_account_data = MyBorshAccount::deserialize_account(
            &mollusk
                .account_store
                .borrow()
                .get(&borsh_account)
                .unwrap()
                .data,
        )?;
        assert_eq!(borsh_account_data.vec, vec![1, 2, 3, 4]);

        Ok(())
    }
    #[cfg(feature = "probe_ix")]
    #[test]
    fn borsh_probe_inner_uninitialized_ix() -> Result<()> {
        if !should_run_sbf_test("borsh_probe_inner_uninitialized_ix", true) {
            return Ok(());
        }

        let mollusk = Mollusk::new(&AccountTest::ID, "account_test");
        let borsh_account = Pubkey::new_unique();
        let borsh_account_data = MyBorshAccount::discriminant_bytes();
        let account_store: HashMap<Pubkey, SolanaAccount> = HashMap::from_iter([(
            borsh_account,
            SolanaAccount {
                lamports: 1_000_000_000,
                data: borsh_account_data,
                owner: AccountTest::ID,
                executable: false,
                rent_epoch: 0,
            },
        )]);
        let mollusk = mollusk.with_context(account_store);

        let _res = mollusk.process_and_validate_instruction(
            &AccountTest::instruction(
                &BorshProbeInner,
                BorshProbeInnerClientAccounts { borsh_account },
            )?,
            &[Check::instruction_err(InstructionError::InvalidAccountData)],
        );

        Ok(())
    }

    #[cfg(feature = "probe_ix")]
    #[test]
    fn borsh_probe_inner_mut_uninitialized_ix() -> Result<()> {
        if !should_run_sbf_test("borsh_probe_inner_mut_uninitialized_ix", true) {
            return Ok(());
        }

        let mollusk = Mollusk::new(&AccountTest::ID, "account_test");
        let borsh_account = Pubkey::new_unique();
        let borsh_account_data = MyBorshAccount::discriminant_bytes();
        let account_store: HashMap<Pubkey, SolanaAccount> = HashMap::from_iter([(
            borsh_account,
            SolanaAccount {
                lamports: 1_000_000_000,
                data: borsh_account_data,
                owner: AccountTest::ID,
                executable: false,
                rent_epoch: 0,
            },
        )]);
        let mollusk = mollusk.with_context(account_store);

        let _res = mollusk.process_and_validate_instruction(
            &AccountTest::instruction(
                &BorshProbeInnerMut,
                BorshProbeInnerMutClientAccounts { borsh_account },
            )?,
            &[Check::instruction_err(InstructionError::InvalidAccountData)],
        );

        Ok(())
    }

    #[cfg(feature = "probe_ix")]
    #[test]
    fn borsh_probe_inner_mut_non_writable_ix() -> Result<()> {
        if !should_run_sbf_test("borsh_probe_inner_mut_non_writable_ix", true) {
            return Ok(());
        }

        let mollusk = Mollusk::new(&AccountTest::ID, "account_test");
        let borsh_account = Pubkey::new_unique();
        let borsh_account_data = MyBorshAccount::serialize_account(&MyBorshAccount::default())?;
        let account_store: HashMap<Pubkey, SolanaAccount> = HashMap::from_iter([(
            borsh_account,
            SolanaAccount {
                lamports: 1_000_000_000,
                data: borsh_account_data,
                owner: AccountTest::ID,
                executable: false,
                rent_epoch: 0,
            },
        )]);
        let mollusk = mollusk.with_context(account_store);

        let _res = mollusk.process_and_validate_instruction(
            &AccountTest::instruction(
                &BorshProbeNonWritableMut,
                BorshProbeNonWritableMutClientAccounts { borsh_account },
            )?,
            &[Check::instruction_err(InstructionError::Custom(
                ErrorCode::ExpectedWritable.code(),
            ))],
        );

        Ok(())
    }
}
