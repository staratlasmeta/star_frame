use crate::account_set::data_account::{AccountData, DataAccount};
use crate::account_set::mutable::Writable;
use crate::account_set::program::Program;
use crate::account_set::seeded_account::{GetSeeds, SeededAccount};
use crate::account_set::SingleAccountSet;
use crate::program::system_program::SystemProgram;
use crate::program::StarFrameProgram;
use crate::serialize::FrameworkInit;
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use advance::Advance;
use anyhow::bail;
use bytemuck::bytes_of;
use derivative::Derivative;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memset;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use star_frame_proc::AccountSet;
use std::fmt::Debug;
use std::mem::size_of;

#[derive(AccountSet, Debug)]
#[validate(
    generics = [<A, WT, S, const CHECK: bool> where T: FrameworkInit<A>, WT: SingleAccountSet<'info>, S: GetSeeds],
    arg = CreateAccountWithArg<'_, 'info, A, WT, S, CHECK>,
    extra_validation = init_validate_create(self, arg, sys_calls),
)]
pub struct InitAccount<'info, T>
where
    T: AccountData + ?Sized,
{
    inner: DataAccount<'info, T>,
}
impl<'info, T: ?Sized> SingleAccountSet<'info> for InitAccount<'info, T>
where
    T: AccountData,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}

#[derive(Derivative)]
#[derivative(Debug(
    bound = "A: Debug, Program<'info, SystemProgram>: Debug, Funder<'a, WT, S, CHECK>: Debug"
))]
pub struct CreateAccountWithArg<
    'a,
    'info,
    A,
    WT = AccountInfo<'info>,
    S = (),
    const CHECK: bool = false,
> where
    S: GetSeeds,
{
    arg: A,
    system_program: &'a Program<'info, SystemProgram>,
    funder: Funder<'a, WT, S, CHECK>,
}

#[derive(Derivative)]
#[derivative(Debug(bound = "Writable<WT, CHECK>: Debug, SeededAccount<WT, S>: Debug"))]
pub enum Funder<'a, WT, S = (), const CHECK: bool = false>
where
    S: GetSeeds,
{
    Signature(&'a Writable<WT, CHECK>),
    Seeded(&'a SeededAccount<WT, S>),
}
impl<'a, 'info, WT, S, const CHECK: bool> Funder<'a, WT, S, CHECK>
where
    WT: SingleAccountSet<'info>,
    S: GetSeeds,
{
    fn owner(&self) -> &'info Pubkey {
        match self {
            Funder::Signature(inner) => inner.owner(),
            Funder::Seeded(inner) => inner.owner(),
        }
    }

    fn key(&self) -> &'info Pubkey {
        match self {
            Funder::Signature(inner) => inner.key(),
            Funder::Seeded(inner) => inner.key(),
        }
    }

    fn account_info_cloned(&self) -> AccountInfo<'info> {
        match self {
            Funder::Signature(inner) => inner.account_info_cloned(),
            Funder::Seeded(inner) => inner.account_info_cloned(),
        }
    }
}

fn init_validate_create<'info, A, WT, T, S, const CHECK: bool>(
    account: &mut InitAccount<'info, T>,
    arg: CreateAccountWithArg<'_, 'info, A, WT, S, CHECK>,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()>
where
    T: AccountData + FrameworkInit<A> + ?Sized,
    WT: SingleAccountSet<'info>,
    S: GetSeeds,
{
    if account.owner() != arg.system_program.key() || arg.funder.owner() != arg.system_program.key()
    {
        bail!(ProgramError::IllegalOwner);
    }
    let rent = sys_calls.get_rent()?;
    let size =
        T::INIT_LENGTH + size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>();
    let ix = system_instruction::create_account(
        arg.funder.key(),
        account.key(),
        rent.minimum_balance(size),
        size as u64,
        &T::program_id(),
    );
    let accounts: &[AccountInfo<'info>] = &[
        account.account_info_cloned(),
        arg.system_program.account_info_cloned(),
        arg.funder.account_info_cloned(),
    ];
    match arg.funder {
        Funder::Signature(_) => {
            sys_calls.invoke(&ix, accounts)?;
        }
        Funder::Seeded(funder) => {
            sys_calls.invoke_signed(&ix, accounts, &[&funder.seeds_with_bump()])?;
        }
    }

    let mut data_bytes = account.info_data_bytes_mut()?;
    let mut data_bytes = &mut *data_bytes;

    data_bytes
        .try_advance(size_of::<
            <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
        >())?
        .copy_from_slice(bytes_of(&T::DISCRIMINANT));
    let data_bytes = data_bytes.try_advance(T::INIT_LENGTH)?;
    sol_memset(data_bytes, 0, data_bytes.len());
    unsafe {
        T::init(data_bytes, arg.arg, |_, _| {
            panic!("Cannot resize during init")
        })?;
    }

    Ok(())
}
