use crate::account_set::{AccountSet, HasOwnerProgram, SignedAccount, WritableAccount};
use crate::anyhow::Result;
use crate::client::{ClientAccountSet, MakeCpi};
use crate::prelude::{StarFrameProgram, SyscallInvoke, System};
use crate::program::system_program;
use crate::syscalls::SyscallCore;
use anyhow::{anyhow, bail};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use star_frame::client::CpiAccountSet;
use std::cell::{Ref, RefMut};
use std::cmp::Ordering;
use std::mem::size_of;

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

/// An [`AccountSet`] that contains exactly 1 account.
pub trait SingleAccountSet<'info>: AccountSet<'info> {
    const META: SingleSetMeta;
    /// Gets the contained account.
    fn account_info(&self) -> &AccountInfo<'info>;
    /// Gets the contained account cloned.
    #[inline]
    fn account_info_cloned(&self) -> AccountInfo<'info> {
        self.account_info().clone()
    }
    /// Gets the account meta of the contained account.
    #[inline]
    fn account_meta(&self) -> AccountMeta {
        let info = self.account_info();
        AccountMeta {
            pubkey: *info.key(),
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
            Err(ProgramError::MissingRequiredSignature.into())
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
            Err(ProgramError::AccountBorrowFailed.into())
        }
    }

    /// Gets the key of the contained account.
    #[inline]
    fn key(&self) -> &'info Pubkey {
        self.account_info().key()
    }

    /// Checks if the key matches the expected key.
    #[inline]
    fn check_key(&self, expected: &Pubkey) -> Result<()> {
        if self.key() == expected {
            Ok(())
        } else {
            Err(ProgramError::InvalidAccountData.into())
        }
    }

    /// Gets the owner of the contained account.
    #[inline]
    fn owner(&self) -> &'info Pubkey {
        self.account_info().owner()
    }

    /// Gets the data of the contained account immutably.
    #[inline]
    fn info_data_bytes<'a>(&'a self) -> Result<Ref<'a, [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes()
    }
    /// Gets the data of the contained account mutably.
    #[inline]
    fn info_data_bytes_mut<'a>(&'a self) -> Result<RefMut<'a, &'info mut [u8]>>
    where
        'info: 'a,
    {
        self.account_info().info_data_bytes_mut()
    }
}

impl<'info, T> CpiAccountSet<'info> for T
where
    T: SingleAccountSet<'info>,
{
    type CpiAccounts<'a> = AccountInfo<'info>;
    const MIN_LEN: usize = 1;
    #[inline]
    fn to_cpi_accounts(&self) -> Self::CpiAccounts<'info> {
        self.account_info_cloned()
    }
    #[inline]
    fn extend_account_infos(
        account_info: Self::CpiAccounts<'info>,
        infos: &mut Vec<AccountInfo<'info>>,
    ) {
        infos.push(account_info);
    }
    #[inline]
    fn extend_account_metas(
        _program_id: &Pubkey,
        account_info: &Self::CpiAccounts<'info>,
        metas: &mut Vec<AccountMeta>,
    ) {
        metas.push(AccountMeta {
            pubkey: *account_info.key,
            is_signer: T::META.signer,
            is_writable: T::META.writable,
        });
    }
}

impl<'info, T> ClientAccountSet for T
where
    T: SingleAccountSet<'info>,
{
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
            is_signer: T::META.signer,
            is_writable: T::META.writable,
        });
    }
}

pub trait CanCloseAccount<'info>: SingleAccountSet<'info> {
    /// Closes the account by zeroing the lamports and replacing the discriminant with all `u8::MAX`,
    /// reallocating down to size.
    fn close(&self, recipient: &impl WritableAccount<'info>) -> Result<()>
    where
        Self: HasOwnerProgram,
    {
        let info = self.account_info();
        info.realloc(
            size_of::<<Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant>(),
            false,
        )?;
        self.info_data_bytes_mut()?.fill(u8::MAX);
        **recipient.account_info().try_borrow_mut_lamports()? += **info.try_borrow_lamports()?;
        **info.try_borrow_mut_lamports()? = 0;
        Ok(())
    }

    /// Closes the account by reallocating to zero and assigning to the System program.
    /// This is the same as calling `close` but not abusable and harder for indexer detection.
    fn close_full(&self, recipient: &impl WritableAccount<'info>) -> Result<()> {
        let info = self.account_info();
        **recipient.account_info().try_borrow_mut_lamports()? += **info.try_borrow_lamports()?;
        **info.try_borrow_mut_lamports()? = 0;
        info.realloc(0, false)?;
        info.assign(&System::ID);
        Ok(())
    }
}

impl<'info, T> CanCloseAccount<'info> for T where T: SingleAccountSet<'info> {}

pub trait CanModifyRent<'info>: SingleAccountSet<'info> {
    /// Normalizes the rent of an account if data size is changed.
    /// Assumes `Self` is owned by this program and funder is a System account
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn normalize_rent<F: WritableAccount<'info> + SignedAccount<'info>>(
        &self,
        funder: &F,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let rent = syscalls.get_rent()?;
        let lamports = **self.account_info().try_borrow_lamports()?;
        let data_len = self.account_info().data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                if lamports == 0 {
                    return Ok(());
                }
                let transfer_amount = rent_lamports - lamports;
                let cpi = System::cpi(
                    &system_program::Transfer {
                        lamports: transfer_amount,
                    },
                    system_program::TransferCpiAccounts {
                        funder: funder.account_info_cloned(),
                        recipient: self.account_info_cloned(),
                    },
                )?;
                match funder.signer_seeds() {
                    None => cpi.invoke(syscalls)?,
                    Some(seeds) => cpi.invoke_signed(&[&seeds], syscalls)?,
                };
                Ok(())
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                **self.account_info().try_borrow_mut_lamports()? -= transfer_amount;
                **funder.account_info().try_borrow_mut_lamports()? += transfer_amount;
                Ok(())
            }
        }
    }

    /// Refunds rent to the funder so long as the account has more than the minimum rent.
    /// Assumes `Self` is owned by this program.
    ///
    /// If the account has 0 lamports (i.e., it is set to be closed), this will do nothing.
    fn refund_rent<F: WritableAccount<'info>>(
        &self,
        funder: &F,
        syscalls: &impl SyscallCore,
    ) -> Result<()> {
        let rent = syscalls.get_rent()?;
        let lamports = **self.account_info().try_borrow_lamports()?;
        let data_len = self.account_info().data_len();
        let rent_lamports = rent.minimum_balance(data_len);
        match rent_lamports.cmp(&lamports) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => {
                if lamports > 0 {
                    Err(anyhow!(
                        "Funder must be Signer to increase rent on {}",
                        self.key()
                    ))
                } else {
                    Ok(())
                }
            }
            Ordering::Less => {
                let transfer_amount = lamports - rent_lamports;
                **self.account_info().try_borrow_mut_lamports()? -= transfer_amount;
                **funder.account_info().try_borrow_mut_lamports()? += transfer_amount;
                Ok(())
            }
        }
    }

    /// Emits a warning message if the account has more lamports than required by rent.
    #[cfg_attr(not(feature = "cleanup_rent_warning"), allow(unused_variables))]
    fn check_cleanup(&self, sys_calls: &impl SyscallCore) -> Result<()> {
        #[cfg(feature = "cleanup_rent_warning")]
        {
            use std::cmp::Ordering;
            if self.is_writable() {
                let rent = sys_calls.get_rent()?;
                let lamports = self.account_info().lamports();
                let data_len = self.account_info().data_len();
                let rent_lamports = rent.minimum_balance(data_len);
                if rent_lamports.cmp(&lamports) == Ordering::Less {
                    solana_program::msg!(
                        "{} was left with more lamports than required by rent",
                        self.key()
                    );
                }
            }
        }
        Ok(())
    }
}

impl<'info, T> CanModifyRent<'info> for T where T: SingleAccountSet<'info> {}

pub trait CanSystemCreateAccount<'info>: SingleAccountSet<'info> {
    /// Creates an account using the system program
    /// Assumes `Self` is owned by the System program and funder is a System account
    fn system_create_account<F: WritableAccount<'info> + SignedAccount<'info>>(
        &self,
        funder: &F,
        owner: Pubkey,
        space: usize,
        account_seeds: &Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if self.owner() != &System::ID {
            bail!(ProgramError::InvalidAccountOwner);
        }
        let current_lamports = **self.account_info().try_borrow_lamports()?;
        let exempt_lamports = syscalls.get_rent()?.minimum_balance(space);

        if current_lamports == 0 {
            let funder_seeds = funder.signer_seeds();
            let seeds: &[&[&[u8]]] = match (&funder_seeds, account_seeds) {
                (Some(signer_seeds), Some(account_seeds)) => &[signer_seeds, account_seeds],
                (Some(signer_seeds), None) => &[signer_seeds],
                (None, Some(account_seeds)) => &[account_seeds],
                (None, None) => &[],
            };
            System::cpi(
                &system_program::CreateAccount {
                    lamports: exempt_lamports,
                    space: space as u64,
                    owner,
                },
                system_program::CreateAccountCpiAccounts {
                    funder: funder.account_info_cloned(),
                    new_account: self.account_info_cloned(),
                },
            )?
            .invoke_signed(seeds, syscalls)?;
        } else {
            let required_lamports = exempt_lamports.saturating_sub(current_lamports).max(1);
            if required_lamports > 0 {
                let cpi = System::cpi(
                    &system_program::Transfer {
                        lamports: required_lamports,
                    },
                    system_program::TransferCpiAccounts {
                        funder: funder.account_info_cloned(),
                        recipient: self.account_info_cloned(),
                    },
                )?;
                match funder.signer_seeds() {
                    None => cpi.invoke(syscalls)?,
                    Some(seeds) => cpi.invoke_signed(&[&seeds], syscalls)?,
                }
            }
            let account_seeds: &[&[&[u8]]] = match &account_seeds {
                Some(seeds) => &[seeds],
                None => &[],
            };
            System::cpi(
                &system_program::Allocate {
                    space: space as u64,
                },
                system_program::AllocateCpiAccounts {
                    account: self.account_info_cloned(),
                },
            )?
            .invoke_signed(account_seeds, syscalls)?;
            System::cpi(
                &system_program::Assign { owner },
                system_program::AssignCpiAccounts {
                    account: self.account_info_cloned(),
                },
            )?
            .invoke_signed(account_seeds, syscalls)?;
        }
        Ok(())
    }
}

impl<'info, T> CanSystemCreateAccount<'info> for T where T: SingleAccountSet<'info> {}
