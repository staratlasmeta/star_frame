//! This program is used as a testing ground for on chain compute and unsized type behavior

use star_frame::{
    account_set::{modifiers::MaybeMut, CheckKey as _},
    borsh::{BorshDeserialize, BorshSerialize},
    pinocchio::syscalls::sol_remaining_compute_units,
    prelude::*,
};

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
