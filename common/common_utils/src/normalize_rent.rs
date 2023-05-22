use crate::UtilError;
use anchor_lang::system_program::{transfer, Transfer};
use common_utils::prelude::*;
use std::cmp::Ordering;

/// Normalizes the rent of an account if data size is changed.
/// Assumes `info` is owned by this program.
pub fn normalize_rent<'info>(
    info: AccountInfo<'info>,
    funder: AccountInfo<'info>,
    system_program: &Program<'info, System>,
    rent: Option<Rent>,
) -> Result<()> {
    let rent = match rent {
        Some(rent) => rent,
        None => Rent::get()?,
    };
    let lamports = info.lamports();
    let data_len = info.data_len();
    let rent_lamports = rent.minimum_balance(data_len);
    match rent_lamports.cmp(&lamports) {
        Ordering::Equal => Ok(()),
        Ordering::Greater => {
            let transfer_amount = rent_lamports - lamports;
            if *funder.owner == System::id() {
                let transfer_accounts = Transfer {
                    from: funder,
                    to: info,
                };
                let transfer_ctx =
                    CpiContext::new(system_program.to_account_info(), transfer_accounts);
                transfer(transfer_ctx, transfer_amount).map_err(Into::into)
            } else {
                Err(error!(UtilError::InvalidRentFunder))
            }
        }
        Ordering::Less => {
            let transfer_amount = lamports - rent_lamports;
            **info.lamports.borrow_mut() -= transfer_amount;
            **funder.lamports.borrow_mut() += transfer_amount;
            Ok(())
        }
    }
}
