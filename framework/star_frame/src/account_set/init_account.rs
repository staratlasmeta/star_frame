use crate::account_set::data_account::{DataAccount, ProgramAccount};
use crate::account_set::mutable::Writable;
use crate::account_set::program::Program;
use crate::account_set::seeded_account::GetSeeds;
use crate::account_set::{AccountSetValidate, SignedAccount, SingleAccountSet};
use crate::prelude::*;
use crate::program::system_program::SystemProgram;
use crate::program::StarFrameProgram;
use crate::serialize::FrameworkInit;
use crate::sys_calls::SysCallInvoke;
use crate::Result;
use advance::Advance;
use anyhow::{bail, Context};
use bytemuck::bytes_of;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memset;
use solana_program::system_instruction;
use star_frame::account_set::WritableAccount;
use star_frame_proc::AccountSet;
use std::fmt::Debug;
use std::mem::size_of;

#[derive(AccountSet, Debug, Deref, DerefMut)]
#[account_set(skip_default_validate)]
#[validate(
    id = "create",
    generics = [<A> where 'info: 'validate, A: InitCreateArg<'validate, 'info>, T: FrameworkInit<A::FrameworkInitArg>],
    arg = Create<A>,
    extra_validation = self.init_validate_create(arg.0, sys_calls),
)]
#[validate(
    id = "create_if_needed",
    generics = [<A> where 'info: 'validate, A: InitCreateArg<'validate, 'info>, T: FrameworkInit<A::FrameworkInitArg>],
    arg = CreateIfNeeded<A>,
    extra_validation = self.init_if_needed(arg.0, sys_calls),
)]
#[cleanup(
    generics = [<A> where DataAccount<'info, T>: AccountSetCleanup<'cleanup, 'info, A>],
    arg = A,
)]
pub struct InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    #[validate(id = "create", skip)]
    #[validate(id = "create_if_needed", skip)]
    #[cleanup(arg = arg)]
    inner: Writable<DataAccount<'info, T>>,
}
impl<'info, T: ?Sized> SingleAccountSet<'info> for InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType,
{
    fn account_info(&self) -> &AccountInfo<'info> {
        self.inner.account_info()
    }
}
impl<'info, T> WritableAccount<'info> for InitAccount<'info, T> where
    T: ProgramAccount + UnsizedType + ?Sized
{
}

pub trait InitCreateArg<'a, 'info: 'a>: 'a {
    type FrameworkInitArg;
    type AccountSeeds: GetSeeds;
    type FunderAccount: SignedAccount<'info> + WritableAccount<'info>;

    fn system_program(&self) -> &Program<'info, SystemProgram>;

    fn split(
        self,
    ) -> CreateSplit<'a, 'info, Self::FrameworkInitArg, Self::AccountSeeds, Self::FunderAccount>;
}
#[derive(Derivative)]
#[derivative(
    Debug(bound = "IA: Debug, SeedsWithBump<AS>: Debug, FA: Debug",),
    Clone(bound = "IA: Clone"),
    Copy(bound = "IA: Copy")
)]
pub struct CreateSplit<'a, 'info: 'a, IA, AS, FA>
where
    AS: GetSeeds,
{
    pub arg: IA,
    pub account_seeds: Option<&'a SeedsWithBump<AS>>,
    pub system_program: &'a Program<'info, SystemProgram>,
    pub funder: &'a FA,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Create<T>(pub T);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct CreateIfNeeded<T>(pub T);

#[derive(Derivative)]
#[derivative(
    Debug(bound = "Program<'info, SystemProgram>: Debug, WT: Debug"),
    Copy(bound = ""),
    Clone(bound = "")
)]
pub struct CreateAccount<'a, 'info, WT> {
    pub system_program: &'a Program<'info, SystemProgram>,
    pub funder: &'a WT,
}
impl<'a, 'info, WT: SignedAccount<'info> + WritableAccount<'info>> InitCreateArg<'a, 'info>
    for CreateAccount<'a, 'info, WT>
{
    type FrameworkInitArg = ();
    type AccountSeeds = ();
    type FunderAccount = WT;

    fn system_program(&self) -> &Program<'info, SystemProgram> {
        self.system_program
    }

    fn split(
        self,
    ) -> CreateSplit<'a, 'info, Self::FrameworkInitArg, Self::AccountSeeds, Self::FunderAccount>
    {
        CreateSplit {
            arg: (),
            account_seeds: None,
            system_program: self.system_program,
            funder: self.funder,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = "A: Debug, Program<'info, SystemProgram>: Debug, WT: Debug"))]
pub struct CreateAccountWithArg<'a, 'info, A, WT> {
    arg: A,
    system_program: &'a Program<'info, SystemProgram>,
    funder: &'a WT,
}
impl<'a, 'info, A, WT> CreateAccountWithArg<'a, 'info, A, WT> {
    pub fn new(
        arg: A,
        system_program: &'a Program<'info, SystemProgram>,
        funder: &'a Writable<WT>,
    ) -> Self {
        Self {
            arg,
            system_program,
            funder,
        }
    }
}
impl<'a, 'info, A: 'a, WT: SignedAccount<'info> + WritableAccount<'info>> InitCreateArg<'a, 'info>
    for CreateAccountWithArg<'a, 'info, A, WT>
{
    type FrameworkInitArg = A;
    type AccountSeeds = ();
    type FunderAccount = WT;

    fn system_program(&self) -> &'a Program<'info, SystemProgram> {
        self.system_program
    }

    fn split(
        self,
    ) -> CreateSplit<'a, 'info, Self::FrameworkInitArg, Self::AccountSeeds, Self::FunderAccount>
    {
        CreateSplit {
            arg: self.arg,
            account_seeds: None,
            system_program: self.system_program,
            funder: self.funder,
        }
    }
}

impl<'info, T> InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    fn init_if_needed<'a, A>(&mut self, arg: A, sys_calls: &mut impl SysCallInvoke) -> Result<()>
    where
        'info: 'a,
        A: InitCreateArg<'a, 'info>,
        T: FrameworkInit<A::FrameworkInitArg>,
    {
        if self.owner() == arg.system_program().key()
            || self.account_info().data.borrow_mut()
                [..size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>()]
                .iter()
                .all(|x| *x == 0)
        {
            self.init_validate_create(arg, sys_calls)
        } else {
            // skip writable check on init_if_needed when not needed
            self.inner.0.validate_accounts((), sys_calls)
        }
    }

    fn init_validate_create<'a, A>(
        &mut self,
        arg: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()>
    where
        'info: 'a,
        A: InitCreateArg<'a, 'info>,
        T: FrameworkInit<A::FrameworkInitArg>,
    {
        self.inner
            .check_writable()
            .context("InitAccount must be writable")?;
        let CreateSplit {
            arg,
            account_seeds,
            system_program,
            funder,
        } = arg.split();
        if self.owner() != system_program.key() || funder.owner() != system_program.key() {
            bail!(ProgramError::IllegalOwner);
        }
        let rent = sys_calls.get_rent()?;
        let size = T::INIT_LENGTH
            + size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>();
        let ix = system_instruction::create_account(
            funder.key(),
            self.key(),
            rent.minimum_balance(size),
            size as u64,
            &T::OwnerProgram::program_id(sys_calls)?,
        );
        let accounts: &[AccountInfo<'info>] = &[
            self.account_info_cloned(),
            system_program.account_info_cloned(),
            funder.account_info_cloned(),
        ];
        match (funder.signer_seeds(), account_seeds) {
            (None, None) => {
                sys_calls.invoke(&ix, accounts)?;
            }
            (Some(funder), None) => {
                sys_calls.invoke_signed(&ix, accounts, &[&funder])?;
            }
            (None, Some(account_seeds)) => {
                sys_calls.invoke_signed(&ix, accounts, &[&account_seeds.seeds_with_bump()])?;
            }
            (Some(funder), Some(account_seeds)) => {
                sys_calls.invoke_signed(
                    &ix,
                    accounts,
                    &[&account_seeds.seeds_with_bump(), &funder],
                )?;
            }
        }

        let mut data_bytes = self.info_data_bytes_mut()?;
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
}
