//! Implementation of `AccountSet` traits for [`AccountView`].

use core::mem::MaybeUninit;

use crate::{
    account_set::{
        single_set::SingleSetMeta, AccountSetCleanup, AccountSetDecode, AccountSetValidate,
        CpiAccountSet,
    },
    prelude::*,
};
use advancer::AdvanceArray;

impl SingleAccountSet for AccountView {
    #[inline]
    fn meta() -> SingleSetMeta {
        SingleSetMeta::default()
    }
    #[inline]
    fn account_info(&self) -> &AccountView {
        self
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
    fn address(&self) -> &Address {
        self.address()
    }
}

#[cfg(not(target_os = "solana"))]
impl crate::account_set::ClientAccountSet for &AccountView {
    type ClientAccounts = Address;
    const MIN_LEN: usize = 1;

    #[inline]
    fn extend_account_metas(
        _program_id: &Address,
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

#[cfg(not(target_os = "solana"))]
impl crate::account_set::ClientAccountSet for AccountView {
    type ClientAccounts = Address;
    const MIN_LEN: usize = 1;

    #[inline]
    fn extend_account_metas(
        _program_id: &Address,
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

unsafe impl CpiAccountSet for AccountView {
    type ContainsOption = typenum::False;
    type CpiAccounts = AccountView;
    type AccountLen = typenum::U1;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        *self.account_info()
    }

    #[inline]
    fn write_account_infos<'a>(
        _program: Option<&'a AccountView>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountView>],
    ) -> Result<()> {
        infos[*index] = MaybeUninit::new(accounts);
        *index += 1;
        Ok(())
    }

    #[inline]
    fn write_account_metas<'a>(
        _program_id: &Address,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<InstructionAccount<'a>>],
    ) {
        metas[*index] = MaybeUninit::new(InstructionAccount {
            address: accounts.address(),
            is_signer: false,
            is_writable: false,
        });
        *index += 1;
    }
}

unsafe impl CpiAccountSet for &AccountView {
    type ContainsOption = typenum::False;
    type CpiAccounts = AccountView;
    type AccountLen = typenum::U1;

    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        *self.account_info()
    }

    #[inline]
    fn write_account_infos<'a>(
        _program: Option<&'a AccountView>,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        infos: &mut [MaybeUninit<&'a AccountView>],
    ) -> Result<()> {
        infos[*index] = MaybeUninit::new(accounts);
        *index += 1;
        Ok(())
    }

    #[inline]
    fn write_account_metas<'a>(
        _program_id: &Address,
        accounts: &'a Self::CpiAccounts,
        index: &mut usize,
        metas: &mut [MaybeUninit<InstructionAccount<'a>>],
    ) {
        metas[*index] = MaybeUninit::new(InstructionAccount {
            address: accounts.address(),
            is_signer: false,
            is_writable: false,
        });
        *index += 1;
    }
}

impl SingleAccountSet for &AccountView {
    #[inline]
    fn meta() -> SingleSetMeta {
        SingleSetMeta::default()
    }
    #[inline]
    fn account_info(&self) -> &AccountView {
        self
    }
}
impl<'a> AccountSetDecode<'a, ()> for AccountView {
    #[inline]
    fn decode_accounts(
        accounts: &mut &'a [AccountView],
        _decode_input: (),
        _ctx: &mut Context,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts
            .try_advance_array()
            .ctx("Not enough accounts to decode AccountView")?;
        Ok(account[0])
    }
}
impl<'a> AccountSetDecode<'a, ()> for &'a AccountView {
    #[inline]
    fn decode_accounts(
        accounts: &mut &'a [AccountView],
        _decode_input: (),
        _ctx: &mut Context,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts
            .try_advance_array()
            .ctx("Not enough accounts to decode AccountView")?;
        Ok(&account[0])
    }
}
impl AccountSetValidate<()> for AccountView {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn validate_accounts(&mut self, _validate_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl AccountSetValidate<()> for &AccountView {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn validate_accounts(&mut self, _validate_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl AccountSetCleanup<()> for AccountView {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn cleanup_accounts(&mut self, _cleanup_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}
impl AccountSetCleanup<()> for &AccountView {
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

    impl AccountSetToIdl<()> for AccountView {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            _arg: (),
        ) -> crate::IdlResult<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet::default()))
        }
    }

    impl<T> AccountSetToIdl<Seeds<(T, Address)>> for AccountView
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: Seeds<(T, Address)>,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            let (seeds, program) = arg.0;
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                seeds: Some(IdlFindSeeds {
                    seeds: T::find_seeds(&seeds)?,
                    program: Some(program),
                }),
                ..Default::default()
            }))
        }
    }

    impl<T> AccountSetToIdl<Seeds<T>> for AccountView
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: Seeds<T>,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                seeds: Some(IdlFindSeeds {
                    seeds: T::find_seeds(&arg.0)?,
                    program: None,
                }),
                ..Default::default()
            }))
        }
    }

    impl<A> AccountSetToIdl<A> for &AccountView
    where
        AccountView: AccountSetToIdl<A>,
    {
        fn account_set_to_idl(
            idl_definition: &mut IdlDefinition,
            arg: A,
        ) -> crate::IdlResult<IdlAccountSetDef> {
            AccountView::account_set_to_idl(idl_definition, arg)
        }
    }
}
