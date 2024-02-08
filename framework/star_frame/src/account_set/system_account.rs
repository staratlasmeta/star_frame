use derive_more::{Deref, DerefMut};
use solana_program::system_program;
use star_frame::prelude::*;

#[derive(AccountSet, Debug, Deref, DerefMut)]
#[validate(
    extra_validation = if self.0.owner() == & system_program::ID { Ok(()) } else { Err(ProgramError::IllegalOwner.into()) }
)]
#[repr(transparent)]
pub struct SystemAccount<'info>(AccountInfo<'info>);

impl<'info> SingleAccountSet<'info> for SystemAccount<'info> {
    fn account_info(&self) -> &AccountInfo<'info> {
        &self.0
    }
}
