use crate::associated_token::AssociatedTokenProgram;
use crate::pod::PodOption;
use crate::token::instructions::{InitializeMint2, InitializeMint2CpiAccounts};
use crate::token::TokenProgram;
use star_frame::account_set::AccountSet;
use star_frame::anyhow::{bail, Context};
use star_frame::bytemuck;
use star_frame::prelude::*;
use std::cell::Ref;

/// A wrapper around `AccountInfo` for the [`spl_token::state::Mint`] account.
/// It validates the account data on validate and provides cheap accessor methods for accessing fields
/// without deserializing the entire account data.
#[derive(AccountSet, Debug, Clone)]
#[validate(extra_validation = self.validate())]
#[validate(
    id = "validate_mint", arg = ValidateMint, generics = [],
    extra_validation = {
        self.validate()?;
        self.validate_mint(&arg)
    }
)]
pub struct MintAccount<'info> {
    #[single_account_set(skip_can_init_account, skip_has_owner_program)]
    info: AccountInfo<'info>,
}

impl HasOwnerProgram for MintAccount<'_> {
    type OwnerProgram = TokenProgram;
}

/// See [`spl_token::state::Mint`].
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default, Zeroable, CheckedBitPattern, Align1)]
pub struct MintData {
    pub mint_authority: PodOption<Pubkey>,
    pub supply: PackedValue<u64>,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: PodOption<Pubkey>,
}

impl<'info> MintAccount<'info> {
    /// See [`spl_token::state::Mint::LEN`].
    /// ```
    /// # use star_frame::solana_program::program_pack::Pack;
    /// # use star_frame_spl::token::{MintAccount, MintData};
    /// assert_eq!(MintAccount::LEN, spl_token::state::Mint::LEN);
    /// assert_eq!(MintAccount::LEN, core::mem::size_of::<MintData>());
    /// ```
    pub const LEN: usize = 82;

    pub fn validate(&self) -> Result<()> {
        let data = self.info.try_borrow_data()?;
        // todo: maybe relax this check to allow token22
        if self.owner() != &TokenProgram::PROGRAM_ID {
            bail!(ProgramError::InvalidAccountOwner);
        }
        if data.len() != Self::LEN {
            bail!(ProgramError::InvalidAccountData);
        }
        // check initialized
        if !self.data()?.is_initialized {
            bail!(ProgramError::UninitializedAccount);
        }
        Ok(())
    }

    pub fn data(&self) -> Result<Ref<MintData>> {
        Ok(Ref::map(self.info.data.try_borrow()?, |data| {
            bytemuck::checked::from_bytes::<MintData>(data)
        }))
    }

    pub fn validate_mint(&self, validate_mint: &ValidateMint) -> Result<()> {
        let data = self.data()?;
        if let Some(decimals) = validate_mint.decimals {
            if data.decimals != decimals {
                bail!(ProgramError::InvalidArgument);
            }
        }
        if let Some(authority) = validate_mint.authority {
            if data.mint_authority != PodOption::some(authority) {
                bail!(ProgramError::InvalidArgument);
            }
        }
        match validate_mint.freeze_authority {
            FreezeAuthority::None => {
                if data.freeze_authority.is_some() {
                    bail!(ProgramError::InvalidArgument);
                }
            }
            FreezeAuthority::Some(authority) => {
                if data.freeze_authority != PodOption::some(authority) {
                    bail!(ProgramError::InvalidArgument);
                }
            }
            _ => {}
        }
        // if let Some(token_program) = validate_mint.token_program {
        //     if self.owner() != &token_program {
        //         bail!(ProgramError::InvalidArgument);
        //     }
        // }
        Ok(())
    }
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
    // pub token_program: Option<Pubkey>,
}
pub type InitMint = InitializeMint2;

impl From<InitMint> for ValidateMint {
    fn from(value: InitMint) -> Self {
        let freeze_authority = match value.freeze_authority {
            None => FreezeAuthority::None,
            Some(authority) => FreezeAuthority::Some(authority),
        };
        Self {
            decimals: Some(value.decimals),
            authority: Some(value.mint_authority),
            freeze_authority,
        }
    }
}

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
        let (init_mint, funder) = arg.0;
        if self.owner() == &SystemProgram::PROGRAM_ID {
            self.init_account(Create((init_mint, funder)), syscalls, account_seeds)?;
        } else {
            self.validate()?;
            self.validate_mint(&init_mint.into())?;
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
            Self::LEN,
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
        .invoke_signed(account_seeds, syscalls)?;
        Ok(())
    }
}

/// A wrapper around `AccountInfo` for the [`spl_token::state::Token`] account.
/// It validates the account data on validate and provides cheap accessor methods for accessing fields
/// without deserializing the entire account data, although it does provide full deserialization methods.
#[derive(AccountSet, Debug, Clone)]
#[validate(extra_validation = self.validate())]
#[validate(
    id = "validate_token", arg = ValidateToken, generics = [],
    extra_validation = {
        self.validate()?;
        self.validate_token(&arg)
    }
)]
#[validate(
    id = "validate_ata", arg = ValidateAta, generics = [],
    extra_validation = {
        self.validate()?;
        self.validate_ata(&arg)
    }
)]
pub struct TokenAccount<'info> {
    #[single_account_set(skip_can_init_account, skip_can_init_seeds, skip_has_owner_program)]
    info: AccountInfo<'info>,
}

impl HasOwnerProgram for TokenAccount<'_> {
    type OwnerProgram = TokenProgram;
}

/// See [`spl_token::state::AccountState`].
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default, Zeroable, CheckedBitPattern, Align1)]
#[repr(u8)]
pub enum AccountState {
    /// Account is not yet initialized
    #[default]
    Uninitialized,
    /// Account is initialized; the account owner and/or delegate may perform permitted operations
    /// on this account
    Initialized,
    /// Account has been frozen by the mint freeze authority. Neither the account owner nor
    /// the delegate are able to perform operations on this account.
    Frozen,
}

/// See [`spl_token::state::Account`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, CheckedBitPattern, Zeroable)]
pub struct TokenAccountData {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: PackedValue<u64>,
    pub delegate: PodOption<Pubkey>,
    pub state: AccountState,
    pub is_native: PodOption<u64>,
    pub delegated_amount: PackedValue<u64>,
    pub close_authority: PodOption<Pubkey>,
}

impl<'info> TokenAccount<'info> {
    /// See [`spl_token::state::Account::LEN`].
    /// ```
    /// # use star_frame::solana_program::program_pack::Pack;
    /// # use star_frame_spl::token::{TokenAccount, TokenAccountData};
    /// assert_eq!(TokenAccount::LEN, spl_token::state::Account::LEN);
    /// assert_eq!(TokenAccount::LEN, core::mem::size_of::<TokenAccountData>());
    /// ```
    pub const LEN: usize = 165;

    pub fn data(&self) -> Result<Ref<TokenAccountData>> {
        Ok(Ref::map(self.info.data.try_borrow()?, |data| {
            bytemuck::checked::from_bytes::<TokenAccountData>(data)
        }))
    }

    pub fn validate(&self) -> Result<()> {
        let data = self.info.try_borrow_data()?;
        // todo: maybe relax this check to allow token22
        if self.owner() != &TokenProgram::PROGRAM_ID {
            bail!(ProgramError::InvalidAccountOwner);
        }
        if data.len() != Self::LEN {
            bail!(ProgramError::InvalidAccountData);
        }
        if self.data()?.state == AccountState::Uninitialized {
            bail!(ProgramError::UninitializedAccount);
        }
        Ok(())
    }

    pub fn validate_token(&self, validate_token: &ValidateToken) -> Result<()> {
        let data = self.data()?;
        if let Some(mint) = validate_token.mint {
            if data.mint != mint {
                bail!(ProgramError::InvalidAccountData);
            }
        }
        if let Some(owner) = validate_token.owner {
            if data.owner != owner {
                bail!(ProgramError::InvalidAccountData);
            }
        }
        Ok(())
    }

    pub fn validate_ata(&self, validate_ata: &ValidateAta) -> Result<()> {
        let expected_address = Pubkey::find_program_address(
            &[
                &validate_ata.owner.to_bytes(),
                &TokenProgram::PROGRAM_ID.to_bytes(),
                &validate_ata.mint.to_bytes(),
            ],
            &AssociatedTokenProgram::PROGRAM_ID,
        )
        .0;
        if self.owner() != &expected_address {
            bail!(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub struct ValidateToken {
    pub mint: Option<Pubkey>,
    pub owner: Option<Pubkey>,
    // pub token_program: Option<Pubkey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ValidateAta {
    pub mint: Pubkey,
    pub owner: Pubkey,
}

impl<'info, A> CanInitSeeds<'info, A> for TokenAccount<'info>
where
    Self: AccountSetValidate<'info, A>,
{
    fn init_seeds(&mut self, _arg: &A, _syscalls: &impl SyscallInvoke<'info>) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use star_frame::solana_program::program_option::COption;
    use star_frame::solana_program::program_pack::Pack;

    #[test]
    fn test_mint_accessors() -> Result<()> {
        let mut lamports = 0;
        let key = Pubkey::new_unique();
        let mint_authority = Pubkey::new_unique();
        let data = spl_token::state::Mint {
            mint_authority: COption::Some(mint_authority),
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
        mint.validate_mint(&ValidateMint {
            decimals: Some(data.decimals),
            authority: Some(mint_authority),
            freeze_authority: FreezeAuthority::Any,
        })?;
        let pod_data = mint.data()?;
        assert_eq!(
            pod_data.mint_authority.into_option(),
            data.mint_authority.into()
        );
        assert_eq!(pod_data.supply, data.supply);
        assert_eq!(pod_data.decimals, data.decimals);
        assert_eq!(pod_data.is_initialized, data.is_initialized);
        assert_eq!(
            pod_data.freeze_authority.into_option(),
            data.freeze_authority.into()
        );
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
            state: spl_token::state::AccountState::Initialized,
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
        account.validate_token(&ValidateToken {
            mint: Some(data.mint),
            owner: Some(data.owner),
        })?;
        let pod_data = account.data()?;
        assert_eq!(pod_data.mint, data.mint);
        assert_eq!(pod_data.owner, data.owner);
        assert_eq!(pod_data.amount, data.amount);
        assert_eq!(pod_data.delegate.into_option(), data.delegate.into());
        assert_eq!(pod_data.state as u8, data.state as u8);
        assert_eq!(pod_data.is_native.into_option(), data.is_native.into());
        assert_eq!(pod_data.delegated_amount, data.delegated_amount);
        Ok(())
    }
}
