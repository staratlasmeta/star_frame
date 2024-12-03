use crate::prelude::*;
use ref_cast::{ref_cast_custom, RefCastCustom};
use std::marker::PhantomData;

#[derive(AccountSet, Debug, Clone, RefCastCustom)]
#[validate(
    generics = [where T: StarFrameProgram],
    extra_validation = self.check_id(),
)]
#[idl(generics = [where T: StarFrameProgram])]
#[repr(transparent)]
pub struct Program<'info, T>(
    #[single_account_set]
    #[idl(address = T::PROGRAM_ID)]
    pub(crate) AccountInfo<'info>,
    #[account_set(skip = PhantomData)] pub(crate) PhantomData<T>,
);

pub trait InnerProgram {
    type Program: StarFrameProgram;
}

impl<T> InnerProgram for Program<'_, T>
where
    T: StarFrameProgram,
{
    type Program = T;
}

impl<'info, T: StarFrameProgram> Program<'info, T> {
    pub fn check_id(&self) -> Result<()> {
        if self.0.key() == &T::PROGRAM_ID {
            Ok(())
        } else {
            Err(ProgramError::IncorrectProgramId.into())
        }
    }

    /// Allows casting references from an `AccountInfo` without validating the program id.
    #[allow(dead_code)]
    #[ref_cast_custom]
    pub(crate) fn cast_info_unchecked<'a>(info: &'a AccountInfo<'info>) -> &'a Self;
}

// TODO: maybe add some helper methods here? Anchor has a program executable pda find method. Could be helpful to have here too.
