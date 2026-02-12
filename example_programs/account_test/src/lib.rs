//! This program is used as a testing ground for on chain compute and unsized type behavior

use star_frame::{
    account_set::{modifiers::MaybeMut, CheckKey as _, CpiAccountSet, CpiConstWrapper},
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
    WrapperProbe(WrapperProbe),
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

    accounts.borsh_account.vec.push(4);

    list.insert(
        0,
        ListInner {
            id: 1,
            key: arg.key_to_find,
        },
    )?;

    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
#[borsh(crate = "star_frame::borsh")]
pub struct WrapperProbe;

#[derive(AccountSet)]
pub struct WrapperProbeAccounts {
    pub system_program: Program<System>,
}

#[star_frame_instruction]
fn WrapperProbe(accounts: &mut WrapperProbeAccounts) -> Result<()> {
    let wrapper_accounts = None::<()>;
    let inner_accounts = None::<()>;
    ensure_eq!(
        wrapper_accounts,
        inner_accounts,
        ProgramError::InvalidInstructionData
    );

    let mut infos_with_program = [std::mem::MaybeUninit::uninit(); 2];
    let mut with_program_index = 1;
    CpiConstWrapper::<Option<()>, 0>::write_account_infos(
        Some(accounts.system_program.account_info()),
        &wrapper_accounts,
        &mut with_program_index,
        &mut infos_with_program,
    )?;
    ensure_eq!(with_program_index, 2, ProgramError::InvalidInstructionData);

    let mut wrapper_infos_without_program = [std::mem::MaybeUninit::uninit(); 2];
    let mut inner_infos_without_program = [std::mem::MaybeUninit::uninit(); 2];
    let mut wrapper_without_program_index = 1;
    let mut inner_without_program_index = 1;
    let wrapper_without_program = CpiConstWrapper::<Option<()>, 0>::write_account_infos(
        None,
        &wrapper_accounts,
        &mut wrapper_without_program_index,
        &mut wrapper_infos_without_program,
    );
    let inner_without_program = <Option<()> as CpiAccountSet>::write_account_infos(
        None,
        &inner_accounts,
        &mut inner_without_program_index,
        &mut inner_infos_without_program,
    );
    ensure!(
        wrapper_without_program.is_err(),
        ProgramError::InvalidInstructionData
    );
    ensure!(
        inner_without_program.is_err(),
        ProgramError::InvalidInstructionData
    );

    let wrapper_without_program_error = ProgramError::from(wrapper_without_program.unwrap_err());
    let inner_without_program_error = ProgramError::from(inner_without_program.unwrap_err());
    ensure_eq!(
        wrapper_without_program_error,
        inner_without_program_error,
        ProgramError::InvalidInstructionData
    );
    ensure_eq!(
        wrapper_without_program_index,
        inner_without_program_index,
        ProgramError::InvalidInstructionData
    );
    ensure_eq!(
        wrapper_without_program_index,
        1,
        ProgramError::InvalidInstructionData
    );

    let program_id = accounts.system_program.pubkey();
    let mut wrapper_meta_index = 3;
    let mut inner_meta_index = 3;
    let mut wrapper_metas: [std::mem::MaybeUninit<PinocchioAccountMeta<'_>>; 4] =
        std::array::from_fn(|_| std::mem::MaybeUninit::uninit());
    let mut inner_metas: [std::mem::MaybeUninit<PinocchioAccountMeta<'_>>; 4] =
        std::array::from_fn(|_| std::mem::MaybeUninit::uninit());
    CpiConstWrapper::<Option<()>, 0>::write_account_metas(
        program_id,
        &wrapper_accounts,
        &mut wrapper_meta_index,
        &mut wrapper_metas,
    );
    <Option<()> as CpiAccountSet>::write_account_metas(
        program_id,
        &inner_accounts,
        &mut inner_meta_index,
        &mut inner_metas,
    );
    ensure_eq!(
        wrapper_meta_index,
        inner_meta_index,
        ProgramError::InvalidInstructionData
    );
    ensure_eq!(wrapper_meta_index, 4, ProgramError::InvalidInstructionData);

    Ok(())
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

    enum SbfCiPolicy {
        OptionalInCi,
        RequiredInCi,
    }

    fn env_is_truthy(name: &str) -> bool {
        env::var(name)
            .map(|value| {
                let normalized = value.to_ascii_lowercase();
                normalized == "1" || normalized == "true" || normalized == "yes"
            })
            .unwrap_or(false)
    }

    fn should_run_sbf_test(test_name: &str, ci_policy: SbfCiPolicy) -> bool {
        if env::var_os("SBF_OUT_DIR").is_some() {
            return true;
        }

        let required_in_ci = env_is_truthy("CI")
            && env_is_truthy("SF_REQUIRE_SBF_TESTS")
            && matches!(ci_policy, SbfCiPolicy::RequiredInCi);

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
        if !should_run_sbf_test("test_ix", SbfCiPolicy::OptionalInCi) {
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

    #[test]
    fn wrapper_probe_ix() -> Result<()> {
        if !should_run_sbf_test("wrapper_probe_ix", SbfCiPolicy::RequiredInCi) {
            return Ok(());
        }

        let mollusk = Mollusk::new(&AccountTest::ID, "account_test");
        let account_store: HashMap<Pubkey, SolanaAccount> =
            HashMap::from_iter([keyed_account_for_system_program()]);
        let mollusk = mollusk.with_context(account_store);

        let _res = mollusk.process_and_validate_instruction(
            &AccountTest::instruction(
                &WrapperProbe,
                WrapperProbeClientAccounts {
                    system_program: None,
                },
            )?,
            &[Check::success()],
        );

        Ok(())
    }
}
