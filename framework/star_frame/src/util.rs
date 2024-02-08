use crate::account_set::{SignedAccount, WritableAccount};
use crate::prelude::*;
use anyhow::anyhow;
use solana_program::system_instruction::transfer;
use std::cell::{Ref, RefMut};
use std::cmp::Ordering;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Devnet,
    Testnet,
    Custom(&'static str),
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "Mainnet"),
            Network::Devnet => write!(f, "Devnet"),
            Network::Testnet => write!(f, "Testnet"),
            Network::Custom(c) => write!(f, "Custom: {c}"),
        }
    }
}

#[cfg(feature = "idl")]
impl From<Network> for star_frame_idl::Network {
    fn from(value: Network) -> Self {
        match value {
            Network::Mainnet => Self::Mainnet,
            Network::Devnet => Self::Devnet,
            Network::Testnet => Self::Testnet,
            Network::Custom(c) => Self::Custom(c.to_string()),
        }
    }
}

/// Similar to [`Ref::map`], but the closure can return an error.
pub fn try_map_ref<'a, I: 'a + ?Sized, O: 'a + ?Sized, E>(
    r: Ref<'a, I>,
    f: impl FnOnce(&I) -> Result<&O, E>,
) -> Result<Ref<'a, O>, E> {
    // Safety: We don't extend the lifetime of the reference beyond what it is.
    unsafe {
        // let value: &'a I = &*(&*r as *const I); // &*:( => &:) Since :( impl deref => :)
        let result = f(&r)? as *const O;
        Ok(Ref::map(r, |_| &*result))
    }
}

/// Similar to [`RefMut::map`], but the closure can return an error.
pub fn try_map_ref_mut<'a, I: 'a + ?Sized, O: 'a + ?Sized, E>(
    mut r: RefMut<'a, I>,
    f: impl FnOnce(&mut I) -> Result<&mut O, E>,
) -> Result<RefMut<'a, O>, E> {
    // Safety: We don't extend the lifetime of the reference beyond what it is.
    unsafe {
        // let value: &'a mut I = &mut *(&mut *r as *mut I);
        let result = f(&mut r)? as *mut O;
        Ok(RefMut::map(r, |_| &mut *result))
    }
}

#[must_use]
pub const fn compare_strings(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut index = 0;
    loop {
        if index >= a_bytes.len() {
            break true;
        }
        if a_bytes[index] != b_bytes[index] {
            break false;
        }
        index += 1;
    }
}

/// Normalizes the rent of an account if data size is changed.
/// Assumes `info` is owned by this program.
pub fn normalize_rent<
    'info,
    T: ?Sized + UnsizedType + ProgramAccount,
    F: WritableAccount<'info> + SignedAccount<'info>,
>(
    info: &DataAccount<'info, T>,
    funder: &F,
    system_program: &Program<'info, SystemProgram>,
    sys_calls: &mut impl SysCallInvoke,
) -> Result<()> {
    let rent = sys_calls.get_rent()?;
    let lamports = info.account_info().lamports();
    let data_len = info.account_info().data_len();
    let rent_lamports = rent.minimum_balance(data_len);
    match rent_lamports.cmp(&lamports) {
        Ordering::Equal => Ok(()),
        Ordering::Greater => {
            let transfer_amount = rent_lamports - lamports;
            if funder.owner() == system_program.key() {
                let transfer_ix = transfer(funder.key(), info.key(), transfer_amount);
                let transfer_accounts = &[info.account_info_cloned(), funder.account_info_cloned()];
                match funder.signer_seeds() {
                    None => sys_calls
                        .invoke(&transfer_ix, transfer_accounts)
                        .map_err(Into::into),
                    Some(seeds) => sys_calls
                        .invoke_signed(&transfer_ix, transfer_accounts, &[&seeds])
                        .map_err(Into::into),
                }
            } else {
                Err(anyhow!(
                    "Funder account `{}` is not owned by the system program, owned by `{}`",
                    funder.key(),
                    funder.owner()
                ))
            }
        }
        Ordering::Less => {
            let transfer_amount = lamports - rent_lamports;
            **info.account_info().lamports.borrow_mut() -= transfer_amount;
            **funder.account_info().lamports.borrow_mut() += transfer_amount;
            Ok(())
        }
    }
}
