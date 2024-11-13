use crate::prelude::*;
use derive_more::{Deref, DerefMut};

#[derive(AccountSet, Clone, Debug, Deref, DerefMut)]
#[repr(transparent)]
#[account_set(skip_default_idl, skip_default_validate)]
#[validate(
    id = "create",
    generics = [<C> where T: CanInitSeeds<'info, ()> + CanInitAccount<'info, Create<C>>],
    arg = Create<C>,
    before_validation = {
        self.init_seeds(&(), syscalls)?;
        self.init_account(arg, syscalls, None)
    }
)]
#[validate(
    id = "create_generic",
    generics = [<C, A> where T: CanInitSeeds<'info, A> + CanInitAccount<'info, Create<C>>],
    arg = (Create<C>, A),
    before_validation = {
        self.init_seeds(&arg.1, syscalls)?;
        self.init_account(arg.0, syscalls, None)
    }
)]
#[validate(
    id = "create_if_needed",
    generics = [<C> where T: CanInitSeeds<'info, ()> + CanInitAccount<'info, CreateIfNeeded<C>>],
    arg = CreateIfNeeded<C>,
    before_validation = {
        self.init_seeds(&(), syscalls)?;
        self.init_account(arg, syscalls, None)
    }
)]
#[validate(
    id = "create_if_needed_generic",
    generics = [<C, A> where T: CanInitSeeds<'info, A> + CanInitAccount<'info, CreateIfNeeded<C>>],
    arg = (CreateIfNeeded<C>, A),
    before_validation = {
        self.init_seeds(&arg.1, syscalls)?;
        self.init_account(arg.0, syscalls, None)
    }
)]
pub struct Init<T>(
    #[single_account_set(writable, skip_can_init_seeds, skip_can_init_account)]
    #[validate(id = "create_generic", arg = arg.1)]
    #[validate(id = "create_if_needed_generic", arg = arg.1)]
    T,
);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Create<T>(pub T);
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct CreateIfNeeded<T>(pub T);

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use anyhow::bail;
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
            let mut set =
                <Mut<T> as AccountSetToIdl<'info, A>>::account_set_to_idl(idl_definition, arg)?;
            let single = set.single()?;
            if single.is_init {
                bail!(
                    "Account set is already wrapped with `Init`! Got {:?}",
                    single
                );
            }
            set.single()?.is_init = true;
            Ok(set)
        }
    }
}
