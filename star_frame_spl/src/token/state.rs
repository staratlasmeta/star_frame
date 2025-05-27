use crate::pod::PodOption;
use crate::token::instructions::{
    InitializeAccount3, InitializeAccount3CpiAccounts, InitializeMint2, InitializeMint2CpiAccounts,
};
use crate::token::Token;
use star_frame::account_set::AccountSet;
use star_frame::anyhow::{bail, Context};
use star_frame::bytemuck;
use star_frame::prelude::*;
use star_frame::util::try_map_ref;
use std::cell::Ref;

/// A wrapper around `AccountInfo` for the [`spl_token::state::Mint`] account.
/// It validates the account data on validate and provides cheap accessor methods for accessing fields
/// without deserializing the entire account data.
#[derive(AccountSet, Debug, Clone)]
#[validate(extra_validation = self.validate())]
#[validate(
    id = "validate_mint", arg = ValidateMint<'a>, generics = [<'a>],
    extra_validation = {
        self.validate()?;
        self.validate_mint(arg)
    }
)]
pub struct MintAccount<'info> {
    #[single_account_set(skip_can_init_account, skip_has_owner_program, skip_has_inner_type)]
    info: AccountInfo<'info>,
}

impl HasOwnerProgram for MintAccount<'_> {
    type OwnerProgram = Token;
}

impl HasInnerType for MintAccount<'_> {
    type Inner = MintAccount<'static>;
}

/// See [`spl_token::state::Mint`].
#[derive(Debug, Clone, PartialEq, Eq, Copy, Default, Zeroable, CheckedBitPattern, Align1)]
#[repr(C, packed)]
pub struct MintData {
    pub mint_authority: PodOption<Pubkey>,
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
    pub freeze_authority: PodOption<Pubkey>,
}

impl MintAccount<'_> {
    /// See [`spl_token::state::Mint::LEN`].
    /// ```
    /// # use star_frame::solana_program::program_pack::Pack;
    /// # use star_frame_spl::token::state::{MintAccount, MintData};
    /// assert_eq!(MintAccount::LEN, spl_token::state::Mint::LEN);
    /// assert_eq!(MintAccount::LEN, core::mem::size_of::<MintData>());
    /// ```
    pub const LEN: usize = 82;

    #[inline]
    pub fn validate(&self) -> Result<()> {
        // // todo: maybe relax this check to allow token22
        if self.owner() != &Token::ID {
            bail!(
                "MintAccount owner {} does not match expected Token program ID {}",
                self.owner(),
                Token::ID
            );
        }
        if self.info_data_bytes()?.len() != Self::LEN {
            bail!(
                "MintAccount {} has invalid data length {}, expected {}",
                self.key(),
                self.info_data_bytes()?.len(),
                Self::LEN
            );
        }
        // check initialized
        if !self.data_unchecked()?.is_initialized {
            bail!("MintAccount {} is not initialized", self.key());
        }
        Ok(())
    }

    #[inline]
    pub fn data_unchecked(&self) -> Result<Ref<MintData>> {
        Ok(try_map_ref(self.info_data_bytes()?, |data| {
            bytemuck::checked::try_from_bytes::<MintData>(data)
        })?)
    }

    #[inline]
    pub fn data(&self) -> Result<Ref<MintData>> {
        if self.is_writable() {
            self.validate()?;
        }
        self.data_unchecked()
    }

    #[inline]
    pub fn validate_mint(&self, validate_mint: ValidateMint) -> Result<()> {
        let data = self.data()?;
        if let Some(decimals) = validate_mint.decimals {
            if data.decimals != decimals {
                bail!(
                    "MintAccount {} has decimals {}, expected {}",
                    self.key(),
                    data.decimals,
                    decimals
                );
            }
        }
        if let Some(authority) = validate_mint.authority {
            if data.mint_authority != PodOption::some(*authority) {
                bail!(
                    "MintAccount {} has mint authority {:?}, expected {:?}",
                    self.key(),
                    data.mint_authority,
                    authority
                );
            }
        }
        match validate_mint.freeze_authority {
            FreezeAuthority::None => {
                if data.freeze_authority.is_some() {
                    bail!(
                        "MintAccount {} has a freeze authority but expected none",
                        self.key()
                    );
                }
            }
            FreezeAuthority::Some(authority) => {
                if data.freeze_authority != PodOption::some(*authority) {
                    bail!(
                        "MintAccount {} has freeze authority {:?}, expected {:?}",
                        self.key(),
                        data.freeze_authority,
                        authority
                    );
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
pub enum FreezeAuthority<'a> {
    #[default]
    Any,
    None,
    Some(&'a Pubkey),
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub struct ValidateMint<'a> {
    pub decimals: Option<u8>,
    pub authority: Option<&'a Pubkey>,
    pub freeze_authority: FreezeAuthority<'a>,
    // pub token_program: Option<Pubkey>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InitMint<'a> {
    pub decimals: u8,
    pub mint_authority: &'a Pubkey,
    pub freeze_authority: Option<&'a Pubkey>,
}

impl<'a> From<InitMint<'a>> for ValidateMint<'a> {
    fn from(value: InitMint<'a>) -> Self {
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

impl<'info> CanInitAccount<'info, InitMint<'_>> for MintAccount<'info> {
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: InitMint,
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let funder = syscalls
            .get_funder()
            .context("Missing tagged `funder` for MintAccount `init_account`")?;
        self.init_account::<IF_NEEDED>((arg, funder), account_seeds, syscalls)
    }
}

impl<'info> CanInitAccount<'info, (InitMint<'_>, &dyn CanFundRent<'info>)> for MintAccount<'info> {
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (InitMint, &dyn CanFundRent<'info>),
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let (init_mint, funder) = arg;
        if IF_NEEDED && self.owner() == &Token::ID {
            self.validate()?;
            self.validate_mint(init_mint.into())?;
            return Ok(());
        }
        self.check_writable()?;
        self.system_create_account(funder, Token::ID, Self::LEN, &account_seeds, syscalls)?;
        let account_seeds: &[&[&[u8]]] = match &account_seeds {
            Some(seeds) => &[seeds],
            None => &[],
        };
        Token::cpi(
            &InitializeMint2 {
                decimals: init_mint.decimals,
                mint_authority: *init_mint.mint_authority,
                freeze_authority: init_mint.freeze_authority.cloned(),
            },
            InitializeMint2CpiAccounts {
                mint: self.account_info_cloned(),
            },
        )?
        .invoke_signed(account_seeds, syscalls)?;
        Ok(())
    }
}

/// A wrapper around `AccountInfo` for the [`spl_token::state::Account`] account.
/// It validates the account data on validate and provides cheap accessor methods for accessing fields
/// without deserializing the entire account data, although it does provide full deserialization methods.
#[derive(AccountSet, Debug, Clone)]
#[validate(extra_validation = self.validate())]
#[validate(
    id = "validate_token", arg = ValidateToken<'a>, generics = [<'a>],
    extra_validation = {
        self.validate()?;
        self.validate_token(arg)
    }
)]
pub struct TokenAccount<'info> {
    #[single_account_set(skip_can_init_account, skip_has_owner_program, skip_has_inner_type)]
    info: AccountInfo<'info>,
}

impl HasOwnerProgram for TokenAccount<'_> {
    type OwnerProgram = Token;
}

impl HasInnerType for TokenAccount<'_> {
    type Inner = TokenAccount<'static>;
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
#[derive(Clone, Copy, Debug, Default, PartialEq, CheckedBitPattern, Zeroable)]
#[repr(C, packed)]
pub struct TokenAccountData {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate: PodOption<Pubkey>,
    pub state: AccountState,
    pub is_native: PodOption<u64>,
    pub delegated_amount: u64,
    pub close_authority: PodOption<Pubkey>,
}

#[cfg(doc)]
use star_frame::solana_program::program_pack::Pack;

impl TokenAccount<'_> {
    /// See [`spl_token::state::Account`] LEN.
    /// ```
    /// # use star_frame::solana_program::program_pack::Pack;
    /// # use star_frame_spl::token::state::{TokenAccount, TokenAccountData};
    /// assert_eq!(TokenAccount::LEN, spl_token::state::Account::LEN);
    /// assert_eq!(TokenAccount::LEN, core::mem::size_of::<TokenAccountData>());
    /// ```
    pub const LEN: usize = 165;

    #[inline]
    pub fn validate(&self) -> Result<()> {
        // todo: maybe relax this check to allow token22
        if self.owner() != &Token::ID {
            bail!(
                "TokenAccount owner {} does not match expected Token program ID {}",
                self.owner(),
                Token::ID
            );
        }
        if self.info_data_bytes()?.len() != Self::LEN {
            bail!(
                "TokenAccount {} has invalid data length {}, expected {}",
                self.key(),
                self.info_data_bytes()?.len(),
                Self::LEN
            );
        }
        // set validate before checking state to allow us to call .data()
        if self.data_unchecked()?.state == AccountState::Uninitialized {
            bail!("TokenAccount {} is not initialized", self.key());
        }
        Ok(())
    }

    #[inline]
    pub fn data_unchecked(&self) -> Result<Ref<TokenAccountData>> {
        Ok(try_map_ref(self.info_data_bytes()?, |data| {
            bytemuck::checked::try_from_bytes::<TokenAccountData>(data)
        })?)
    }

    #[inline]
    pub fn data(&self) -> Result<Ref<TokenAccountData>> {
        if self.is_writable() {
            self.validate()?;
        }
        self.data_unchecked()
    }

    #[inline]
    pub fn validate_token(&self, validate_token: ValidateToken) -> Result<()> {
        let data = self.data()?;
        if let Some(mint) = validate_token.mint {
            if data.mint != *mint {
                bail!(
                    "TokenAccount {} has mint {}, expected {}",
                    self.key(),
                    data.mint,
                    mint
                );
            }
        }
        if let Some(owner) = validate_token.owner {
            if data.owner != *owner {
                bail!(
                    "TokenAccount {} has owner {}, expected {}",
                    self.key(),
                    data.owner,
                    owner
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub struct ValidateToken<'a> {
    pub mint: Option<&'a Pubkey>,
    pub owner: Option<&'a Pubkey>,
    // pub token_program: Option<Pubkey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct InitToken<'a, MintInfo> {
    pub owner: &'a Pubkey,
    pub mint: &'a MintInfo,
}

impl<'a, 'info, MintInfo> From<InitToken<'a, MintInfo>> for ValidateToken<'a>
where
    MintInfo: SingleAccountSet<'info>,
    'info: 'a,
{
    fn from(value: InitToken<'a, MintInfo>) -> Self {
        Self {
            mint: Some(value.mint.key()),
            owner: Some(value.owner),
        }
    }
}

impl<'info, MintInfo> CanInitAccount<'info, InitToken<'_, MintInfo>> for TokenAccount<'info>
where
    MintInfo: SingleAccountSet<'info>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: InitToken<MintInfo>,
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        let funder = syscalls
            .get_funder()
            .context("Missing tagged `funder` for TokenAccount `init_account`")?;
        self.init_account::<IF_NEEDED>((arg, funder), account_seeds, syscalls)
    }
}

impl<'info, MintInfo> CanInitAccount<'info, (InitToken<'_, MintInfo>, &dyn CanFundRent<'info>)>
    for TokenAccount<'info>
where
    MintInfo: SingleAccountSet<'info>,
{
    fn init_account<const IF_NEEDED: bool>(
        &mut self,
        arg: (InitToken<MintInfo>, &dyn CanFundRent<'info>),
        account_seeds: Option<Vec<&[u8]>>,
        syscalls: &impl SyscallInvoke<'info>,
    ) -> Result<()> {
        if IF_NEEDED && self.owner() == &Token::ID {
            self.validate()?;
            self.validate_token(arg.0.into())?;
            return Ok(());
        }
        self.check_writable()?;
        let (init_token, funder) = arg;
        self.system_create_account(funder, Token::ID, Self::LEN, &account_seeds, syscalls)?;
        let account_seeds: &[&[&[u8]]] = match &account_seeds {
            Some(seeds) => &[seeds],
            None => &[],
        };
        Token::cpi(
            &InitializeAccount3 {
                owner: *init_token.owner,
            },
            InitializeAccount3CpiAccounts {
                account: self.account_info_cloned(),
                mint: init_token.mint.account_info_cloned(),
            },
        )?
        .invoke_signed(account_seeds, syscalls)?;
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
            &Token::ID,
            false,
            0,
        );
        let mint = MintAccount { info };
        mint.validate()?;
        mint.validate_mint(ValidateMint {
            decimals: Some(data.decimals),
            authority: Some(&mint_authority),
            freeze_authority: FreezeAuthority::Any,
        })?;
        let pod_data = mint.data()?;
        assert_eq!(
            pod_data.mint_authority.into_option(),
            data.mint_authority.into()
        );
        assert_eq!({ pod_data.supply }, data.supply);
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
            &Token::ID,
            false,
            0,
        );
        let account = TokenAccount { info };
        account.validate()?;
        account.validate_token(ValidateToken {
            mint: Some(&data.mint),
            owner: Some(&data.owner),
        })?;
        let pod_data = account.data()?;
        assert_eq!(pod_data.mint, data.mint);
        assert_eq!(pod_data.owner, data.owner);
        assert_eq!({ pod_data.amount }, data.amount);
        assert_eq!(pod_data.delegate.into_option(), data.delegate.into());
        assert_eq!(pod_data.state as u8, data.state as u8);
        assert_eq!(pod_data.is_native.into_option(), data.is_native.into());
        assert_eq!({ pod_data.delegated_amount }, data.delegated_amount);
        Ok(())
    }
}
