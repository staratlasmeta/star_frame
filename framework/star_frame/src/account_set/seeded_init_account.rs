use crate::prelude::{InitAccount, SeededAccount, SeededAccountData, UnsizedType};
use star_frame_proc::AccountSet;

#[derive(AccountSet)]
pub struct SeededInitAccount<'info, T>(SeededAccount<InitAccount<'info, T>, T::Seeds>)
where
    T: SeededAccountData + UnsizedType + ?Sized;
