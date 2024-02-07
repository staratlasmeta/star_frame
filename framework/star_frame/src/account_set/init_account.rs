use crate::account_set::data_account::{DataAccount, ProgramAccount};
use crate::account_set::mutable::Writable;
use crate::account_set::program::Program;
use crate::account_set::seeded_account::{GetSeeds, SeededAccount};
use crate::account_set::{AccountSetValidate, SingleAccountSet};
use crate::prelude::{SeedsWithBump, UnsizedType};
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
use std::ops::{Deref, DerefMut};

#[derive(AccountSet, Debug)]
#[account_set(skip_default_validate)]
#[validate(
    id = "create",
    generics = [<'a, A> where 'info: 'a, A: InitCreateArg<'a, 'info>, T: FrameworkInit<A::FrameworkInitArg>],
    arg = Create<A>,
    extra_validation = init_validate_create(self, arg.0, sys_calls),
)]
#[validate(
    id = "create_if_needed",
    generics = [<'a, A> where 'info: 'a, A: InitCreateArg<'a, 'info>, T: FrameworkInit<A::FrameworkInitArg>],
    arg = CreateIfNeeded<A>,
    extra_validation = init_if_needed(self, arg.0, sys_calls),
)]
pub struct InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    inner: DataAccount<'info, T>,
}
impl<'info, T: ?Sized> SingleAccountSet<'info> for InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}
impl<'info, T> Deref for InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    type Target = DataAccount<'info, T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'info, T> DerefMut for InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub trait InitCreateArg<'a, 'info: 'a>: 'a {
    type FrameworkInitArg;
    type AccountSeeds: GetSeeds;
    type FunderAccount: SingleAccountSet<'info>;
    type FunderSeeds: GetSeeds;

    fn system_program(&self) -> &'a Program<'info, SystemProgram>;

    fn split(
        self,
    ) -> CreateSplit<
        'a,
        'info,
        Self::FrameworkInitArg,
        Self::AccountSeeds,
        Self::FunderAccount,
        Self::FunderSeeds,
    >;
}
#[derive(Derivative)]
#[derivative(
    Debug(
        bound = "IA: Debug, SeedsWithBump<AS>: Debug, Funder<'a, FA, FS>: Debug, Funder<'a, FA, FS>: Debug",
    ),
    Clone(bound = "IA: Clone"),
    Copy(bound = "IA: Copy")
)]
pub struct CreateSplit<'a, 'info: 'a, IA, AS, FA, FS>
where
    AS: GetSeeds,
    FS: GetSeeds,
{
    pub arg: IA,
    pub account_seeds: Option<&'a SeedsWithBump<AS>>,
    pub system_program: &'a Program<'info, SystemProgram>,
    pub funder: Funder<'a, FA, FS>,
}

#[derive(Derivative)]
#[derivative(
    Debug(bound = "Writable<WT>: Debug, SeededAccount<WT, S>: Debug",),
    Clone(bound = ""),
    Copy(bound = "")
)]
pub enum Funder<'a, WT, S = ()>
where
    S: GetSeeds,
{
    Signature(&'a Writable<WT>),
    Seeded(&'a SeededAccount<WT, S>),
}
impl<'a, 'info, WT, S> Funder<'a, WT, S>
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
struct Create<T>(pub T);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
struct CreateIfNeeded<T>(pub T);

#[derive(Derivative)]
#[derivative(
    Debug(bound = "Program<'info, SystemProgram>: Debug, Writable<WT>: Debug"),
    Copy(bound = ""),
    Clone(bound = "")
)]
pub struct CreateAccount<'a, 'info, WT> {
    pub system_program: &'a Program<'info, SystemProgram>,
    pub funder: &'a Writable<WT>,
}
impl<'a, 'info: 'a, WT: SingleAccountSet<'info>> InitCreateArg<'a, 'info>
    for CreateAccount<'a, 'info, WT>
{
    type FrameworkInitArg = ();
    type AccountSeeds = ();
    type FunderAccount = WT;
    type FunderSeeds = ();

    fn system_program(&self) -> &'a Program<'info, SystemProgram> {
        self.system_program
    }

    fn split(
        self,
    ) -> CreateSplit<
        'a,
        'info,
        Self::FrameworkInitArg,
        Self::AccountSeeds,
        Self::FunderAccount,
        Self::FunderSeeds,
    > {
        CreateSplit {
            arg: (),
            account_seeds: None,
            system_program: self.system_program,
            funder: Funder::Signature(self.funder),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = "A: Debug, Program<'info, SystemProgram>: Debug, Writable<WT>: Debug"))]
pub struct CreateAccountWithArg<'a, 'info, A, WT> {
    pub arg: A,
    pub system_program: &'a Program<'info, SystemProgram>,
    pub funder: &'a Writable<WT>,
}
impl<'a, 'info: 'a, A: 'a, WT: SingleAccountSet<'info>> InitCreateArg<'a, 'info>
    for CreateAccountWithArg<'a, 'info, A, WT>
{
    type FrameworkInitArg = A;
    type AccountSeeds = ();
    type FunderAccount = WT;
    type FunderSeeds = ();

    fn system_program(&self) -> &'a Program<'info, SystemProgram> {
        self.system_program
    }

    fn split(
        self,
    ) -> CreateSplit<
        'a,
        'info,
        Self::FrameworkInitArg,
        Self::AccountSeeds,
        Self::FunderAccount,
        Self::FunderSeeds,
    > {
        CreateSplit {
            arg: self.arg,
            account_seeds: None,
            system_program: self.system_program,
            funder: Funder::Signature(self.funder),
        }
    }
}

fn init_if_needed<'a, 'info, A, T>(
    account: &mut InitAccount<'info, T>,
    arg: A,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()>
where
    'info: 'a,
    A: 'a + InitCreateArg<'a, 'info>,
    T: ProgramAccount + FrameworkInit<A::FrameworkInitArg> + ?Sized,
{
    if account.owner() == arg.system_program().key()
        || account.account_info().data.borrow_mut()
            [..size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()]
            .iter()
            .all(|x| *x == 0)
    {
        init_validate_create(account, arg, sys_calls)
    } else {
        account.inner.validate_accounts((), sys_calls)
    }
}

fn init_validate_create<'a, 'info, A, T>(
    account: &mut InitAccount<'info, T>,
    arg: A,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()>
where
    'info: 'a,
    A: 'a + InitCreateArg<'a, 'info>,
    T: ProgramAccount + FrameworkInit<A::FrameworkInitArg> + ?Sized,
{
    let CreateSplit {
        arg,
        account_seeds,
        system_program,
        funder,
    } = arg.split();
    if account.owner() != system_program.key() || funder.owner() != system_program.key() {
        bail!(ProgramError::IllegalOwner);
    }
    let rent = sys_calls.get_rent()?;
    let size =
        T::INIT_LENGTH + size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>();
    let ix = system_instruction::create_account(
        funder.key(),
        account.key(),
        rent.minimum_balance(size),
        size as u64,
        &T::OwnerProgram::program_id(sys_calls)?,
    );
    let accounts: &[AccountInfo<'info>] = &[
        account.account_info_cloned(),
        system_program.account_info_cloned(),
        funder.account_info_cloned(),
    ];
    match (funder, account_seeds) {
        (Funder::Signature(_), None) => {
            sys_calls.invoke(&ix, accounts)?;
        }
        (Funder::Seeded(funder), None) => {
            sys_calls.invoke_signed(&ix, accounts, &[&funder.seeds_with_bump()])?;
        }
        (Funder::Signature(_), Some(account_seeds)) => {
            sys_calls.invoke_signed(&ix, accounts, &[&account_seeds.seeds_with_bump()])?;
        }
        (Funder::Seeded(funder), Some(account_seeds)) => {
            sys_calls.invoke_signed(
                &ix,
                accounts,
                &[&account_seeds.seeds_with_bump(), &funder.seeds_with_bump()],
            )?;
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
        T::init(data_bytes, arg, |_, _| panic!("Cannot resize during init"))?;
    }

    Ok(())
}
