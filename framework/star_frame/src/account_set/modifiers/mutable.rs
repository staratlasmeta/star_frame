use crate::account_set::{AccountSet, SingleAccountSet, WritableAccount};
use derive_more::{Deref, DerefMut};

#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[account_set(skip_default_idl)]
#[validate(
    extra_validation = self.check_writable(),
)]
#[repr(transparent)]
pub struct Mut<T>(#[single_account_set(skip_writable_account)] pub(crate) T);

impl<'info, T> WritableAccount<'info> for Mut<T> where T: SingleAccountSet<'info> {}

#[cfg(feature = "idl")]
mod idl_impl {
    use super::*;
    use crate::Result;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, T, A> AccountSetToIdl<'info, A> for Mut<T>
    where
        T: AccountSetToIdl<'info, A> + SingleAccountSet<'info>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            let mut set = T::account_set_to_idl(idl_definition, arg)?;
            set.single()?.writable = true;
            Ok(set)
        }
    }
}
