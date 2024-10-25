use crate::prelude::*;
use anyhow::Context;
use derivative::Derivative;
use derive_more::{Deref, DerefMut};
use star_frame::syscalls::SyscallAccountCache;
use star_frame_proc::AccountSet;
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
    #[single_account_set(skip_can_set_seeds, skip_can_init_account)]
    T,
);

use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Create<T>(pub T);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct CreateIfNeeded<T>(pub T);

#[derive(Derivative)]
#[derivative(Debug(bound = "A: Debug, Program<'info, SystemProgram>: Debug, WT: Debug"))]
pub struct CreateAccount<'info, A, WT> {
    pub(crate) arg: A,
    pub(crate) system_program: Program<'info, SystemProgram>,
    pub(crate) funder: WT,
}

impl<'info, WT: Clone> CreateAccount<'info, Zeroed, WT> {
    pub fn new(system_program: &Program<'info, SystemProgram>, funder: &WT) -> Self {
        Self::new_with_arg(Zeroed, system_program, funder)
    }
}

impl<'info> CreateAccount<'info, Zeroed, Funder<'info>> {
    pub fn new_from_syscalls(syscalls: &impl SyscallAccountCache<'info>) -> Result<Self> {
        Self::new_with_arg_from_syscalls(Zeroed, syscalls)
    }
}

impl<'info, A, WT: Clone> CreateAccount<'info, A, WT> {
    pub fn new_with_arg(
        arg: A,
        system_program: &Program<'info, SystemProgram>,
        funder: &WT,
    ) -> Self {
        Self {
            arg,
            system_program: system_program.clone(),
            funder: funder.clone(),
        }
    }
}

impl<'info, A> CreateAccount<'info, A, Funder<'info>> {
    pub fn new_with_arg_from_syscalls(
        arg: A,
        syscalls: &impl SyscallAccountCache<'info>,
    ) -> Result<Self> {
        let system_program = syscalls
            .get_system_program()
            .context("Missing `system_program` for CreateAccount auto")?;
        let funder = syscalls
            .get_funder()
            .context("Missing `funder` for CreateAccount auto")?;
        Ok(Self::new_with_arg(arg, system_program, funder))
    }
}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, A, T> AccountSetToIdl<'info, A> for Init<T>
    where
        T: AccountSetToIdl<'info, A> + SingleAccountSet<'info>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            // manually mark as writable. Nothing else is needed for the IDL. T Will be marked as signer automatically
            <Writable<T> as AccountSetToIdl<'info, A>>::account_set_to_idl(idl_definition, arg)
        }
    }
}
