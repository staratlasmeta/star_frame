use solana_program::account_info::AccountInfo;
use star_frame_proc::AccountSet;
use std::marker::PhantomData;

#[derive(AccountSet, Debug)]
pub struct InitAccount<'info, T> {
    info: AccountInfo<'info>,
    phantom_t: PhantomData<T>,
}
