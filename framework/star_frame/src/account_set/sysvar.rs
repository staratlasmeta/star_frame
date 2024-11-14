use solana_program::sysvar::{Sysvar as SysvarTrait, SysvarId};
use star_frame::prelude::*;
use std::marker::PhantomData;

#[derive(AccountSet, Debug, Clone)]
#[idl(generics = [])]
#[validate(generics = [])]
pub struct Sysvar<'info, T>
where
    T: SysvarId,
{
    #[single_account_set]
    #[idl(arg = T::id())]
    #[validate(arg = &T::id())]
    info: AccountInfo<'info>,
    #[account_set(skip = PhantomData)]
    phantom_t: PhantomData<fn() -> T>,
}

impl<'info, T> Sysvar<'info, T>
where
    T: SysvarTrait,
{
    pub fn from_account_info(info: &AccountInfo<'info>) -> Result<T> {
        let sysvar = T::from_account_info(info)?;
        Ok(sysvar)
    }
}
