use crate::prelude::*;
use advance::Advance;
use anyhow::{bail, Context};
use bytemuck::bytes_of;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use solana_program::program_memory::sol_memset;
use solana_program::system_instruction;
use star_frame_proc::AccountSet;
use std::fmt::Debug;
use std::mem::size_of;

#[derive(AccountSet, Debug, Deref, DerefMut)]
#[account_set(skip_default_validate)]
#[validate(
    id = "create",
    generics = [<A> where A: InitCreateArg<'info>, T: UnsizedInit<A::StarFrameInitArg>],
    arg = Create<A>,
    extra_validation = self.init_validate_create(arg.0, sys_calls),
)]
#[validate(
    id = "create_if_needed",
    generics = [<A> where A: InitCreateArg<'info>, T: UnsizedInit<A::StarFrameInitArg>],
    arg = CreateIfNeeded<A>,
    extra_validation = self.init_if_needed(arg.0, sys_calls),
)]
#[cleanup(
    generics = [<A> where DataAccount<'info, T>: AccountSetCleanup<'info, A>],
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

impl<'info, T> HasProgramAccount<'info> for InitAccount<'info, T>
where
    T: ProgramAccount + UnsizedType + ?Sized,
{
    type ProgramAccount = T;
}

pub trait InitCreateArg<'info> {
    type StarFrameInitArg;
    type AccountSeeds: GetSeeds;
    type FunderAccount: SignedAccount<'info> + WritableAccount<'info>;

    fn system_program(&self) -> &Program<'info, SystemProgram>;

    fn split<'a>(
        &'a mut self,
    ) -> CreateSplit<'a, 'info, Self::StarFrameInitArg, Self::AccountSeeds, Self::FunderAccount>;
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
#[derivative(Debug(bound = "A: Debug, Program<'info, SystemProgram>: Debug, WT: Debug"))]
pub struct CreateAccount<'a, 'info, A, WT> {
    arg: Option<A>,
    system_program: &'a Program<'info, SystemProgram>,
    funder: &'a WT,
}

impl<'a, 'info, WT> CreateAccount<'a, 'info, Zeroed, WT> {
    pub fn new(system_program: &'a Program<'info, SystemProgram>, funder: &'a WT) -> Self {
        Self::new_with_arg(Zeroed, system_program, funder)
    }
}

impl<'a, 'info, A, WT> CreateAccount<'a, 'info, A, WT> {
    pub fn new_with_arg(
        arg: A,
        system_program: &'a Program<'info, SystemProgram>,
        funder: &'a WT,
    ) -> Self {
        Self {
            arg: Some(arg),
            system_program,
            funder,
        }
    }
}
impl<'a, 'info, A, WT: SignedAccount<'info> + WritableAccount<'info>> InitCreateArg<'info>
    for CreateAccount<'a, 'info, A, WT>
{
    type StarFrameInitArg = A;
    type AccountSeeds = ();
    type FunderAccount = WT;

    fn system_program(&self) -> &'a Program<'info, SystemProgram> {
        self.system_program
    }

    fn split<'b>(
        &'b mut self,
    ) -> CreateSplit<'b, 'info, Self::StarFrameInitArg, Self::AccountSeeds, Self::FunderAccount>
    {
        CreateSplit {
            arg: self.arg.take().unwrap(),
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
    fn init_if_needed<A>(&mut self, arg: A, sys_calls: &mut impl SysCallInvoke) -> Result<()>
    where
        A: InitCreateArg<'info>,
        T: UnsizedInit<A::StarFrameInitArg>,
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

    fn init_validate_create<A>(
        &mut self,
        mut arg: A,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<()>
    where
        A: InitCreateArg<'info>,
        T: UnsizedInit<A::StarFrameInitArg>,
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
        let size =
            T::INIT_BYTES + size_of::<<T::OwnerProgram as StarFrameProgram>::AccountDiscriminant>();
        let ix = system_instruction::create_account(
            funder.key(),
            self.key(),
            rent.minimum_balance(size),
            size as u64,
            &T::OwnerProgram::PROGRAM_ID,
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
        let mut data_bytes = &mut **data_bytes;

        data_bytes
            .try_advance(size_of::<
                <T::OwnerProgram as StarFrameProgram>::AccountDiscriminant,
            >())?
            .copy_from_slice(bytes_of(&T::DISCRIMINANT));
        let data_bytes = data_bytes.try_advance(T::INIT_BYTES)?;
        sol_memset(data_bytes, 0, data_bytes.len());
        unsafe {
            T::init(data_bytes, arg)?;
        }

        Ok(())
    }
}
