use crate::prelude::*;
use anyhow::Context as _;
use derive_more::{Deref, DerefMut};

#[derive(AccountSet, Clone, Debug, Deref, DerefMut)]
#[repr(transparent)]
#[account_set(skip_default_idl, skip_default_validate)]
#[validate(
    id = "create",
    generics = [<C> where T: CanInitSeeds<()> + CanInitAccount<C>],
    arg = Create<C>,
    before_validation = {
        self.init_seeds(&(), ctx).context("Failed to init seeds")?;
        self.init_account::<false>(arg.0, None, ctx).context("Failed to init account")
    }
)]
#[validate(
    id = "create_generic",
    generics = [<C, A> where T: CanInitSeeds<A> + CanInitAccount<C>],
    arg = (Create<C>, A),
    before_validation = {
        self.init_seeds(&arg.1, ctx).context("Failed to init seeds")?;
        self.init_account::<false>(arg.0.0, None, ctx).context("Failed to init account")
    }
)]
#[validate(
    id = "create_if_needed",
    generics = [<C> where T: CanInitSeeds<()> + CanInitAccount<C>],
    arg = CreateIfNeeded<C>,
    before_validation = {
        self.init_seeds(&(), ctx).context("Failed to init seeds")?;
        self.init_account::<true>(arg.0, None, ctx).context("Failed to init account")
    }
)]
#[validate(
    id = "create_if_needed_generic",
    generics = [<C, A> where T: CanInitSeeds<A> + CanInitAccount<C>],
    arg = (CreateIfNeeded<C>, A),
    before_validation = {
        self.init_seeds(&arg.1, ctx).context("Failed to init seeds")?;
        self.init_account::<true>(arg.0.0, None, ctx).context("Failed to init account")
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

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use anyhow::bail;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

    impl<A, T> AccountSetToIdl<A> for Init<T>
    where
        T: AccountSetToIdl<A> + SingleAccountSet,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            let mut set = <Mut<T> as AccountSetToIdl<A>>::account_set_to_idl(idl_definition, arg)?;
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
