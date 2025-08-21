use borsh::{BorshDeserialize, BorshSerialize};
use star_frame::{empty_star_frame_instruction, pinocchio::sysvars::rent::Rent, prelude::*};

#[derive(Copy, Debug, Clone, PartialEq, Eq, InstructionSet)]
#[ix_set(use_repr)]
#[repr(u8)]
pub enum TokenInstructionSet {
    InitializeMint(InitializeMint),
    InitializeAccount(InitializeAccount),
    InitializeMultisig(InitializeMultisig),
    Transfer(Transfer),
    Approve(Approve),
    Revoke(Revoke),
    SetAuthority(SetAuthority),
    MintTo(MintTo),
    Burn(Burn),
    CloseAccount(CloseAccount),
    FreezeAccount(FreezeAccount),
    ThawAccount(ThawAccount),
    TransferChecked(TransferChecked),
    ApproveChecked(ApproveChecked),
    MintToChecked(MintToChecked),
    BurnChecked(BurnChecked),
    InitializeAccount2(InitializeAccount2),
    SyncNative(SyncNative),
    InitializeAccount3(InitializeAccount3),
    InitializeMultisig2(InitializeMultisig2),
    InitializeMint2(InitializeMint2),
    GetAccountDataSize(GetAccountDataSize),
    InitializeImmutableOwner(InitializeImmutableOwner),
    AmountToUiAmount(AmountToUiAmount),
}

/// Specifies the authority type for SetAuthority instructions
/// Copied from [`spl_token::instruction::AuthorityType`] to allow for `TypeToIdl` implementation
#[derive(Clone, Copy, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize, TypeToIdl)]
#[type_to_idl(program = crate::token::Token)]
#[repr(u8)]
pub enum AuthorityType {
    /// Authority to mint new tokens
    MintTokens,
    /// Authority to freeze any account associated with the Mint
    FreezeAccount,
    /// Owner of a given token account
    AccountOwner,
    /// Authority to close a token account
    CloseAccount,
}

// initialize mint
/// See [`spl_token::instruction::TokenInstruction::InitializeMint`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeMint {
    pub decimals: u8,
    pub mint_authority: Pubkey,
    pub freeze_authority: Option<Pubkey>,
}
/// Accounts for the [`InitializeMint`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeMintAccounts {
    pub mint: Mut<AccountInfo>,
    pub rent: Sysvar<Rent>,
}
empty_star_frame_instruction!(InitializeMint, InitializeMintAccounts);

// initialize account
/// See [`spl_token::instruction::TokenInstruction::InitializeAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeAccount;
/// Accounts for the [`InitializeAccount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeAccountAccounts {
    pub account: Mut<AccountInfo>,
    pub mint: AccountInfo,
    pub owner: AccountInfo,
    pub rent: Sysvar<Rent>,
}
empty_star_frame_instruction!(InitializeAccount, InitializeAccountAccounts);

// initialize multisig
/// See [`spl_token::instruction::TokenInstruction::InitializeMultisig`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeMultisig {
    pub m: u8,
}
/// Accounts for the [`InitializeMultisig`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeMultisigAccounts {
    pub multisig: Mut<AccountInfo>,
    pub rent: Sysvar<Rent>,
    pub signers: Rest<AccountInfo>,
}
empty_star_frame_instruction!(InitializeMultisig, InitializeMultisigAccounts);

// transfer
/// See [`spl_token::instruction::TokenInstruction::Transfer`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct Transfer {
    pub amount: u64,
}
// todo: handle multisig with AccountSet enums
/// Accounts for the [`Transfer`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct TransferAccounts {
    pub source: Mut<AccountInfo>,
    pub destination: Mut<AccountInfo>,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Transfer, TransferAccounts);

// approve
/// See [`spl_token::instruction::TokenInstruction::Approve`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct Approve {
    pub amount: u64,
}
// todo: handle multisig with AccountSet enums
/// Accounts for the [`Approve`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct ApproveAccounts {
    pub source: Mut<AccountInfo>,
    pub delegate: AccountInfo,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Approve, ApproveAccounts);

// revoke
/// See [`spl_token::instruction::TokenInstruction::Revoke`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct Revoke;
// todo: handle multisig with AccountSet enums
/// Accounts for the [`Revoke`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct RevokeAccounts {
    pub source: Mut<AccountInfo>,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Revoke, RevokeAccounts);

// set authority
/// See [`spl_token::instruction::TokenInstruction::SetAuthority`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct SetAuthority {
    pub authority_type: AuthorityType,
    pub new_authority: Option<Pubkey>,
}
// todo: handle multisig with AccountSet enums
/// Accounts for the [`SetAuthority`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct SetAuthorityAccounts {
    pub account: Mut<AccountInfo>,
    pub current_authority: Signer<AccountInfo>,
}
empty_star_frame_instruction!(SetAuthority, SetAuthorityAccounts);

// mint to
/// See [`spl_token::instruction::TokenInstruction::MintTo`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct MintTo {
    pub amount: u64,
}
// todo: handle multisig with AccountSet enums
/// Accounts for the [`MintTo`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct MintToAccounts {
    pub mint: Mut<AccountInfo>,
    pub account: Mut<AccountInfo>,
    pub mint_authority: Signer<AccountInfo>,
}
empty_star_frame_instruction!(MintTo, MintToAccounts);

// burn
/// See [`spl_token::instruction::TokenInstruction::Burn`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct Burn {
    pub amount: u64,
}
// todo: handle multisig with AccountSet enums
/// Accounts for the [`Burn`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct BurnAccounts {
    pub account: Mut<AccountInfo>,
    pub mint: Mut<AccountInfo>,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(Burn, BurnAccounts);

// close account
/// See [`spl_token::instruction::TokenInstruction::CloseAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct CloseAccount;
// todo: handle multisig with AccountSet enums
/// Accounts for the [`CloseAccount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct CloseAccountAccounts {
    pub account: Mut<AccountInfo>,
    pub destination: Mut<AccountInfo>,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(CloseAccount, CloseAccountAccounts);

// freeze account
/// See [`spl_token::instruction::TokenInstruction::FreezeAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct FreezeAccount;
// todo: handle multisig with AccountSet enums
/// Accounts for the [`FreezeAccount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct FreezeAccountAccounts {
    pub account: Mut<AccountInfo>,
    pub mint: AccountInfo,
    pub authority: Signer<AccountInfo>,
}
empty_star_frame_instruction!(FreezeAccount, FreezeAccountAccounts);

// thaw account
/// See [`spl_token::instruction::TokenInstruction::ThawAccount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct ThawAccount;
// todo: handle multisig with AccountSet enums
/// Accounts for the [`ThawAccount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct ThawAccountAccounts {
    pub account: Mut<AccountInfo>,
    pub mint: AccountInfo,
    pub authority: Signer<AccountInfo>,
}
empty_star_frame_instruction!(ThawAccount, ThawAccountAccounts);

// transfer checked
/// See [`spl_token::instruction::TokenInstruction::TransferChecked`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct TransferChecked {
    pub amount: u64,
    pub decimals: u8,
}
/// Accounts for the [`TransferChecked`] instruction.
/// todo: Handle multisig with AccountSet enums.
#[derive(Debug, Clone, AccountSet)]
pub struct TransferCheckedAccounts {
    pub source: Mut<AccountInfo>,
    pub mint: AccountInfo,
    pub destination: Mut<AccountInfo>,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(TransferChecked, TransferCheckedAccounts);

// approve checked
/// See [`spl_token::instruction::TokenInstruction::ApproveChecked`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct ApproveChecked {
    pub amount: u64,
    pub decimals: u8,
}
/// Accounts for the [`ApproveChecked`] instruction.
/// todo: Handle multisig with AccountSet enums.
#[derive(Debug, Clone, AccountSet)]
pub struct ApproveCheckedAccounts {
    pub source: Mut<AccountInfo>,
    pub mint: AccountInfo,
    pub delegate: AccountInfo,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(ApproveChecked, ApproveCheckedAccounts);

// mint to checked
/// See [`spl_token::instruction::TokenInstruction::MintToChecked`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct MintToChecked {
    pub amount: u64,
    pub decimals: u8,
}
/// Accounts for the [`MintToChecked`] instruction.
/// todo: Handle multisig with AccountSet enums.
#[derive(Debug, Clone, AccountSet)]
pub struct MintToCheckedAccounts {
    pub mint: Mut<AccountInfo>,
    pub account: Mut<AccountInfo>,
    pub mint_authority: Signer<AccountInfo>,
}
empty_star_frame_instruction!(MintToChecked, MintToCheckedAccounts);

// burn checked
/// See [`spl_token::instruction::TokenInstruction::BurnChecked`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct BurnChecked {
    pub amount: u64,
    pub decimals: u8,
}
/// Accounts for the [`BurnChecked`] instruction.
/// todo: Handle multisig with AccountSet enums.
#[derive(Debug, Clone, AccountSet)]
pub struct BurnCheckedAccounts {
    pub account: Mut<AccountInfo>,
    pub mint: Mut<AccountInfo>,
    pub owner: Signer<AccountInfo>,
}
empty_star_frame_instruction!(BurnChecked, BurnCheckedAccounts);

// initialize account 2
/// See [`spl_token::instruction::TokenInstruction::InitializeAccount2`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeAccount2 {
    pub owner: Pubkey,
}
/// Accounts for the [`InitializeAccount2`] instruction.
/// todo: Consider multisig ownership scenarios if required.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeAccount2Accounts {
    pub account: Mut<AccountInfo>,
    pub mint: AccountInfo,
    pub rent: Sysvar<Rent>,
}
empty_star_frame_instruction!(InitializeAccount2, InitializeAccount2Accounts);

// sync native
/// See [`spl_token::instruction::TokenInstruction::SyncNative`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct SyncNative;
/// Accounts for the [`SyncNative`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct SyncNativeAccounts {
    pub account: Mut<AccountInfo>,
}
empty_star_frame_instruction!(SyncNative, SyncNativeAccounts);

// initialize account 3
/// See [`spl_token::instruction::TokenInstruction::InitializeAccount3`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeAccount3 {
    pub owner: Pubkey,
}
/// Accounts for the [`InitializeAccount3`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeAccount3Accounts {
    pub account: Mut<AccountInfo>,
    pub mint: AccountInfo,
}
empty_star_frame_instruction!(InitializeAccount3, InitializeAccount3Accounts);

// initialize multisig 2
/// See [`spl_token::instruction::TokenInstruction::InitializeMultisig2`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeMultisig2 {
    pub m: u8,
}
/// Accounts for the [`InitializeMultisig2`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeMultisig2Accounts {
    pub multisig: Mut<AccountInfo>,
    pub signers: Rest<AccountInfo>,
}
empty_star_frame_instruction!(InitializeMultisig2, InitializeMultisig2Accounts);

// initialize mint 2
/// See [`spl_token::instruction::TokenInstruction::InitializeMint2`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeMint2 {
    pub decimals: u8,
    pub mint_authority: Pubkey,
    pub freeze_authority: Option<Pubkey>,
}
/// Accounts for the [`InitializeMint2`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeMint2Accounts {
    pub mint: Mut<AccountInfo>,
}
empty_star_frame_instruction!(InitializeMint2, InitializeMint2Accounts);

// get account data size
/// See [`spl_token::instruction::TokenInstruction::GetAccountDataSize`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct GetAccountDataSize;
/// Accounts for the [`GetAccountDataSize`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct GetAccountDataSizeAccounts {
    pub mint: AccountInfo,
}
empty_star_frame_instruction!(GetAccountDataSize, GetAccountDataSizeAccounts);

// initialize immutable owner
/// See [`spl_token::instruction::TokenInstruction::InitializeImmutableOwner`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct InitializeImmutableOwner;
/// Accounts for the [`InitializeImmutableOwner`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct InitializeImmutableOwnerAccounts {
    pub account: Mut<AccountInfo>,
}
empty_star_frame_instruction!(InitializeImmutableOwner, InitializeImmutableOwnerAccounts);

// amount to ui amount
/// See [`spl_token::instruction::TokenInstruction::AmountToUiAmount`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = crate::token::Token)]
pub struct AmountToUiAmount {
    pub amount: u64,
}
/// Accounts for the [`AmountToUiAmount`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct AmountToUiAmountAccounts {
    pub mint: AccountInfo,
}
empty_star_frame_instruction!(AmountToUiAmount, AmountToUiAmountAccounts);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;
    use pretty_assertions::assert_eq;
    use star_frame::itertools::Itertools;

    #[cfg(feature = "idl")]
    #[test]
    fn print_token_idl() -> Result<()> {
        let idl = Token::program_to_idl()?;
        println!("{}", star_frame::serde_json::to_string_pretty(&idl)?);
        Ok(())
    }

    #[test]
    fn test_initialize_mint() -> Result<()> {
        let decimals = 8u8;
        let mint = Pubkey::new_unique();
        let mint_authority = Pubkey::new_unique();
        let freeze_authority = Some(Pubkey::new_unique());

        let initialize_mint_sf = Token::instruction(
            &InitializeMint {
                decimals,
                mint_authority,
                freeze_authority,
            },
            InitializeMintClientAccounts { mint, rent: None },
        )?;

        let initialize_mint_ix = spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint,
            &mint_authority,
            freeze_authority.as_ref(),
            decimals,
        )?;
        assert_eq!(initialize_mint_sf, initialize_mint_ix);
        Ok(())
    }

    #[test]
    fn test_initialize_account() -> Result<()> {
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let initialize_account_sf = Token::instruction(
            &InitializeAccount,
            InitializeAccountClientAccounts {
                account,
                mint,
                owner,
                rent: None,
            },
        )?;

        let initialize_account_ix =
            spl_token::instruction::initialize_account(&spl_token::id(), &account, &mint, &owner)?;
        assert_eq!(initialize_account_sf, initialize_account_ix);
        Ok(())
    }

    #[test]
    fn test_initialize_multisig() -> Result<()> {
        let multisig = Pubkey::new_unique();
        let signers = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let m = 2u8;

        let initialize_multisig_sf = Token::instruction(
            &InitializeMultisig { m },
            InitializeMultisigClientAccounts {
                multisig,
                rent: None,
                signers: signers.clone(),
            },
        )?;
        let signers = signers.iter().collect_vec();

        let initialize_multisig_ix =
            spl_token::instruction::initialize_multisig(&spl_token::id(), &multisig, &signers, m)?;
        assert_eq!(initialize_multisig_sf, initialize_multisig_ix);
        Ok(())
    }

    #[test]
    fn test_transfer() -> Result<()> {
        let source = Pubkey::new_unique();
        let destination = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let amount = 500u64;

        let transfer_sf = Token::instruction(
            &Transfer { amount },
            TransferClientAccounts {
                source,
                destination,
                owner,
            },
        )?;

        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &source,
            &destination,
            &owner,
            &[],
            amount,
        )?;
        assert_eq!(transfer_sf, transfer_ix);
        Ok(())
    }

    #[test]
    fn test_approve() -> Result<()> {
        let source = Pubkey::new_unique();
        let delegate = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let amount = 250u64;

        let approve_sf = Token::instruction(
            &Approve { amount },
            ApproveClientAccounts {
                source,
                delegate,
                owner,
            },
        )?;

        let approve_ix = spl_token::instruction::approve(
            &spl_token::id(),
            &source,
            &delegate,
            &owner,
            &[],
            amount,
        )?;
        assert_eq!(approve_sf, approve_ix);
        Ok(())
    }

    #[test]
    fn test_revoke() -> Result<()> {
        let source = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let revoke_sf = Token::instruction(&Revoke, RevokeClientAccounts { source, owner })?;

        let revoke_ix = spl_token::instruction::revoke(&spl_token::id(), &source, &owner, &[])?;
        assert_eq!(revoke_sf, revoke_ix);
        Ok(())
    }

    #[test]
    fn test_set_authority() -> Result<()> {
        let account = Pubkey::new_unique();
        let current_authority = Pubkey::new_unique();
        let new_authority = Some(Pubkey::new_unique());
        let authority_type = AuthorityType::AccountOwner;
        let authority_type_spl = spl_token::instruction::AuthorityType::AccountOwner;

        let set_authority_sf = Token::instruction(
            &SetAuthority {
                authority_type,
                new_authority,
            },
            SetAuthorityClientAccounts {
                account,
                current_authority,
            },
        )?;

        let set_authority_ix = spl_token::instruction::set_authority(
            &spl_token::id(),
            &account,
            new_authority.as_ref(),
            authority_type_spl,
            &current_authority,
            &[],
        )?;
        assert_eq!(set_authority_sf, set_authority_ix);
        Ok(())
    }

    #[test]
    fn test_mint_to() -> Result<()> {
        let mint = Pubkey::new_unique();
        let account = Pubkey::new_unique();
        let mint_authority = Pubkey::new_unique();
        let amount = 1000u64;

        let mint_to_sf = Token::instruction(
            &MintTo { amount },
            MintToClientAccounts {
                mint,
                account,
                mint_authority,
            },
        )?;

        let mint_to_ix = spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint,
            &account,
            &mint_authority,
            &[],
            amount,
        )?;
        assert_eq!(mint_to_sf, mint_to_ix);
        Ok(())
    }

    #[test]
    fn test_burn() -> Result<()> {
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let amount = 500u64;

        let burn_sf = Token::instruction(
            &Burn { amount },
            BurnClientAccounts {
                account,
                mint,
                owner,
            },
        )?;

        let burn_ix =
            spl_token::instruction::burn(&spl_token::id(), &account, &mint, &owner, &[], amount)?;
        assert_eq!(burn_sf, burn_ix);
        Ok(())
    }

    #[test]
    fn test_close_account() -> Result<()> {
        let account = Pubkey::new_unique();
        let destination = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let close_account_sf = Token::instruction(
            &CloseAccount,
            CloseAccountClientAccounts {
                account,
                destination,
                owner,
            },
        )?;

        let close_account_ix = spl_token::instruction::close_account(
            &spl_token::id(),
            &account,
            &destination,
            &owner,
            &[],
        )?;
        assert_eq!(close_account_sf, close_account_ix);
        Ok(())
    }

    #[test]
    fn test_freeze_account() -> Result<()> {
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let freeze_account_sf = Token::instruction(
            &FreezeAccount,
            FreezeAccountClientAccounts {
                account,
                mint,
                authority,
            },
        )?;

        let freeze_account_ix = spl_token::instruction::freeze_account(
            &spl_token::id(),
            &account,
            &mint,
            &authority,
            &[],
        )?;
        assert_eq!(freeze_account_sf, freeze_account_ix);
        Ok(())
    }

    #[test]
    fn test_thaw_account() -> Result<()> {
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let thaw_account_sf = Token::instruction(
            &ThawAccount,
            ThawAccountClientAccounts {
                account,
                mint,
                authority,
            },
        )?;

        let thaw_account_ix = spl_token::instruction::thaw_account(
            &spl_token::id(),
            &account,
            &mint,
            &authority,
            &[],
        )?;
        assert_eq!(thaw_account_sf, thaw_account_ix);
        Ok(())
    }

    #[test]
    fn test_transfer_checked() -> Result<()> {
        let source = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let destination = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let amount = 100u64;
        let decimals = 2u8;

        let transfer_checked_sf = Token::instruction(
            &TransferChecked { amount, decimals },
            TransferCheckedClientAccounts {
                source,
                mint,
                destination,
                owner,
            },
        )?;

        let transfer_checked_ix = spl_token::instruction::transfer_checked(
            &spl_token::id(),
            &source,
            &mint,
            &destination,
            &owner,
            &[],
            amount,
            decimals,
        )?;
        assert_eq!(transfer_checked_sf, transfer_checked_ix);
        Ok(())
    }

    #[test]
    fn test_approve_checked() -> Result<()> {
        let source = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let delegate = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let amount = 50u64;
        let decimals = 2u8;

        let approve_checked_sf = Token::instruction(
            &ApproveChecked { amount, decimals },
            ApproveCheckedClientAccounts {
                source,
                mint,
                delegate,
                owner,
            },
        )?;

        let approve_checked_ix = spl_token::instruction::approve_checked(
            &spl_token::id(),
            &source,
            &mint,
            &delegate,
            &owner,
            &[],
            amount,
            decimals,
        )?;
        assert_eq!(approve_checked_sf, approve_checked_ix);
        Ok(())
    }

    #[test]
    fn test_mint_to_checked() -> Result<()> {
        let mint = Pubkey::new_unique();
        let account = Pubkey::new_unique();
        let mint_authority = Pubkey::new_unique();
        let amount = 1000u64;
        let decimals = 2u8;

        let mint_to_checked_sf = Token::instruction(
            &MintToChecked { amount, decimals },
            MintToCheckedClientAccounts {
                mint,
                account,
                mint_authority,
            },
        )?;

        let mint_to_checked_ix = spl_token::instruction::mint_to_checked(
            &spl_token::id(),
            &mint,
            &account,
            &mint_authority,
            &[],
            amount,
            decimals,
        )?;
        assert_eq!(mint_to_checked_sf, mint_to_checked_ix);
        Ok(())
    }

    #[test]
    fn test_burn_checked() -> Result<()> {
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let amount = 500u64;
        let decimals = 2u8;

        let burn_checked_sf = Token::instruction(
            &BurnChecked { amount, decimals },
            BurnCheckedClientAccounts {
                account,
                mint,
                owner,
            },
        )?;

        let burn_checked_ix = spl_token::instruction::burn_checked(
            &spl_token::id(),
            &account,
            &mint,
            &owner,
            &[],
            amount,
            decimals,
        )?;
        assert_eq!(burn_checked_sf, burn_checked_ix);
        Ok(())
    }

    #[test]
    fn test_sync_native() -> Result<()> {
        let account = Pubkey::new_unique();

        let sync_native_sf = Token::instruction(&SyncNative, SyncNativeClientAccounts { account })?;

        let sync_native_ix = spl_token::instruction::sync_native(&spl_token::id(), &account)?;
        assert_eq!(sync_native_sf, sync_native_ix);
        Ok(())
    }

    #[test]
    fn test_initialize_account2() -> Result<()> {
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let initialize_account2_sf = Token::instruction(
            &InitializeAccount2 { owner },
            InitializeAccount2ClientAccounts {
                account,
                mint,
                rent: None,
            },
        )?;

        let initialize_account2_ix =
            spl_token::instruction::initialize_account2(&spl_token::id(), &account, &mint, &owner)?;
        assert_eq!(initialize_account2_sf, initialize_account2_ix);
        Ok(())
    }

    #[test]
    fn test_initialize_account3() -> Result<()> {
        let account = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let owner = Pubkey::new_unique();

        let initialize_account3_sf = Token::instruction(
            &InitializeAccount3 { owner },
            InitializeAccount3ClientAccounts { account, mint },
        )?;

        let initialize_account3_ix =
            spl_token::instruction::initialize_account3(&spl_token::id(), &account, &mint, &owner)?;
        assert_eq!(initialize_account3_sf, initialize_account3_ix);
        Ok(())
    }

    #[test]
    fn test_initialize_multisig2() -> Result<()> {
        let multisig = Pubkey::new_unique();
        let signers = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let m = 2u8;

        let initialize_multisig2_sf = Token::instruction(
            &InitializeMultisig2 { m },
            InitializeMultisig2ClientAccounts {
                multisig,
                signers: signers.clone(),
            },
        )?;

        let signers = signers.iter().collect_vec();

        let initialize_multisig2_ix =
            spl_token::instruction::initialize_multisig2(&spl_token::id(), &multisig, &signers, m)?;
        assert_eq!(initialize_multisig2_sf, initialize_multisig2_ix);
        Ok(())
    }

    #[test]
    fn test_initialize_mint2() -> Result<()> {
        let decimals = 6u8;
        let mint = Pubkey::new_unique();
        let mint_authority = Pubkey::new_unique();
        let freeze_authority = None;

        let initialize_mint2_sf = Token::instruction(
            &InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            },
            InitializeMint2ClientAccounts { mint },
        )?;

        let initialize_mint2_ix = spl_token::instruction::initialize_mint2(
            &spl_token::id(),
            &mint,
            &mint_authority,
            freeze_authority.as_ref(),
            decimals,
        )?;
        assert_eq!(initialize_mint2_sf, initialize_mint2_ix);
        Ok(())
    }

    #[test]
    fn test_get_account_data_size() -> Result<()> {
        let mint = Pubkey::new_unique();

        let get_account_data_size_sf = Token::instruction(
            &GetAccountDataSize,
            GetAccountDataSizeClientAccounts { mint },
        )?;

        let get_account_data_size_ix =
            spl_token::instruction::get_account_data_size(&spl_token::id(), &mint)?;
        assert_eq!(get_account_data_size_sf, get_account_data_size_ix);
        Ok(())
    }

    #[test]
    fn test_initialize_immutable_owner() -> Result<()> {
        let account = Pubkey::new_unique();

        let initialize_immutable_owner_sf = Token::instruction(
            &InitializeImmutableOwner,
            InitializeImmutableOwnerClientAccounts { account },
        )?;

        let initialize_immutable_owner_ix =
            spl_token::instruction::initialize_immutable_owner(&spl_token::id(), &account)?;
        assert_eq!(initialize_immutable_owner_sf, initialize_immutable_owner_ix);
        Ok(())
    }

    #[test]
    fn test_amount_to_ui_amount() -> Result<()> {
        let mint = Pubkey::new_unique();
        let amount = 1000u64;

        let amount_to_ui_amount_sf = Token::instruction(
            &AmountToUiAmount { amount },
            AmountToUiAmountClientAccounts { mint },
        )?;

        let amount_to_ui_amount_ix =
            spl_token::instruction::amount_to_ui_amount(&spl_token::id(), &mint, amount)?;
        assert_eq!(amount_to_ui_amount_sf, amount_to_ui_amount_ix);
        Ok(())
    }
}
