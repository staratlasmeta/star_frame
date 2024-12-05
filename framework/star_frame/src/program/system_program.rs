use crate::empty_star_frame_instruction;
use crate::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::rent::Rent;
use solana_program::system_program;
#[allow(deprecated)]
use solana_program::sysvar::recent_blockhashes::RecentBlockhashes;

/// Solana's system program.
#[derive(Debug, Copy, Clone, Align1, PartialEq, Eq, Ord, PartialOrd)]
pub struct System;
impl StarFrameProgram for System {
    type InstructionSet = SystemInstructionSet;
    type AccountDiscriminant = ();
    const ID: Pubkey = system_program::ID;
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
impl ProgramToIdl for System {
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
/// See [`solana_program::system_instruction::SystemInstruction::CreateAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct CreateAccount {
    pub lamports: u64,
    pub space: u64,
    pub owner: Pubkey,
}
/// Accounts for the [`CreateAccount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct CreateAccountAccounts<'info> {
    pub funder: Mut<Signer<AccountInfo<'info>>>,
    pub new_account: Mut<Signer<AccountInfo<'info>>>,
}
empty_star_frame_instruction!(CreateAccount, CreateAccountAccounts);

// Assign
/// See [`solana_program::system_instruction::SystemInstruction::Assign`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct Assign {
    pub owner: Pubkey,
}
/// Accounts for the [`Assign`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct AssignAccounts<'info> {
    pub account: Mut<Signer<AccountInfo<'info>>>,
}
empty_star_frame_instruction!(Assign, AssignAccounts);

// Transfer
/// See [`solana_program::system_instruction::SystemInstruction::Transfer`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct Transfer {
    pub lamports: u64,
}
/// Accounts for the [`Transfer`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct TransferAccounts<'info> {
    pub funder: Mut<Signer<AccountInfo<'info>>>,
    pub recipient: Mut<AccountInfo<'info>>,
}
empty_star_frame_instruction!(Transfer, TransferAccounts);

// AdvanceNonceAccount
/// See [`solana_program::system_instruction::SystemInstruction::AdvanceNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct AdvanceNonceAccount;
/// Accounts for the [`AdvanceNonceAccount`] instruction.
#[allow(deprecated)]
mod advance_nonce {
    use super::*;
    #[derive(Debug, Clone, AccountSet)]
    pub struct AdvanceNonceAccountAccounts<'info> {
        pub nonce_account: Mut<AccountInfo<'info>>,
        pub recent_blockhashes: Sysvar<'info, RecentBlockhashes>,
        pub nonce_authority: Signer<AccountInfo<'info>>,
    }
}
pub use advance_nonce::*;
empty_star_frame_instruction!(AdvanceNonceAccount, AdvanceNonceAccountAccounts);

// WithdrawNonceAccount
/// See [`solana_program::system_instruction::SystemInstruction::WithdrawNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct WithdrawNonceAccount(pub u64);
#[allow(deprecated)]
mod withdraw_nonce {
    use super::*;
    /// Accounts for the [`WithdrawNonceAccount`] instruction.
    #[derive(Debug, Clone, AccountSet)]
    pub struct WithdrawNonceAccountAccounts<'info> {
        pub nonce_account: Mut<AccountInfo<'info>>,
        pub recipient: Mut<AccountInfo<'info>>,
        pub recent_blockhashes: Sysvar<'info, RecentBlockhashes>,
        pub rent: Sysvar<'info, Rent>,
        pub nonce_authority: Signer<AccountInfo<'info>>,
    }
}
pub use withdraw_nonce::*;
empty_star_frame_instruction!(WithdrawNonceAccount, WithdrawNonceAccountAccounts);

// InitializeNonceAccount
/// See [`solana_program::system_instruction::SystemInstruction::InitializeNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct InitializeNonceAccount(pub Pubkey);
#[allow(deprecated)]
mod initialize_nonce {
    use super::*;
    /// Accounts for the [`InitializeNonceAccount`] instruction.
    #[derive(Debug, Clone, AccountSet)]
    pub struct InitializeNonceAccountAccounts<'info> {
        pub nonce_account: Mut<AccountInfo<'info>>,
        pub recent_blockhashes: Sysvar<'info, RecentBlockhashes>,
        pub rent: Sysvar<'info, Rent>,
    }
}
pub use initialize_nonce::*;
empty_star_frame_instruction!(InitializeNonceAccount, InitializeNonceAccountAccounts);

// AuthorizeNonceAccount
/// See [`solana_program::system_instruction::SystemInstruction::AuthorizeNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct AuthorizeNonceAccount(pub Pubkey);
/// Accounts for the [`AuthorizeNonceAccount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct AuthorizeNonceAccountAccounts<'info> {
    pub nonce_account: Mut<AccountInfo<'info>>,
    pub nonce_authority: Signer<AccountInfo<'info>>,
}
empty_star_frame_instruction!(AuthorizeNonceAccount, AuthorizeNonceAccountAccounts);

// Allocate
/// See [`solana_program::system_instruction::SystemInstruction::Allocate`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct Allocate {
    pub space: u64,
}
/// Accounts for the [`Allocate`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct AllocateAccounts<'info> {
    pub account: Mut<Signer<AccountInfo<'info>>>,
}
empty_star_frame_instruction!(Allocate, AllocateAccounts);

// UpgradeNonceAccount
/// See [`solana_program::system_instruction::SystemInstruction::UpgradeNonceAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = System)]
pub struct UpgradeNonceAccount;
/// Accounts for the [`UpgradeNonceAccount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct UpgradeNonceAccountAccounts<'info> {
    pub nonce_account: Mut<AccountInfo<'info>>,
}
empty_star_frame_instruction!(UpgradeNonceAccount, UpgradeNonceAccountAccounts);

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(feature = "idl", not(target_os = "solana")))]
    #[test]
    fn check_idl() {
        use star_frame_idl::item_source;
        use star_frame_idl::ty::IdlTypeDef;

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
        println!("{}", serde_json::to_string_pretty(&idl).unwrap());
    }

    // TODO: add tests for all the ix builders to ensure they match the solana_program::system_instruction ix methods
}
