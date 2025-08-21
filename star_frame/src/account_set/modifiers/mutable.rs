use crate::{
    account_set::{AccountSet, SingleAccountSet, WritableAccount},
    prelude::SingleSetMeta,
};
use derive_more::{Deref, DerefMut};

/// A potentially mutable account, contingent on the `MUT` const generic being true.
#[derive(AccountSet, Copy, Clone, Debug, Deref, DerefMut)]
#[account_set(skip_default_idl)]
#[validate(
    extra_validation = if MUT { self.check_writable() } else { Ok(()) }
)]
#[repr(transparent)]
pub struct MaybeMut<const MUT: bool, T>(
    #[single_account_set(meta = SingleSetMeta { writable: MUT, ..T::meta() }, skip_writable_account)]
    pub(crate) T,
);

/// A mutable account
pub type Mut<T> = MaybeMut<true, T>;

impl<T> WritableAccount for MaybeMut<true, T> where T: SingleAccountSet {}

// A false MaybeMut just acts as a pass-through, so we need to pass this through!
impl<T> WritableAccount for MaybeMut<false, T> where T: WritableAccount {}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use crate::Result;
    use star_frame::idl::AccountSetToIdl;
    use star_frame_idl::{account_set::IdlAccountSetDef, IdlDefinition};

    impl<const MUT: bool, T, A> AccountSetToIdl<A> for MaybeMut<MUT, T>
    where
        T: AccountSetToIdl<A> + SingleAccountSet,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> Result<IdlAccountSetDef> {
            let mut set = T::account_set_to_idl(idl_definition, arg)?;
            if MUT {
                set.single()?.writable = true;
            }
            Ok(set)
        }
    }
}
