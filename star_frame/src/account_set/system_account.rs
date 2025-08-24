use derive_more::{Deref, DerefMut};
use star_frame::prelude::*;

use crate::account_set::HasOwnerProgram;

#[derive(AccountSet, Debug, Deref, DerefMut, Clone, Copy)]
#[validate(extra_validation = self.check_id())]
#[repr(transparent)]
pub struct SystemAccount(#[single_account_set(skip_has_owner_program)] AccountInfo);

impl SystemAccount {
    pub fn check_id(&self) -> Result<()> {
        if self.owner_pubkey() == System::ID {
            Ok(())
        } else {
            Err(ProgramError::IllegalOwner.into())
        }
    }
}

impl HasOwnerProgram for SystemAccount {
    type OwnerProgram = System;
}
