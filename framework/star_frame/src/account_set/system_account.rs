use derive_more::{Deref, DerefMut};
use solana_program::system_program;
use star_frame::prelude::*;

#[derive(AccountSet, Debug, Deref, DerefMut, Clone)]
#[account_set(skip_default_idl)]
#[validate(
    generics = [<A> where AccountInfo<'info>: AccountSetValidate<'info, A>], arg = A,
    extra_validation = self.check_id(),
)]
#[repr(transparent)]
pub struct SystemAccount<'info>(
    #[single_account_set(skip_has_owner_program)]
    #[validate(arg = arg)]
    AccountInfo<'info>,
);

impl SystemAccount<'_> {
    pub fn check_id(&self) -> Result<()> {
        if self.0.owner() == &system_program::ID {
            Ok(())
        } else {
            Err(ProgramError::IllegalOwner.into())
        }
    }
}

impl HasOwnerProgram for SystemAccount<'_> {
    type OwnerProgram = SystemProgram;
}

#[cfg(feature = "idl")]
mod idl_impl {
    use crate::idl::AccountSetToIdl;
    use crate::prelude::SystemAccount;
    use solana_program::account_info::AccountInfo;
    use star_frame_idl::account_set::IdlAccountSetDef;
    use star_frame_idl::IdlDefinition;

    impl<'info, A> AccountSetToIdl<'info, A> for SystemAccount<'info>
    where
        AccountInfo<'info>: AccountSetToIdl<'info, A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> anyhow::Result<IdlAccountSetDef> {
            <AccountInfo<'info>>::account_set_to_idl(idl_definition, arg)
        }
    }
}
