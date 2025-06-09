use pinocchio::account_info::Ref;
use star_frame::prelude::*;
use std::marker::PhantomData;

pub trait SysvarId: Sized {
    fn id() -> Pubkey;
}

impl SysvarId for pinocchio::sysvars::rent::Rent {
    fn id() -> Pubkey {
        bytemuck::cast(pinocchio::sysvars::rent::RENT_ID)
    }
}

pub const RECENT_BLOCKHASHES_ID: Pubkey = pubkey!("SysvarRecentB1ockHashes11111111111111111111");

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct InstructionsSysvar;

impl SysvarId for InstructionsSysvar {
    fn id() -> Pubkey {
        bytemuck::cast(pinocchio::sysvars::instructions::INSTRUCTIONS_ID)
    }
}

#[derive(AccountSet, derive_where::DeriveWhere)]
#[derive_where(Clone, Copy, Debug)]
#[idl(generics = [])]
#[validate(generics = [])]
pub struct Sysvar<T>
where
    T: SysvarId,
{
    #[single_account_set]
    #[idl(address = T::id())]
    #[validate(address = &T::id())]
    info: AccountInfo,
    #[account_set(skip = PhantomData)]
    phantom_t: PhantomData<fn() -> T>,
}

impl Sysvar<InstructionsSysvar> {
    pub fn instructions(
        &self,
    ) -> Result<pinocchio::sysvars::instructions::Instructions<Ref<'_, [u8]>>> {
        (&self.info).try_into().map_err(Into::into)
    }
}
