//! Typed wrapper for program accounts in instruction contexts.
//!
//! The `Program<T>` type represents a reference to a specific Star Frame program within an instruction's
//! account set. It automatically validates that the provided account matches the expected program ID
//! and provides type-safe access to program-specific functionality.

use crate::prelude::*;
use core::marker::PhantomData;
use ref_cast::{ref_cast_custom, RefCastCustom};

/// A typed wrapper for a program account that validates the program ID matches the expected ID.
///
/// This type is used in instruction account sets to represent a specific Star Frame program account.
/// During validation, it ensures the provided account's public key matches `T::ID`, preventing
/// incorrect program references in cross-program invocations or instruction contexts.
#[derive(AccountSet, Debug, RefCastCustom, derive_where::DeriveWhere)]
#[derive_where(Clone, Copy)]
#[account_set(skip_client_account_set)]
#[validate(
    extra_validation = self.check_id(),
)]
#[repr(transparent)]
pub struct Program<T: StarFrameProgram>(
    #[single_account_set]
    #[idl(address = T::ID)]
    pub(crate) AccountView,
    #[account_set(skip = PhantomData)] pub(crate) PhantomData<T>,
);

#[cfg(not(target_os = "solana"))]
impl<T: StarFrameProgram> crate::account_set::ClientAccountSet for Program<T> {
    type ClientAccounts = Option<Address>;

    const MIN_LEN: usize = 1;

    fn extend_account_metas(
        _program_id: &Address,
        accounts: &Self::ClientAccounts,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta::new_readonly(accounts.unwrap_or(T::ID), false));
    }
}

impl<T: StarFrameProgram> Program<T> {
    pub fn check_id(&self) -> Result<()> {
        ensure!(self.0.address() == &T::ID, ProgramError::IncorrectProgramId);
        Ok(())
    }

    /// Allows casting references from an `AccountView` without validating the program id.
    #[allow(dead_code)]
    #[ref_cast_custom]
    pub(crate) fn cast_info_unchecked<'a>(info: &'a AccountView) -> &'a Self;
}
