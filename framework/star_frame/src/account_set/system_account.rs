use derive_more::{Deref, DerefMut};
use solana_program::system_program;
use star_frame::prelude::*;

#[derive(AccountSet, Debug, Deref, DerefMut, Clone)]
#[validate(
    generics = [<A> where AccountInfo<'info>: AccountSetValidate<'info, A>], arg = A,
    extra_validation = self.check_id(),
)]
#[repr(transparent)]
pub struct SystemAccount<'info>(#[validate(arg = arg)] AccountInfo<'info>);

impl<'info> SingleAccountSet<'info> for SystemAccount<'info> {
    const METADATA: SingleAccountSetMetadata = SingleAccountSetMetadata::DEFAULT;

    fn account_info(&self) -> &AccountInfo<'info> {
        &self.0
    }
}

impl SystemAccount<'_> {
    pub fn check_id(&self) -> Result<()> {
        if self.0.owner() == &system_program::ID {
            Ok(())
        } else {
            Err(ProgramError::IllegalOwner.into())
        }
    }
}
