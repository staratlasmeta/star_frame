//! A single account that is owned by the system program.
use derive_more::{Deref, DerefMut};
use star_frame::prelude::*;

use crate::account_set::HasOwnerProgram;

/// A single account that is owned by the system program.
#[derive(AccountSet, Debug, Deref, DerefMut, Clone, Copy)]
#[validate(extra_validation = self.check_id())]
#[repr(transparent)]
pub struct SystemAccount(#[single_account_set(skip_has_owner_program)] AccountView);

impl SystemAccount {
    #[inline]
    pub fn check_id(&self) -> Result<()> {
        // SAFETY:
        // The reference is immediately used and dropped, so we don't need to worry about it being used after the function returns
        if unsafe { self.owner() }.fast_eq(&System::ID) {
            Ok(())
        } else {
            Err(ProgramError::IllegalOwner.into())
        }
    }
}

impl HasOwnerProgram for SystemAccount {
    type OwnerProgram = System;
}
