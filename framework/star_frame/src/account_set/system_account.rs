use derive_more::{Deref, DerefMut};
use solana_program::system_program;
use star_frame::prelude::*;

#[derive(AccountSet, Debug, Deref, DerefMut, Clone)]
#[validate(extra_validation = self.check_id())]
#[repr(transparent)]
pub struct SystemAccount<'info>(#[single_account_set(skip_has_owner_program)] AccountInfo<'info>);

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
