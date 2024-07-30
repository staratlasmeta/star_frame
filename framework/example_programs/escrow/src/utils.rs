use star_frame::anyhow::bail;
use star_frame::solana_program::account_info::AccountInfo;
use star_frame::solana_program::program_pack::Pack;
use star_frame::solana_program::pubkey::Pubkey;

pub fn validate_token_mint(
    account_info: &AccountInfo,
    token_program_id: &Pubkey,
) -> star_frame::Result<()> {
    if account_info.owner != token_program_id {
        bail!("Account not owned by token program");
    }
    spl_token::state::Mint::unpack(&account_info.try_borrow_data()?)?;
    Ok(())
}

pub fn validate_token_account(
    account_info: &AccountInfo,
    token_program_id: &Pubkey,
    expected_owner: Option<&Pubkey>,
    minimum_balance: Option<u64>,
    expected_mint: Option<&Pubkey>,
) -> star_frame::Result<()> {
    if account_info.owner != token_program_id {
        bail!("Account not owned by token program");
    }
    let account = spl_token::state::Account::unpack(&account_info.try_borrow_data()?)?;
    if let Some(token_account_owner) = expected_owner {
        if token_account_owner != &account.owner {
            bail!("Unexpected token account owner");
        }
    }
    if let Some(token_account_mint) = expected_mint {
        if token_account_mint != &account.mint {
            bail!("Unexpected token account mint");
        }
    }
    if let Some(min_balance) = minimum_balance {
        if min_balance > account.amount {
            bail!("Insufficient token balance");
        }
    }
    if account.delegate.is_some() {
        bail!("Token delegate not allowed");
    }
    Ok(())
}
