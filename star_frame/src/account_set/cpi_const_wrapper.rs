use super::CpiAccountSet;
use crate::prelude::*;
use std::mem::MaybeUninit;

fn internal_only_unimplemented() -> ! {
    unimplemented!("CpiConstWrapper is an internal macro helper and must not be called directly")
}

/// Internal helper type used by proc-macro-generated `CpiAccountSet` trait bounds 
/// due to an issue with the rustc trait solver
/// 
/// See [discussion in Discord](https://discord.com/channels/273534239310479360/1120124565591425034/1414399888006971432) for details.
#[doc(hidden)]
#[derive(Debug)]
pub struct CpiConstWrapper<T, const N: usize>(T);

unsafe impl<T, const N: usize> CpiAccountSet for CpiConstWrapper<T, N>
where
    T: CpiAccountSet,
{
    type CpiAccounts = T::CpiAccounts;
    type ContainsOption = T::ContainsOption;
    type AccountLen = T::AccountLen;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        internal_only_unimplemented()
    }

    #[inline]
    fn write_account_infos<'a>(
        _program: Option<&'a AccountInfo>,
        _accounts: &'a Self::CpiAccounts,
        _index: &mut usize,
        _infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        internal_only_unimplemented()
    }

    #[inline]
    fn write_account_metas<'a>(
        _program_id: &'a Pubkey,
        _accounts: &'a Self::CpiAccounts,
        _index: &mut usize,
        _metas: &mut [MaybeUninit<PinocchioAccountMeta<'a>>],
    ) {
        internal_only_unimplemented();
    }
}
