//! Implementation of `AccountSet` traits for [`AccountInfo`].

use std::mem::MaybeUninit;

use crate::{
    account_set::{
        single_set::SingleSetMeta, AccountSetCleanup, AccountSetDecode, AccountSetValidate,
        ClientAccountSet, CpiAccountSet,
    },
    prelude::*,
};
use advancer::AdvanceArray;

impl SingleAccountSet for AccountInfo {
    #[inline]
    fn meta() -> SingleSetMeta {
        SingleSetMeta::default()
    }
    #[inline]
    fn account_info(&self) -> &AccountInfo {
        self
    }
    #[inline]
    fn account_meta(&self) -> AccountMeta {
        AccountMeta {
            pubkey: *SingleAccountSet::pubkey(self),
            is_signer: self.is_signer(),
            is_writable: self.is_writable(),
        }
    }
    #[inline]
    fn is_signer(&self) -> bool {
        self.is_signer()
    }
    #[inline]
    fn is_writable(&self) -> bool {
        self.is_writable()
    }
    #[inline]
    fn pubkey(&self) -> &Pubkey {
        bytemuck::cast_ref(self.key())
    }
}

impl ClientAccountSet for &AccountInfo {
    type ClientAccounts = Pubkey;
    const MIN_LEN: usize = 1;

    #[inline]
    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta {
            pubkey: *accounts,
            is_signer: false,
            is_writable: false,
        });
    }
}

impl ClientAccountSet for AccountInfo {
    type ClientAccounts = Pubkey;
    const MIN_LEN: usize = 1;

    #[inline]
    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta {
            pubkey: *accounts,
            is_signer: false,
            is_writable: false,
        });
    }
}

unsafe impl CpiAccountSet for AccountInfo {
    type ContainsOption = typenum::False;
    type CpiAccounts = AccountInfo;
    type AccountLen = typenum::U1;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        *self.account_info()
    }

    #[inline]
    fn write_account_infos<'a>(
        _program: Option<&'a AccountInfo>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        infos[*index] = MaybeUninit::new(accounts);
        *index += 1;
        Ok(())
    }

    #[inline]
    fn write_account_metas<'a>(
        _program_id: &Pubkey,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<PinocchioAccountMeta<'a>>],
    ) {
        metas[*index] = MaybeUninit::new(PinocchioAccountMeta {
            pubkey: accounts.key(),
            is_signer: false,
            is_writable: false,
        });
        *index += 1;
    }
}

unsafe impl CpiAccountSet for &AccountInfo {
    type ContainsOption = typenum::False;
    type CpiAccounts = AccountInfo;
    type AccountLen = typenum::U1;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        *self.account_info()
    }

    #[inline]
    fn write_account_infos<'a>(
        _program: Option<&'a AccountInfo>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountInfo>],
    ) -> Result<()> {
        infos[*index] = MaybeUninit::new(accounts);
        *index += 1;
        Ok(())
    }

    #[inline]
    fn write_account_metas<'a>(
        _program_id: &Pubkey,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<PinocchioAccountMeta<'a>>],
    ) {
        metas[*index] = MaybeUninit::new(PinocchioAccountMeta {
            pubkey: accounts.key(),
            is_signer: false,
            is_writable: false,
        });
        *index += 1;
    }
}

impl SingleAccountSet for &AccountInfo {
    #[inline]
    fn meta() -> SingleSetMeta {
        SingleSetMeta::default()
    }
    #[inline]
    fn account_info(&self) -> &AccountInfo {
        self
    }
}
impl<'a> AccountSetDecode<'a, ()> for AccountInfo {
    #[inline]
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        _decode_input: (),
        _ctx: &mut Context,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts
            .try_advance_array()
            .ctx("Not enough accounts to decode AccountInfo")?;
        Ok(account[0])
    }
}
impl<'a> AccountSetDecode<'a, ()> for &'a AccountInfo {
    #[inline]
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        _decode_input: (),
        _ctx: &mut Context,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts
            .try_advance_array()
            .ctx("Not enough accounts to decode AccountInfo")?;
        Ok(&account[0])
    }
}
impl AccountSetValidate<()> for AccountInfo {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn validate_accounts(&mut self, _validate_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl AccountSetValidate<()> for &AccountInfo {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn validate_accounts(&mut self, _validate_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl AccountSetCleanup<()> for AccountInfo {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn cleanup_accounts(&mut self, _cleanup_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}
impl AccountSetCleanup<()> for &AccountInfo {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn cleanup_accounts(&mut self, _cleanup_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl_impl {
    use super::*;
    use crate::idl::{AccountSetToIdl, FindIdlSeeds};
    use star_frame_idl::{
        account_set::{IdlAccountSetDef, IdlSingleAccountSet},
        seeds::IdlFindSeeds,
        IdlDefinition,
    };

    impl AccountSetToIdl<()> for AccountInfo {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            _arg: (),
        ) -> crate::IdlResult<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet::default()))
        }
    }

    impl<T> AccountSetToIdl<(T, Pubkey)> for AccountInfo
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: (T, Pubkey),
        ) -> crate::IdlResult<IdlAccountSetDef> {
            let (seeds, program) = arg;
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                seeds: Some(IdlFindSeeds {
                    seeds: T::find_seeds(&seeds)?,
                    program: Some(program),
                }),
                ..Default::default()
            }))
        }
    }

    impl<T> AccountSetToIdl<T> for AccountInfo
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: T,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                seeds: Some(IdlFindSeeds {
                    seeds: T::find_seeds(&arg)?,
                    program: None,
                }),
                ..Default::default()
            }))
        }
    }

    impl<A> AccountSetToIdl<A> for &AccountInfo
    where
        AccountInfo: AccountSetToIdl<A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            AccountInfo::account_set_to_idl(idl_definition, arg)
        }
    }
}
