use crate::{
    account_set::{AccountSetDecode, SingleAccountSet, SingleSetMeta},
    client::ClientAccountSet,
    prelude::{Context, CpiAccountSet},
    Result,
};
use advancer::AdvanceArray;
use anyhow::Context as _;
use pinocchio::account_info::AccountInfo;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;
use star_frame::account_set::{AccountSetCleanup, AccountSetValidate};

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

impl CpiAccountSet for &AccountInfo {
    type CpiAccounts = AccountInfo;
    const MIN_LEN: usize = 1;

    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        *self.account_info()
    }

    fn extend_account_infos(
        _program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        _ctx: &Context,
    ) -> Result<()> {
        infos.push(accounts);
        Ok(())
    }

    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta {
            pubkey: *SingleAccountSet::pubkey(&accounts),
            is_signer: false,
            is_writable: false,
        });
    }
}

impl CpiAccountSet for AccountInfo {
    type CpiAccounts = AccountInfo;
    const MIN_LEN: usize = 1;

    fn to_cpi_accounts(&self) -> Self::CpiAccounts {
        *self.account_info()
    }

    fn extend_account_infos(
        _program_id: &Pubkey,
        accounts: Self::CpiAccounts,
        infos: &mut Vec<AccountInfo>,
        _ctx: &Context,
    ) -> Result<()> {
        infos.push(accounts);
        Ok(())
    }

    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::CpiAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta {
            pubkey: *accounts.pubkey(),
            is_signer: false,
            is_writable: false,
        });
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
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        _decode_input: (),
        _ctx: &mut Context,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts
            .try_advance_array()
            .context("Not enough accounts to decode AccountInfo")?;
        Ok(account[0])
    }
}
impl<'a> AccountSetDecode<'a, ()> for &'a AccountInfo {
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        _decode_input: (),
        _ctx: &mut Context,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts
            .try_advance_array()
            .context("Not enough accounts to decode AccountInfo")?;
        Ok(&account[0])
    }
}
impl AccountSetValidate<()> for AccountInfo {
    fn validate_accounts(&mut self, _validate_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

impl AccountSetValidate<()> for &AccountInfo {
    fn validate_accounts(&mut self, validate_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(validate_input)
    }
}

impl AccountSetCleanup<()> for AccountInfo {
    fn cleanup_accounts(&mut self, cleanup_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(cleanup_input)
    }
}
impl AccountSetCleanup<()> for &AccountInfo {
    fn cleanup_accounts(&mut self, cleanup_input: (), _ctx: &mut Context) -> Result<()> {
        Ok(cleanup_input)
    }
}
#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub mod idl_impl {
    use super::*;
    use crate::{
        idl::{AccountSetToIdl, FindIdlSeeds},
        prelude::Seeds,
    };
    use star_frame_idl::{
        account_set::{IdlAccountSetDef, IdlSingleAccountSet},
        seeds::IdlFindSeeds,
        IdlDefinition,
    };

    impl AccountSetToIdl<()> for AccountInfo {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            _arg: (),
        ) -> Result<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet::default()))
        }
    }

    impl<T> AccountSetToIdl<Seeds<(T, Pubkey)>> for AccountInfo
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: Seeds<(T, Pubkey)>,
        ) -> Result<IdlAccountSetDef> {
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

    impl<T> AccountSetToIdl<Seeds<T>> for AccountInfo
    where
        T: FindIdlSeeds,
    {
        fn account_set_to_idl(
            _idl_definition: &mut IdlDefinition,
            arg: Seeds<T>,
        ) -> Result<IdlAccountSetDef> {
            Ok(IdlAccountSetDef::Single(IdlSingleAccountSet {
                seeds: Some(IdlFindSeeds {
                    seeds: T::find_seeds(&arg.0)?,
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
        ) -> Result<IdlAccountSetDef> {
            AccountInfo::account_set_to_idl(idl_definition, arg)
        }
    }
}
