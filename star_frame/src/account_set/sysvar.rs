use star_frame::prelude::*;
use std::{marker::PhantomData, ops::Deref};

pub trait SysvarId: Sized {
    fn id() -> Pubkey;
}

impl SysvarId for pinocchio::sysvars::rent::Rent {
    fn id() -> Pubkey {
        bytemuck::cast(pinocchio::sysvars::rent::RENT_ID)
    }
}

pub const RECENT_BLOCKHASHES_ID: Pubkey = pubkey!("SysvarRecentB1ockHashes11111111111111111111");

impl<T> SysvarId for pinocchio::sysvars::instructions::Instructions<T>
where
    T: Deref<Target = [u8]>,
{
    fn id() -> Pubkey {
        bytemuck::cast(pinocchio::sysvars::instructions::INSTRUCTIONS_ID)
    }
}

impl SysvarId for pinocchio::sysvars::clock::Clock {
    fn id() -> Pubkey {
        unimplemented!("Get the clock sysvar ID into pinocchio!")
    }
}
impl SysvarId for pinocchio::sysvars::fees::Fees {
    fn id() -> Pubkey {
        unimplemented!("Get the fees sysvar ID into pinocchio!")
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
