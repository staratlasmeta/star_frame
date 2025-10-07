//! Star Frame Program implementation for Solana's system program. Provides type-safe wrappers for most system program instructions.
//!
//! Currently missing the `with_seed` variants of instructions as they are not directly compatible with borsh.

use crate::{empty_star_frame_instruction, prelude::*};
#[allow(deprecated)]
use pinocchio::sysvars::rent::Rent;

/// Solana's system program.
#[derive(Debug, Copy, Clone, Align1, PartialEq, Eq, Ord, PartialOrd)]
pub struct System;
impl StarFrameProgram for System {
    type InstructionSet = SystemInstructionSet;
    type AccountDiscriminant = ();
    /// The system program ID is all zeroes.
    ///
    /// ```
    /// use solana_system_interface::program::ID;
    /// use star_frame::prelude::*;
    /// assert_eq!(solana_system_interface::program::ID, System::ID);
    /// ```
    const ID: Address = Address::new_from_array([0; 32]);
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
impl ProgramToIdl for System {
    type Errors = ();
    fn crate_metadata() -> star_frame_idl::CrateMetadata {
        star_frame_idl::CrateMetadata {
            version: star_frame_idl::Version::new(1, 18, 10),
            name: "system_program".to_string(),
            ..Default::default()
        }
    }
}

// we miss some instructions due to being incompatible with borsh. Probably need to update our IDL spec
// to add a string length width, and then can fix w/ manual borsh impls.
// Bincode prefixes with u64 and borsh with u32 :(
#[derive(Copy, Debug, Clone, PartialEq, Eq, InstructionSet)]
#[ix_set(use_repr)]
#[repr(u32)]
pub enum SystemInstructionSet {
    CreateAccount(CreateAccount),
    Assign(Assign),
    Transfer(Transfer),
    AdvanceNonceAccount(AdvanceNonceAccount) = 4,
    WithdrawNonceAccount(WithdrawNonceAccount),
    InitializeNonceAccount(InitializeNonceAccount),
    AuthorizeNonceAccount(AuthorizeNonceAccount),
    Allocate(Allocate),
    UpgradeNonceAccount(UpgradeNonceAccount) = 12,
}

// CreateAccount
/// See [`solana_system_interface::instruction::SystemInstruction::CreateAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize, InstructionArgs)]
#[type_to_idl(program = System)]
pub struct CreateAccount {
    pub lamports: u64,
    pub space: u64,
    pub owner: Address,
}
/// Accounts for the [`CreateAccount`] instruction.
#[derive(Debug, Copy, Clone, AccountSet)]
pub struct CreateAccountAccounts {
    pub funder: Mut<Signer>,
    pub new_account: Mut<Signer>,
}
empty_star_frame_instruction!(CreateAccount, CreateAccountAccounts);

// Assign
/// See [`solana_system_interface::instruction::SystemInstruction::Assign`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize, InstructionArgs)]
#[type_to_idl(program = System)]
pub struct Assign {
    pub owner: Address,
}
/// Accounts for the [`Assign`] instruction.
#[derive(Debug, Copy, Clone, AccountSet)]
pub struct AssignAccounts {
    pub account: Mut<Signer>,
}
empty_star_frame_instruction!(Assign, AssignAccounts);

// Transfer
/// See [`solana_system_interface::instruction::SystemInstruction::Transfer`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[type_to_idl(program = System)]
pub struct Transfer {
    pub lamports: u64,
}
/// Accounts for the [`Transfer`] instruction.
#[derive(Debug, Copy, Clone, AccountSet)]
pub struct TransferAccounts {
    pub funder: Mut<Signer>,
    pub recipient: Mut<AccountView>,
}
empty_star_frame_instruction!(Transfer, TransferAccounts);

// AdvanceNonceAccount
/// See [`solana_system_interface::instruction::SystemInstruction::AdvanceNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[type_to_idl(program = System)]
pub struct AdvanceNonceAccount;
/// Accounts for the [`AdvanceNonceAccount`] instruction.
#[allow(deprecated)]
mod advance_nonce {
    use super::*;
    #[derive(Debug, Copy, Clone, AccountSet)]
    pub struct AdvanceNonceAccountAccounts {
        pub nonce_account: Mut<AccountView>,
        #[idl(address = crate::account_set::sysvar::RECENT_BLOCKHASHES_ID)]
        pub recent_blockhashes: AccountView,
        pub nonce_authority: Signer,
    }
}
pub use advance_nonce::*;
empty_star_frame_instruction!(AdvanceNonceAccount, AdvanceNonceAccountAccounts);

// WithdrawNonceAccount
/// See [`solana_system_interface::instruction::SystemInstruction::WithdrawNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[type_to_idl(program = System)]
pub struct WithdrawNonceAccount(pub u64);
#[allow(deprecated)]
mod withdraw_nonce {
    use super::*;
    /// Accounts for the [`WithdrawNonceAccount`] instruction.
    #[derive(Debug, Copy, Clone, AccountSet)]
    pub struct WithdrawNonceAccountAccounts {
        pub nonce_account: Mut<AccountView>,
        pub recipient: Mut<AccountView>,
        #[idl(address = crate::account_set::sysvar::RECENT_BLOCKHASHES_ID)]
        pub recent_blockhashes: AccountView,
        pub rent: Sysvar<Rent>,
        pub nonce_authority: Signer,
    }
}
pub use withdraw_nonce::*;
empty_star_frame_instruction!(WithdrawNonceAccount, WithdrawNonceAccountAccounts);

// InitializeNonceAccount
/// See [`solana_system_interface::instruction::SystemInstruction::InitializeNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[type_to_idl(program = System)]
pub struct InitializeNonceAccount(pub Address);
#[allow(deprecated)]
mod initialize_nonce {
    use super::*;
    /// Accounts for the [`InitializeNonceAccount`] instruction.
    #[derive(Debug, Copy, Clone, AccountSet)]
    pub struct InitializeNonceAccountAccounts {
        pub nonce_account: Mut<AccountView>,
        #[idl(address = crate::account_set::sysvar::RECENT_BLOCKHASHES_ID)]
        pub recent_blockhashes: AccountView,
        pub rent: Sysvar<Rent>,
    }
}
pub use initialize_nonce::*;
empty_star_frame_instruction!(InitializeNonceAccount, InitializeNonceAccountAccounts);

// AuthorizeNonceAccount
/// See [`solana_system_interface::instruction::SystemInstruction::AuthorizeNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[type_to_idl(program = System)]
pub struct AuthorizeNonceAccount(pub Address);
/// Accounts for the [`AuthorizeNonceAccount`] instruction.
#[derive(Debug, Copy, Clone, AccountSet)]
pub struct AuthorizeNonceAccountAccounts {
    pub nonce_account: Mut<AccountView>,
    pub nonce_authority: Signer,
}
empty_star_frame_instruction!(AuthorizeNonceAccount, AuthorizeNonceAccountAccounts);

// Allocate
/// See [`solana_system_interface::instruction::SystemInstruction::Allocate`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[type_to_idl(program = System)]
pub struct Allocate {
    pub space: u64,
}
/// Accounts for the [`Allocate`] instruction.
#[derive(Debug, Copy, Clone, AccountSet)]
pub struct AllocateAccounts {
    pub account: Mut<Signer>,
}
empty_star_frame_instruction!(Allocate, AllocateAccounts);

// UpgradeNonceAccount
/// See [`solana_system_interface::instruction::SystemInstruction::UpgradeNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[type_to_idl(program = System)]
pub struct UpgradeNonceAccount;
/// Accounts for the [`UpgradeNonceAccount`] instruction.
#[derive(Debug, Copy, Clone, AccountSet)]
pub struct UpgradeNonceAccountAccounts {
    pub nonce_account: Mut<AccountView>,
}
empty_star_frame_instruction!(UpgradeNonceAccount, UpgradeNonceAccountAccounts);

#[cfg(test)]
mod tests {
    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    use super::*;

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    #[test]
    fn check_idl() {
        use star_frame_idl::{item_source, ty::IdlTypeDef};

        let idl = System::program_to_idl().unwrap();
        let ix_set_source = item_source::<CreateAccountAccounts>();
        let ix_source = item_source::<CreateAccount>();
        assert!(idl.instructions.contains_key(&ix_source));
        assert!(idl.account_sets.contains_key(&ix_set_source));
        assert!(idl.types.contains_key(&ix_source));
        let create_account_data = idl.types.get(&ix_source).unwrap();
        assert!(matches!(
            create_account_data.type_def,
            IdlTypeDef::Struct(_)
        ));
    }

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    #[test]
    fn print_idl() {
        let idl = System::program_to_idl().unwrap();
        std::println!("{}", serde_json::to_string_pretty(&idl).unwrap());
    }

    // TODO: add tests for all the ix builders to ensure they match the solana_system_interface::instruction ix methods
}
