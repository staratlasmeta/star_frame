use crate::prelude::*;
use advancer::AdvanceArray;
use anyhow::Context as _;
use ref_cast::{ref_cast_custom, RefCastCustom};
use std::marker::PhantomData;

#[derive(AccountSet, Debug, RefCastCustom, derive_where::DeriveWhere)]
#[derive_where(Clone, Copy)]
#[account_set(skip_default_decode, skip_client_account_set)]
#[validate(
    extra_validation = self.check_id(),
)]
#[repr(transparent)]
pub struct Program<T: StarFrameProgram>(
    #[single_account_set]
    #[idl(address = T::ID)]
    pub(crate) AccountInfo,
    #[account_set(skip = PhantomData)] pub(crate) PhantomData<T>,
);

impl<'a, T> AccountSetDecode<'a, ()> for Program<T>
where
    T: StarFrameProgram,
{
    fn decode_accounts(
        accounts: &mut &'a [AccountInfo],
        _decode_input: (),
        ctx: &mut Context,
    ) -> Result<Self> {
        let account: &[_; 1] = accounts
            .try_advance_array()
            .context("Not enough accounts")?;
        let account = account[0];

        ctx.add_program(*account.pubkey(), account);
        Ok(Self(account, PhantomData))
    }
}

impl<T: StarFrameProgram> ClientAccountSet for Program<T> {
    type ClientAccounts = Option<Pubkey>;

    const MIN_LEN: usize = 1;

    fn extend_account_metas(
        _program_id: &Pubkey,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta::new_readonly(accounts.unwrap_or(T::ID), false));
    }
}

pub trait InnerProgram {
    type Program: StarFrameProgram;
}

impl<T> InnerProgram for Program<T>
where
    T: StarFrameProgram,
{
    type Program = T;
}

impl<T: StarFrameProgram> Program<T> {
    pub fn check_id(&self) -> Result<()> {
        if self.0.pubkey() == &T::ID {
            Ok(())
        } else {
            Err(ProgramError::IncorrectProgramId.into())
        }
    }

    /// Allows casting references from an `AccountInfo` without validating the program id.
    #[allow(dead_code)]
    #[ref_cast_custom]
    pub(crate) fn cast_info_unchecked<'a>(info: &'a AccountInfo) -> &'a Self;
}

// TODO: maybe add some helper methods here? Anchor has a program executable pda find method. Could be helpful to have here too.
