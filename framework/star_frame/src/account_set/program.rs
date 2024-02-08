use crate::program::StarFrameProgram;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use star_frame::account_set::{AccountSet, SingleAccountSet};
use std::marker::PhantomData;

#[derive(AccountSet, Debug)]
#[validate(
    generics = [where T: StarFrameProgram],
    extra_validation = if self.0.key() == &T::program_id(sys_calls)? { Ok(()) } else { Err(ProgramError::IncorrectProgramId.into()) },
)]
pub struct Program<'info, T>(AccountInfo<'info>, PhantomData<T>);

impl<'info, T> SingleAccountSet<'info> for Program<'info, T> {
    fn account_info(&self) -> &AccountInfo<'info> {
        &self.0
    }
}

// TODO: maybe add some helper methods here? Anchor has a program executable pda find method. Could be helpful to have here too.
