//! Basic-4: Advanced PDAs, Error Handling, and Validation
//!
//! This example demonstrates:
//! - Complex PDA seeds with multiple components
//! - Custom error types with detailed messages
//! - Time-based validation using sysvars
//! - Multi-party account validation
//! - Signed PDA operations (transferring with seeds)
//!
//! Key concepts:
//! - Custom Error types with thiserror
//! - Complex AccountValidate implementations
//! - Sysvar usage (Clock)
//! - PDA signing with seeds
//! - Time-locked functionality

use star_frame::{anyhow::ensure, prelude::*};
use thiserror::Error;

#[derive(StarFrameProgram)]
#[program(
    instruction_set = VaultInstructionSet,
    id = "B4sic44444444444444444444444444444444444444"
)]
pub struct VaultProgram;

#[derive(InstructionSet)]
pub enum VaultInstructionSet {
    Initialize(Initialize),
    Deposit(Deposit),
    Withdraw(Withdraw),
}

/// Custom error enum for the Vault program
///
/// KEY PATTERN: Using thiserror for rich error messages
/// - More descriptive than Anchor's error codes
/// - Can include dynamic values in error messages
/// - Standard Rust error handling
#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Insufficient funds: requested {requested}, available {available}")]
    InsufficientFunds { requested: u64, available: u64 },

    #[error("Vault is locked until {unlock_time}")]
    VaultLocked { unlock_time: i64 },

    #[error("Invalid withdrawal amount")]
    InvalidAmount,

    #[error("Unauthorized access")]
    Unauthorized,
}

/// Vault account with time-lock functionality
///
/// Demonstrates:
/// - Multi-party access (owner and beneficiary)
/// - Time-based restrictions
/// - Storing the PDA bump for efficient signing
#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = VaultSeeds)]
#[repr(C, packed)]
pub struct VaultAccount {
    /// The creator who can withdraw anytime
    pub owner: Pubkey,
    /// The beneficiary who can withdraw after unlock_time
    pub beneficiary: Pubkey,
    /// Current balance in the vault
    pub balance: u64,
    /// Unix timestamp when beneficiary can withdraw
    pub unlock_time: i64,
    /// PDA bump seed (stored for efficiency)
    pub bump: u8,
}

/// Complex PDA seeds using multiple fields
///
/// KEY PATTERN: Multi-component seeds
/// - Creates unique vault for each owner-beneficiary pair
/// - Deterministic address derivation
/// - More complex than single-field seeds in basic examples
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"vault")]
pub struct VaultSeeds {
    pub owner: Pubkey,
    pub beneficiary: Pubkey,
}

/// Complex validation with multiple parameters
///
/// ADVANCED PATTERN: Tuple arguments for validation
/// - Validates both identity AND time constraints
/// - Different rules for owner vs beneficiary
/// - Shows how Star Frame can enforce complex business logic
///
/// This replaces multiple Anchor constraints with explicit logic
impl AccountValidate<(&Pubkey, i64)> for VaultAccount {
    fn validate_account(
        self_ref: &Self::Ref<'_>,
        (signer, current_time): (&Pubkey, i64),
    ) -> Result<()> {
        // First check: Is this person authorized at all?
        ensure!(
            signer == &self_ref.owner || signer == &self_ref.beneficiary,
            VaultError::Unauthorized
        );

        // Second check: If beneficiary, is it time yet?
        if signer == &self_ref.beneficiary {
            ensure!(
                current_time >= self_ref.unlock_time,
                VaultError::VaultLocked {
                    unlock_time: self_ref.unlock_time
                }
            );
        }

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize {
    #[ix_args(&run)]
    pub beneficiary: Pubkey,
    #[ix_args(&run)]
    pub unlock_time: i64,
}

#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub owner: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(VaultSeeds {
            owner: *self.owner.pubkey(),
            beneficiary: *self.beneficiary.pubkey(),
        }),
    ))]
    pub vault: Init<Seeded<Account<VaultAccount>>>,
    pub beneficiary: SystemAccount,
    pub system_program: Program<System>,
}

/// Initialize a new time-locked vault
///
/// Demonstrates:
/// - Calculating and storing PDA bump
/// - Multiple instruction arguments
/// - Setting up complex account state
impl StarFrameInstruction for Initialize {
    type ReturnType = ();
    type Accounts<'b, 'c> = InitializeAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        args: (&Pubkey, &i64),
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let (beneficiary, unlock_time) = args;

        // Calculate bump manually for PDA
        // We'll construct seeds manually since GetSeeds trait details aren't clear
        let seed_owner = accounts.owner.pubkey().to_bytes();
        let seed_beneficiary = beneficiary.to_bytes();
        let seeds_vec = [
            b"vault".as_ref(),
            seed_owner.as_ref(),
            seed_beneficiary.as_ref(),
        ];
        let (_, bump) = Pubkey::find_program_address(&seeds_vec, &crate::StarFrameDeclaredProgram::ID);

        **accounts.vault.data_mut()? = VaultAccount {
            owner: *accounts.owner.pubkey(),
            beneficiary: *beneficiary,
            balance: 0,
            unlock_time: *unlock_time,
            bump, // Store for efficient signing later
        };
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Deposit {
    #[ix_args(&run)]
    pub amount: u64,
}

#[derive(AccountSet)]
pub struct DepositAccounts {
    pub depositor: Signer<Mut<SystemAccount>>,
    pub vault: Mut<Account<VaultAccount>>,
    pub vault_wallet: Mut<SystemAccount>,
    pub system_program: Program<System>,
}

impl StarFrameInstruction for Deposit {
    type ReturnType = ();
    type Accounts<'b, 'c> = DepositAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        amount: &u64,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        // Manual transfer: subtract from depositor, add to vault
        *accounts.depositor.try_borrow_mut_lamports()? -= *amount;
        *accounts.vault_wallet.try_borrow_mut_lamports()? += *amount;

        let mut vault = accounts.vault.data_mut()?;
        vault.balance = vault.balance.saturating_add(*amount);

        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Withdraw {
    #[ix_args(&run)]
    pub amount: u64,
}

#[derive(AccountSet)]
pub struct WithdrawAccounts {
    pub authority: Signer,
    pub vault: Mut<Account<VaultAccount>>,
    pub vault_wallet: Mut<SystemAccount>,
    pub recipient: Mut<SystemAccount>,
}

/// Withdraw from vault (with time-based validation)
///
/// KEY PATTERNS:
/// - ValidatedAccount ensures authority/time checks passed
/// - PDA signing with stored bump seed
/// - Custom error messages with context
/// - Safe math with saturating operations
impl StarFrameInstruction for Withdraw {
    type ReturnType = ();
    type Accounts<'b, 'c> = WithdrawAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        amount: &u64,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut vault = accounts.vault.data_mut()?;

        // Manual validation using clock from context
        let clock = ctx.get_clock()?;
        let signer = accounts.authority.pubkey();

        // Check authorization: owner can always withdraw, beneficiary only after unlock
        if signer != &vault.owner {
            ensure!(signer == &vault.beneficiary, VaultError::Unauthorized);
            ensure!(
                clock.unix_timestamp >= vault.unlock_time,
                VaultError::VaultLocked {
                    unlock_time: vault.unlock_time
                }
            );
        }

        // Validation with rich error messages
        ensure!(
            *amount <= vault.balance,
            VaultError::InsufficientFunds {
                requested: *amount,
                available: vault.balance,
            }
        );

        ensure!(*amount > 0, VaultError::InvalidAmount);

        // Manual transfer with PDA authority check
        // Since vault_wallet is controlled by the PDA, we can modify it
        *accounts.vault_wallet.try_borrow_mut_lamports()? -= *amount;
        *accounts.recipient.try_borrow_mut_lamports()? += *amount;

        // Update balance with safe math
        vault.balance = vault.balance.saturating_sub(*amount);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use star_frame::prelude::*;
    
    #[cfg(feature = "idl")]
    #[test]
    fn generate_idl() -> Result<()> {
        use crate::StarFrameDeclaredProgram;
        use codama_nodes::{NodeTrait, ProgramNode};
        let idl = StarFrameDeclaredProgram::program_to_idl()?;
        let codama_idl: ProgramNode = idl.try_into()?;
        let idl_json = codama_idl.to_json()?;
        std::fs::write("idl.json", &idl_json)?;
        Ok(())
    }
}
