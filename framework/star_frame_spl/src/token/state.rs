use crate::token::instructions::{InitializeMint2, InitializeMint2CpiAccounts};
use crate::token::TokenProgram;
use crate::utils::{unpack_option_key, unpack_option_u64};
use arrayref::array_ref;
use num_enum::TryFromPrimitive;
use spl_token::state::AccountState;
use star_frame::account_set::AccountSet;
use star_frame::anyhow::{bail, Context};
use star_frame::prelude::ProgramError::InvalidAccountOwner;
use star_frame::prelude::*;
use star_frame::solana_program::program_pack::Pack;

/// A wrapper around `AccountInfo` for the [`spl_token::state::Mint`] account.
/// It validates the account data on validate and provides cheap accessor methods for accessing fields
/// without deserializing the entire account data.
#[derive(AccountSet, Debug, Clone)]
#[account_set(skip_default_idl)]
#[validate(extra_validation = self.validate())]
#[validate(
    id = "validate_mint", arg = ValidateMint, generics = [],
    extra_validation = {
        self.validate()?;
        self.validate_mint(&arg)
    }
)]
pub struct MintAccount<'info> {
    #[single_account_set(skip_can_init_account)]
    info: AccountInfo<'info>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FreezeAuthority {
    #[default]
    Any,
    None,
    Some(Pubkey),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub struct ValidateMint {
    pub decimals: Option<u8>,
    pub authority: Option<Pubkey>,
    pub freeze_authority: FreezeAuthority,
    pub token_program: Option<Pubkey>,
}

pub type InitMint = InitializeMint2;

impl<'info> CanInitAccount<'info, CreateIfNeeded<InitMint>> for MintAccount<'info> {
    fn init_account(
        &mut self,
        arg: CreateIfNeeded<InitMint>,
        syscalls: &impl SyscallInvoke<'info>,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()> {
        let funder = syscalls
            .get_funder()
            .context("Missing `funder` for `CreateIfNeeded<InitMint>`")?;
        self.init_account(CreateIfNeeded((arg.0, funder)), syscalls, account_seeds)
    }
}

impl<'info, Funder> CanInitAccount<'info, CreateIfNeeded<(InitMint, &Funder)>>
    for MintAccount<'info>
where
    Funder: SignedAccount<'info> + WritableAccount<'info>,
{
    fn init_account(
        &mut self,
        arg: CreateIfNeeded<(InitMint, &Funder)>,
        syscalls: &impl SyscallInvoke<'info>,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()> {
        if self.owner() == &SystemProgram::PROGRAM_ID {
            self.init_account(Create(arg.0), syscalls, account_seeds)?;
        }
        Ok(())
    }
}

impl<'info> CanInitAccount<'info, Create<InitMint>> for MintAccount<'info> {
    fn init_account(
        &mut self,
        arg: Create<InitMint>,
        syscalls: &impl SyscallInvoke<'info>,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()> {
        let funder = syscalls
            .get_funder()
            .context("Missing `funder` for `Create<InitMint>`")?;
        self.init_account(Create((arg.0, funder)), syscalls, account_seeds)
    }
}

impl<'info, Funder> CanInitAccount<'info, Create<(InitMint, &Funder)>> for MintAccount<'info>
where
    Funder: SignedAccount<'info> + WritableAccount<'info>,
{
    fn init_account(
        &mut self,
        arg: Create<(InitMint, &Funder)>,
        syscalls: &impl SyscallInvoke<'info>,
        account_seeds: Option<Vec<&[u8]>>,
    ) -> Result<()> {
        let (init_mint, funder) = arg.0;
        self.system_create_account(
            funder,
            TokenProgram::PROGRAM_ID,
            spl_token::state::Mint::LEN,
            &account_seeds,
            syscalls,
        )?;
        let account_seeds: &[&[&[u8]]] = match &account_seeds {
            Some(seeds) => &[seeds],
            None => &[],
        };
        TokenProgram::cpi(
            &init_mint,
            InitializeMint2CpiAccounts {
                mint: self.account_info_cloned(),
            },
        )?
        .invoke_signed(syscalls, account_seeds)?;
        Ok(())
    }
}

impl<'info> MintAccount<'info> {
    const MINT_AUTHORITY_OFFSET: usize = 0;
    const SUPPLY_OFFSET: usize = Self::MINT_AUTHORITY_OFFSET + 4 + 32;
    const DECIMALS_OFFSET: usize = Self::SUPPLY_OFFSET + 8;
    const IS_INITIALIZED_OFFSET: usize = Self::DECIMALS_OFFSET + 1;
    const FREEZE_AUTHORITY_OFFSET: usize = Self::IS_INITIALIZED_OFFSET + 1;
    pub fn validate(&self) -> Result<()> {
        let data = self.info_data_bytes_mut()?;
        // todo: maybe relax this check to allow token22
        if self.owner() != &spl_token::ID {
            bail!(InvalidAccountOwner);
        }
        if data.len() != spl_token::state::Mint::LEN {
            bail!(ProgramError::InvalidAccountData);
        }
        // check initialized
        if data[Self::IS_INITIALIZED_OFFSET] != 1 {
            bail!(ProgramError::UninitializedAccount);
        }
        Ok(())
    }

    pub fn validate_mint(&self, validate_mint: &ValidateMint) -> Result<()> {
        if let Some(decimals) = validate_mint.decimals {
            if self.decimals()? != decimals {
                bail!(ProgramError::InvalidArgument);
            }
        }
        if let Some(authority) = validate_mint.authority {
            if self.mint_authority()? != Some(authority) {
                bail!(ProgramError::InvalidArgument);
            }
        }
        match validate_mint.freeze_authority {
            FreezeAuthority::None => {
                if self.freeze_authority()?.is_some() {
                    bail!(ProgramError::InvalidArgument);
                }
            }
            FreezeAuthority::Some(authority) => {
                if self.freeze_authority()? != Some(authority) {
                    bail!(ProgramError::InvalidArgument);
                }
            }
            _ => {}
        }
        if let Some(token_program) = validate_mint.token_program {
            if self.owner() != &token_program {
                bail!(ProgramError::InvalidArgument);
            }
        }
        Ok(())
    }

    pub fn mint_authority(&self) -> Result<Option<Pubkey>> {
        let data = self.info_data_bytes()?;
        let mint_authority_array = array_ref![data, Self::MINT_AUTHORITY_OFFSET, 36];
        Ok(unpack_option_key(mint_authority_array))
    }

    pub fn supply(&self) -> Result<u64> {
        let data = self.info_data_bytes()?;
        let supply_array = array_ref![data, Self::SUPPLY_OFFSET, 8];
        Ok(u64::from_le_bytes(*supply_array))
    }

    pub fn decimals(&self) -> Result<u8> {
        let data = self.info_data_bytes()?;
        Ok(data[Self::DECIMALS_OFFSET])
    }

    pub fn is_initialized(&self) -> Result<bool> {
        let data = self.info_data_bytes()?;
        Ok(data[Self::IS_INITIALIZED_OFFSET] == 1)
    }

    pub fn freeze_authority(&self) -> Result<Option<Pubkey>> {
        let data = self.info_data_bytes()?;
        let freeze_authority_array = array_ref![data, Self::FREEZE_AUTHORITY_OFFSET, 36];
        Ok(unpack_option_key(freeze_authority_array))
    }
}

/// A wrapper around `AccountInfo` for the [`spl_token::state::Token`] account.
/// It validates the account data on validate and provides cheap accessor methods for accessing fields
/// without deserializing the entire account data, although it does provide full deserialization methods.
#[derive(AccountSet, Debug, Clone)]
#[account_set(skip_default_idl)]
#[validate(extra_validation = self.validate())]
pub struct TokenAccount<'info> {
    #[single_account_set(skip_can_init_account, skip_can_init_seeds)]
    info: AccountInfo<'info>,
}

impl<'info, A> CanInitSeeds<'info, A> for TokenAccount<'info>
where
    Self: AccountSetValidate<'info, A>,
{
    fn init_seeds(&mut self, _arg: &A, _syscalls: &impl SyscallInvoke<'info>) -> Result<()> {
        Ok(())
    }
}

impl<'info> TokenAccount<'info> {
    const MINT_OFFSET: usize = 0;
    const OWNER_OFFSET: usize = Self::MINT_OFFSET + 32;
    const AMOUNT_OFFSET: usize = Self::OWNER_OFFSET + 32;
    const DELEGATE_OFFSET: usize = Self::AMOUNT_OFFSET + 8;
    const STATE_OFFSET: usize = Self::DELEGATE_OFFSET + 4 + 32;
    const IS_NATIVE_OFFSET: usize = Self::STATE_OFFSET + 1;
    const DELEGATE_AMOUNT_OFFSET: usize = Self::IS_NATIVE_OFFSET + 4 + 8;
    const CLOSE_AUTHORITY_OFFSET: usize = Self::DELEGATE_AMOUNT_OFFSET + 8;
    pub fn validate(&self) -> Result<()> {
        let data = self.info_data_bytes_mut()?;
        if self.owner() != &spl_token::ID {
            bail!(InvalidAccountOwner);
        }
        if data.len() != spl_token::state::Account::LEN {
            bail!(ProgramError::InvalidAccountData);
        }
        // check initialized
        if data[Self::STATE_OFFSET] == AccountState::Uninitialized as u8 {
            bail!(ProgramError::UninitializedAccount);
        }
        Ok(())
    }
    pub fn mint(&self) -> Result<Pubkey> {
        let data = self.info_data_bytes()?;
        let mint_array = array_ref![data, Self::MINT_OFFSET, 32];
        Ok(Pubkey::new_from_array(*mint_array))
    }

    pub fn token_owner(&self) -> Result<Pubkey> {
        let data = self.info_data_bytes()?;
        let owner_array = array_ref![data, Self::OWNER_OFFSET, 32];
        Ok(Pubkey::new_from_array(*owner_array))
    }

    pub fn amount(&self) -> Result<u64> {
        let data = self.info_data_bytes()?;
        let amount_array = array_ref![data, Self::AMOUNT_OFFSET, 8];
        Ok(u64::from_le_bytes(*amount_array))
    }

    pub fn delegate(&self) -> Result<Option<Pubkey>> {
        let data = self.info_data_bytes()?;
        let delegate_array = array_ref![data, Self::DELEGATE_OFFSET, 36];
        Ok(unpack_option_key(delegate_array))
    }

    pub fn state(&self) -> Result<AccountState> {
        let data = self.info_data_bytes()?;
        Ok(AccountState::try_from_primitive(data[Self::STATE_OFFSET])
            .or(Err(ProgramError::InvalidAccountData))?)
    }

    pub fn is_native(&self) -> Result<Option<u64>> {
        let data = self.info_data_bytes()?;
        let is_native_array = array_ref![data, Self::IS_NATIVE_OFFSET, 12];
        Ok(unpack_option_u64(is_native_array))
    }

    pub fn delegate_amount(&self) -> Result<u64> {
        let data = self.info_data_bytes()?;
        let delegate_amount_array = array_ref![data, Self::DELEGATE_AMOUNT_OFFSET, 8];
        Ok(u64::from_le_bytes(*delegate_amount_array))
    }

    pub fn close_authority(&self) -> Result<Option<Pubkey>> {
        let data = self.info_data_bytes()?;
        let close_authority_array = array_ref![data, Self::CLOSE_AUTHORITY_OFFSET, 36];
        Ok(unpack_option_key(close_authority_array))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenProgram;
    use pretty_assertions::assert_eq;
    use spl_token::solana_program::program_option::COption;

    #[test]
    fn test_mint_accessors() -> Result<()> {
        let mut lamports = 0;
        let key = Pubkey::new_unique();
        let data = spl_token::state::Mint {
            mint_authority: COption::Some(Pubkey::new_unique()),
            supply: 42069,
            decimals: 5,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mut mint_data = vec![0u8; spl_token::state::Mint::LEN];
        data.pack_into_slice(&mut mint_data);
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut mint_data,
            &TokenProgram::PROGRAM_ID,
            false,
            0,
        );
        let mint = MintAccount { info };
        mint.validate()?;
        assert_eq!(mint.mint_authority()?, data.mint_authority.into());
        assert_eq!(mint.supply()?, data.supply);
        assert_eq!(mint.decimals()?, data.decimals);
        assert_eq!(mint.is_initialized()?, data.is_initialized);
        assert_eq!(mint.freeze_authority()?, data.freeze_authority.into());
        Ok(())
    }

    #[test]
    fn test_account_accessors() -> Result<()> {
        let mut lamports = 0;
        let key = Pubkey::new_unique();
        let data = spl_token::state::Account {
            mint: Pubkey::new_unique(),
            owner: Pubkey::new_unique(),
            amount: 69,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::Some(100),
            delegated_amount: 42,
            close_authority: COption::Some(Pubkey::new_unique()),
        };
        let mut account_data = vec![0u8; spl_token::state::Account::LEN];
        data.pack_into_slice(&mut account_data);
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut account_data,
            &TokenProgram::PROGRAM_ID,
            false,
            0,
        );
        let account = TokenAccount { info };
        account.validate()?;
        assert_eq!(account.mint()?, data.mint);
        assert_eq!(account.token_owner()?, data.owner);
        assert_eq!(account.amount()?, data.amount);
        assert_eq!(account.delegate()?, data.delegate.into());
        assert_eq!(account.state()?, data.state);
        assert_eq!(account.is_native()?, data.is_native.into());
        assert_eq!(account.delegate_amount()?, data.delegated_amount);
        assert_eq!(account.close_authority()?, data.close_authority.into());
        Ok(())
    }
}
