use crate::prelude::*;
use std::marker::PhantomData;

#[derive(AccountSet, Debug)]
#[validate(
    generics = [where T: StarFrameProgram],
    extra_validation = self.check_id(),
)]
pub struct Program<'info, T>(AccountInfo<'info>, PhantomData<T>);

impl<'info, T> SingleAccountSet<'info> for Program<'info, T> {
    fn account_info(&self) -> &AccountInfo<'info> {
        &self.0
    }
}

impl<T: StarFrameProgram> Program<'_, T> {
    pub fn check_id(&self) -> Result<()> {
        if self.0.key() == &T::PROGRAM_ID {
            Ok(())
        } else {
            Err(ProgramError::IncorrectProgramId.into())
        }
    }
}

// TODO: maybe add some helper methods here? Anchor has a program executable pda find method. Could be helpful to have here too.
