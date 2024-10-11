use crate::prelude::*;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use star_frame_proc::AccountSet;
use std::fmt::Debug;

#[derive(AccountSet, Clone, Debug, Deref, DerefMut)]
#[account_set(
    skip_default_idl,
    skip_default_validate,
    generics = [where T: AccountSet < 'info >]
)]
#[decode(generics = [<A> where T: AccountSetDecode<'a, 'info, A>], arg = A)]
#[validate(
    id = "create",
    generics = [
        <C> where T: AccountSetValidate<'info, ()> + SignedAccount<'info>
        + CanSetSeeds<'info, ()> + CanInitAccount<'info, Create<C>>
    ],
    arg = Create<C>,
    before_validation = {
        self.set_seeds(&(), syscalls)?;
        self.init(arg, syscalls, None)
    }
)]
#[validate(
    id = "create_generic",
    generics = [
        <C, A> where T: AccountSetValidate<'info, A> + SignedAccount<'info>
        + CanSetSeeds<'info, A> + CanInitAccount<'info, Create<C>>
    ],
    arg = (Create<C>, A),
    before_validation = {
        self.set_seeds(&arg.1, syscalls)?;
        self.init(arg.0, syscalls, None)
    }
)]
#[validate(
    id = "create_if_needed",
    generics = [
        <C> where T: AccountSetValidate<'info, ()> + SignedAccount<'info>
        + CanSetSeeds<'info, ()> + CanInitAccount<'info, CreateIfNeeded<C>>,
    ],
    arg = CreateIfNeeded<C>,
    before_validation = {
        self.set_seeds(&(), syscalls)?;
        self.init(arg, syscalls, None)
    }
)]
#[validate(
    id = "create_if_needed_generic",
    generics = [
        <C, A> where T: AccountSetValidate<'info, A> + SignedAccount<'info>
        + CanSetSeeds<'info, A> + CanInitAccount<'info, CreateIfNeeded<C>> +
    ],
    arg = (CreateIfNeeded<C>, A),
    before_validation = {
        self.set_seeds(&arg.1, syscalls)?;
        self.init(arg.0, syscalls, None)
    }
)]
#[cleanup(generics = [<A> where T: AccountSetCleanup <'info, A>], arg = A)]
pub struct Init<T>(
    #[decode(arg = arg)]
    #[validate(id = "create_generic", arg = arg.1)]
    #[validate(id = "create_if_needed_generic", arg = arg.1)]
    #[cleanup(arg = arg)]
    T,
);

impl<'info, T> SingleAccountSet<'info> for Init<T>
where
    T: SingleAccountSet<'info>,
{
    const METADATA: SingleAccountSetMetadata = SingleAccountSetMetadata {
        is_init: true,
        should_mut: true,
        ..T::METADATA
    };
    fn account_info(&self) -> &AccountInfo<'info> {
        self.0.account_info()
    }
}

impl<'info, T> SignedAccount<'info> for Init<T>
where
    T: SignedAccount<'info>,
{
    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        self.0.signer_seeds()
    }
}

impl<'info, T> WritableAccount<'info> for Init<T> where T: SingleAccountSet<'info> {}

impl<T> HasProgramAccount for Init<T>
where
    T: HasProgramAccount,
{
    type ProgramAccount = T::ProgramAccount;
}

impl<T> HasSeeds for Init<T>
where
    T: HasSeeds,
{
    type Seeds = T::Seeds;
}

pub trait InitCreateArg<'info> {
    type StarFrameInitArg;
    type FunderAccount: SignedAccount<'info> + WritableAccount<'info>;

    fn system_program(&self) -> &Program<'info, SystemProgram>;

    fn split<'a>(
        &'a mut self,
    ) -> CreateSplit<'a, 'info, Self::StarFrameInitArg, Self::FunderAccount>;
}
#[derive(Derivative)]
#[derivative(
    Debug(bound = "IA: Debug, FA: Debug",),
    Clone(bound = "IA: Clone"),
    Copy(bound = "IA: Copy")
)]
pub struct CreateSplit<'a, 'info: 'a, IA, FA> {
    pub arg: IA,
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
    type FunderAccount = WT;

    fn system_program(&self) -> &'a Program<'info, SystemProgram> {
        self.system_program
    }

    fn split<'b>(
        &'b mut self,
    ) -> CreateSplit<'b, 'info, Self::StarFrameInitArg, Self::FunderAccount> {
        CreateSplit {
            arg: self.arg.take().unwrap(),
            system_program: self.system_program,
            funder: self.funder,
        }
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, A, T: AccountSetToIdl<'info, A>> AccountSetToIdl<'info, A> for Init<T> {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            // manually mark as writable. Nothing else is needed for the IDL
            <Writable<T> as AccountSetToIdl<'info, A>>::account_set_to_idl(idl_definition, arg)
        }
    }
}
