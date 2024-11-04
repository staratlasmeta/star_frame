use crate::account_set::{AccountSet, HasOwnerProgram, WritableAccount};
use crate::anyhow::Result;
use crate::prelude::{StarFrameProgram, SystemProgram};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::cell::{Ref, RefMut};
use std::mem::size_of;

/// An [`AccountSet`] that contains exactly 1 account.
pub trait SingleAccountSet<'info>: AccountSet<'info> {
    /// Gets the contained account.
    fn account_info(&self) -> &AccountInfo<'info>;
    /// Gets the contained account cloned.
    fn account_info_cloned(&self) -> AccountInfo<'info> {
        self.account_info().clone()
    }
    /// Gets the account meta of the contained account.
    fn account_meta(&self) -> AccountMeta {
        let info = self.account_info();
        AccountMeta {
            pubkey: *info.key(),
            is_signer: info.is_signer(),
            is_writable: info.is_writable(),
        }
    }

    /// Gets whether this account signed.
    fn is_signer(&self) -> bool {
        self.account_info().is_signer()
    }

    /// Checks if this account is signed.
    fn check_signer(&self) -> Result<()> {
        if self.is_signer() {
            Ok(())
        } else {
            Err(ProgramError::MissingRequiredSignature.into())
        }
    }

    /// Gets whether this account is writable.
    fn is_writable(&self) -> bool {
        self.account_info().is_writable()
    }

    /// Checks if this account is writable.
    fn check_writable(&self) -> Result<()> {
        if self.is_writable() {
            Ok(())
        } else {
            Err(ProgramError::AccountBorrowFailed.into())
        }
    }

    /// Gets the key of the contained account.
    fn key(&self) -> &'info Pubkey {
        self.account_info().key()
    }
    /// Gets the owner of the contained account.
    fn owner(&self) -> &'info Pubkey {
        self.account_info().owner()
    }

    /// Gets the data of the contained account immutably.
    fn info_data_bytes<'a>(&'a self) -> Result<Ref<'a, [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes()
    }
    /// Gets the data of the contained account mutably.
    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, &'info mut [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes_mut()
    }
}

pub trait CanCloseAccount<'info>: SingleAccountSet<'info> {
    /// Closes the account by zeroing the lamports and leaving the data as the
    /// [`StarFrameProgram::CLOSED_ACCOUNT_DISCRIMINANT`], reallocating down to size.
    fn close(&self, recipient: &impl WritableAccount<'info>) -> Result<()>
    where
        Self: HasOwnerProgram,
    {
        let info = self.account_info();
        info.realloc(
            size_of::<<Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        info.try_borrow_mut_data()?.fill(u8::MAX);
        **recipient.account_info().try_borrow_mut_lamports()? += info.lamports();
        **info.try_borrow_mut_lamports()? = 0;
        Ok(())
    }

    /// Closes the account by reallocating to zero and assigning to the System program.
    /// This is the same as calling `close` but not abusable and harder for indexer detection.
    fn close_full(&self, recipient: &impl WritableAccount<'info>) -> Result<()> {
        let info = self.account_info();
        **recipient.account_info().try_borrow_mut_lamports()? += info.lamports();
        **info.try_borrow_mut_lamports()? = 0;
        info.realloc(0, false)?;
        info.assign(&SystemProgram::PROGRAM_ID);
        Ok(())
    }
}

impl<'info, T> CanCloseAccount<'info> for T where T: SingleAccountSet<'info> {}
