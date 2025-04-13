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
    #[idl(address = T::id())]
    #[validate(address = &T::id())]
    info: AccountInfo<'info>,
    #[account_set(skip = PhantomData)]
    phantom_t: PhantomData<fn() -> T>,
}

impl<T> Sysvar<'_, T>
where
    T: SysvarTrait,
{
    pub fn deserialize(&self) -> Result<T> {
        let sysvar = T::from_account_info(&self.info)?;
        Ok(sysvar)
    }
}
