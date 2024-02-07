use crate::prelude::*;
use std::ops::{Deref, DerefMut};

pub trait SeededAccountData: ProgramAccount {
    type Seeds: GetSeeds;
}

#[derive(AccountSet, Debug)]
#[validate(arg = (T::Seeds,))]
#[validate(id = "wo_bump", arg = Seeds < T::Seeds >)]
#[validate(id = "with_bump", arg = SeedsWithBump < T::Seeds >)]
pub struct SeededDataAccount<'info, T>(
    #[validate(arg = (arg.0, ()))]
    #[validate(id = "wo_bump", arg = (arg.0, ()))]
    #[validate(id = "with_bump", arg = (arg, ()))]
    SeededAccount<DataAccount<'info, T>, T::Seeds>,
)
where
    T: SeededAccountData + UnsizedType;

impl<'info, T> Deref for SeededDataAccount<'info, T>
where
    T: SeededAccountData + UnsizedType,
{
    type Target = SeededAccount<DataAccount<'info, T>, T::Seeds>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'info, T> DerefMut for SeededDataAccount<'info, T>
where
    T: SeededAccountData + UnsizedType,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
