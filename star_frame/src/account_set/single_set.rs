use crate::{
    account_set::{SignedAccount, WritableAccount},
    anyhow::Result,
    client::MakeCpi,
    prelude::{
        CanAddLamports, CanCloseAccount, CanFundRent, CanModifyRent, CanSystemCreateAccount,
        CheckKey, Context, System,
    },
    program::system,
};
use anyhow::{bail, Context as _};
use pinocchio::account_info::{AccountInfo, Ref, RefMut};
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct SingleSetMeta {
    pub signer: bool,
    pub writable: bool,
}

impl SingleSetMeta {
    #[must_use]
    pub const fn default() -> Self {
        Self {
            signer: false,
            writable: false,
        }
    }
}

/// An Account Set that contains exactly 1 account.
pub trait SingleAccountSet {
    /// Associated metadata for the account set
    fn meta() -> SingleSetMeta
    where
        Self: Sized;

    /// Gets the contained account by reference
    fn account_info(&self) -> &AccountInfo;

    /// Gets the account meta of the contained account.
    #[inline]
    fn account_meta(&self) -> AccountMeta {
        let info = self.account_info();
        AccountMeta {
            pubkey: *info.pubkey(),
            is_signer: info.is_signer(),
            is_writable: info.is_writable(),
        }
    }

    /// Gets whether this account signed.
    #[inline]
    fn is_signer(&self) -> bool {
        self.account_info().is_signer()
    }

    /// Checks if this account is signed.
    #[inline]
    fn check_signer(&self) -> Result<()> {
        if self.is_signer() {
            Ok(())
        } else {
            bail!("Account {} is not signed", self.pubkey())
        }
    }

    /// Gets whether this account is writable.
    #[inline]
    fn is_writable(&self) -> bool {
        self.account_info().is_writable()
    }

    /// Checks if this account is writable.
    #[inline]
    fn check_writable(&self) -> Result<()> {
        if self.is_writable() {
            Ok(())
        } else {
            bail!("Account {} is not writable", self.pubkey())
        }
    }

    /// Gets the key of the contained account.
    #[inline]
    fn pubkey(&self) -> &Pubkey {
        self.account_info().pubkey()
    }

    /// Gets the owner of the contained account.
    #[inline]
    fn owner_pubkey(&self) -> Pubkey {
        bytemuck::cast(self.account_info().owner_key())
    }

    /// Gets the data of the contained account immutably.
    #[inline]
    fn account_data(&self) -> Result<Ref<'_, [u8]>> {
        self.account_info()
            .try_borrow_data()
            .with_context(|| format!("Failed to borrow data for account {}", self.pubkey()))
    }
    /// Gets the data of the contained account mutably.
    #[inline]
    fn account_data_mut(&self) -> Result<RefMut<'_, [u8]>> {
        self.account_info().try_borrow_mut_data().with_context(|| {
            format!(
                "Failed to borrow mutable data for account {}",
                self.pubkey()
            )
        })
    }
}

impl<T> CheckKey for T
where
    T: SingleAccountSet,
{
    #[inline]
    fn check_key(&self, expected: &Pubkey) -> Result<()> {
        if self.pubkey() == expected {
            Ok(())
        } else {
            bail!(
                "Account key {} does not match expected public key {}",
                self.pubkey(),
                expected
            )
        }
    }
}

impl<T> CanCloseAccount for T
where
    T: SingleAccountSet + ?Sized,
{
    fn account_to_close(&self) -> AccountInfo {
        *self.account_info()
    }
}

impl<T> CanAddLamports for T
where
    T: WritableAccount + Debug + ?Sized,
{
    fn account_to_modify(&self) -> AccountInfo {
        *self.account_info()
    }
}

impl<T> CanFundRent for T
where
    T: CanAddLamports + SignedAccount + ?Sized,
{
    fn can_create_account(&self) -> bool {
        true
    }
    fn fund_rent(
        &self,
        recipient: &dyn SingleAccountSet,
        lamports: u64,
        ctx: &Context,
    ) -> Result<()> {
        let cpi = System::cpi(
            &system::Transfer { lamports },
            system::TransferCpiAccounts {
                funder: *self.account_info(),
                recipient: *recipient.account_info(),
            },
            ctx,
        )?;
        match self.signer_seeds() {
            None => cpi.invoke()?,
            Some(seeds) => cpi.invoke_signed(&[&seeds])?,
        }
        Ok(())
    }

    fn signer_seeds(&self) -> Option<Vec<&[u8]>> {
        SignedAccount::signer_seeds(self)
    }
}

impl<T> CanModifyRent for T
where
    T: SingleAccountSet + ?Sized,
{
    fn account_to_modify(&self) -> AccountInfo {
        *self.account_info()
    }
}

impl<T> CanSystemCreateAccount for T
where
    T: SingleAccountSet + ?Sized,
{
    fn account_to_create(&self) -> AccountInfo {
        *self.account_info()
    }
}
