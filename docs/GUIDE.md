# Star Frame Developer Guide

A step-by-step guide to building a complete Solana program with Star Frame.

## What We're Building

A **voting program** where:
- An admin creates a proposal with a title
- Users can vote "yes" or "no" (one vote per user per proposal)
- Votes are tracked on-chain with PDA-derived vote receipt accounts
- The admin can close the proposal

This covers all the Star Frame fundamentals: account creation, PDAs, validation, state management, and account closing.

## Prerequisites

- Rust 1.84.1+ (Star Frame's minimum supported version)
- Solana CLI tools (`cargo build-sbf`)
- Basic Solana knowledge (accounts, transactions, PDAs)

## Step 1: Project Setup

```bash
cargo install star_frame_cli
sf new voting_program
cd voting_program
```

Or manually:

```bash
cargo init --lib voting_program
cd voting_program
```

**Cargo.toml:**

```toml
[package]
name = "voting_program"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]

[features]
default = []
idl = ["star_frame/idl"]
test_helpers = ["star_frame/test_helpers"]

[dependencies]
star_frame = "0.29"
bytemuck = { version = "1.22", features = ["derive"] }
borsh = { version = "1.5", features = ["derive"] }
```

## Step 2: Define the Program

Every Star Frame program starts with a program struct and an instruction set.

**src/lib.rs:**

```rust
use star_frame::prelude::*;

// The program struct — this generates the Solana entrypoint
#[derive(StarFrameProgram)]
#[program(
    instruction_set = VotingInstructionSet,
    id = "Vote111111111111111111111111111111111111111",
    errors = VotingError,
)]
pub struct VotingProgram;

// All instructions your program handles
#[derive(InstructionSet)]
pub enum VotingInstructionSet {
    CreateProposal(CreateProposal),
    CastVote(CastVote),
    CloseProposal(CloseProposal),
}
```

**Key points:**
- `id` is your deployed program ID (use a placeholder during development)
- `instruction_set` links to the enum that defines all instructions
- `errors` (optional) links to a custom error enum
- The `InstructionSet` enum variants wrap instruction data structs

## Step 3: Define Account State

### Proposal Account

```rust
// zero_copy(pod) makes this a bytemuck type — no deserialization overhead
#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
#[program_account(seeds = ProposalSeeds)]
pub struct Proposal {
    pub admin: Pubkey,
    pub title: [u8; 64],       // Fixed-size string (pad with zeros)
    pub title_len: u8,         // Actual length of title
    pub yes_votes: u64,
    pub no_votes: u64,
    pub is_active: u8,         // 0 = closed, 1 = active (no bool in Pod)
}
```

**Why no `String`?** Zero-copy types must be fixed-size. Use byte arrays with a length field. For truly dynamic data, see the `unsized_type` system.

**Why `u8` instead of `bool`?** `bytemuck::Pod` requires types where every bit pattern is valid. `bool` only allows 0 and 1, so we use `u8`.

### Vote Receipt Account

```rust
#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
#[program_account(seeds = VoteReceiptSeeds)]
pub struct VoteReceipt {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote: u8,              // 0 = no, 1 = yes
}
```

### PDA Seeds

```rust
// Proposal PDA: ["PROPOSAL", admin, proposal_id]
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"PROPOSAL")]
pub struct ProposalSeeds {
    pub admin: Pubkey,
    pub proposal_id: u64,
}

// Vote receipt PDA: ["VOTE", proposal, voter]
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"VOTE")]
pub struct VoteReceiptSeeds {
    pub proposal: Pubkey,
    pub voter: Pubkey,
}
```

**How `GetSeeds` works:** The derive macro produces `seeds()` → `vec![b"PROPOSAL", admin.as_bytes(), proposal_id.as_bytes(), &[]]`. The trailing `&[]` is a placeholder for the bump byte.

## Step 4: Define Custom Errors

```rust
#[star_frame_error]
pub enum VotingError {
    #[msg("Proposal is not active")]
    ProposalNotActive,
    #[msg("Invalid vote value (must be 0 or 1)")]
    InvalidVote,
    #[msg("Only the admin can perform this action")]
    Unauthorized,
}
```

## Step 5: Create Proposal Instruction

### Instruction Data

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct CreateProposal {
    #[ix_args(run)]
    pub proposal_id: u64,
    #[ix_args(run)]
    pub title: Vec<u8>,       // Borsh supports Vec for instruction data
}
```

`#[ix_args(run)]` means these fields are passed to the `process` function. The instruction data itself is Borsh-serialized (variable-length is fine for instruction data — just not for zero-copy accounts).

### Account Set

```rust
#[derive(AccountSet)]
pub struct CreateProposalAccounts {
    // The admin pays for account creation
    #[validate(funder)]
    pub admin: Signer<Mut<SystemAccount>>,

    // The proposal PDA — created during this instruction
    #[validate(arg = (
        Create(()),
        Seeds(ProposalSeeds {
            admin: *self.admin.pubkey(),
            proposal_id: self.proposal_id,
        }),
    ))]
    pub proposal: Init<Seeded<Account<Proposal>>>,

    // System program is required for account creation
    pub system_program: Program<System>,

    // Hidden field: proposal_id is set during decode for use in validation
    #[account_set(skip_decode)]
    #[validate(skip)]
    proposal_id: u64,
}
```

**Breaking down `#[validate(arg = (Create(()), Seeds(...)))]`:**

This is a tuple of validation arguments that get distributed to the modifier stack:
- `Create(())` goes to `Init` — tells it to create the account (fails if it exists)
- `Seeds(...)` goes to `Seeded` — validates/derives the PDA address

**The `funder` tag:** `#[validate(funder)]` caches this account in the `Context` so that `Init` can use it to fund account creation without you passing it explicitly.

### Instruction Logic

```rust
#[star_frame_instruction]
fn CreateProposal(
    accounts: &mut CreateProposalAccounts,
    CreateProposal { proposal_id: _, title }: CreateProposal,
) -> Result<()> {
    let mut proposal = accounts.proposal.data_mut()?;

    // Copy title bytes (truncate if too long)
    let len = title.len().min(64);
    proposal.title[..len].copy_from_slice(&title[..len]);
    proposal.title_len = len as u8;
    proposal.admin = *accounts.admin.pubkey();
    proposal.yes_votes = 0;
    proposal.no_votes = 0;
    proposal.is_active = 1;

    Ok(())
}
```

**How `data_mut()` works:** Returns an `ExclusiveWrapper` that derefs to `&mut Proposal`. When you do `**accounts.proposal.data_mut()? = Proposal { ... }`, you're writing directly to the account's data buffer — zero-copy, no serialization.

## Step 6: Cast Vote Instruction

### Custom Account Validation

For accounts that need business logic validation, implement `AccountValidate`:

```rust
pub struct ValidateProposalActive;

impl AccountValidate<ValidateProposalActive> for Proposal {
    fn validate_account(self_ref: &Self::Ptr, _arg: ValidateProposalActive) -> Result<()> {
        ensure!(self_ref.is_active == 1, VotingError::ProposalNotActive);
        Ok(())
    }
}
```

### Instruction Data

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct CastVote {
    #[ix_args(run)]
    pub vote: u8,   // 0 = no, 1 = yes
}
```

### Account Set

```rust
#[derive(AccountSet)]
pub struct CastVoteAccounts {
    #[validate(funder)]
    pub voter: Signer<Mut<SystemAccount>>,

    // The proposal must be active — validated via AccountValidate
    #[validate(arg = ValidateProposalActive)]
    pub proposal: Mut<ValidatedAccount<Proposal>>,

    // Vote receipt PDA — one per voter per proposal
    #[validate(arg = (
        Create(()),
        Seeds(VoteReceiptSeeds {
            proposal: *self.proposal.pubkey(),
            voter: *self.voter.pubkey(),
        }),
    ))]
    pub vote_receipt: Init<Seeded<Account<VoteReceipt>>>,

    pub system_program: Program<System>,
}
```

**`ValidatedAccount<Proposal>` vs `Account<Proposal>`:** `ValidatedAccount` calls your `AccountValidate` implementation during the validation phase. `Account` only checks owner + discriminant.

### Instruction Logic

```rust
#[star_frame_instruction]
fn CastVote(accounts: &mut CastVoteAccounts, CastVote { vote }: CastVote) -> Result<()> {
    ensure!(vote <= 1, VotingError::InvalidVote);

    // Record the vote receipt
    **accounts.vote_receipt.data_mut()? = VoteReceipt {
        voter: *accounts.voter.pubkey(),
        proposal: *accounts.proposal.pubkey(),
        vote,
    };

    // Update vote counts
    let mut proposal = accounts.proposal.data_mut()?;
    if vote == 1 {
        proposal.yes_votes += 1;
    } else {
        proposal.no_votes += 1;
    }

    Ok(())
}
```

## Step 7: Close Proposal Instruction

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct CloseProposal;

#[derive(AccountSet)]
pub struct CloseProposalAccounts {
    #[validate(address = &self.proposal.data()?.admin)]
    pub admin: Signer,

    #[validate(recipient)]
    pub funds_to: Mut<SystemAccount>,

    // CloseAccount sends lamports to the cached recipient and zeros the data
    #[cleanup(arg = CloseAccount(()))]
    pub proposal: Mut<Account<Proposal>>,
}

#[star_frame_instruction]
fn CloseProposal(accounts: &mut CloseProposalAccounts) -> Result<()> {
    // Mark as inactive before closing
    accounts.proposal.data_mut()?.is_active = 0;
    Ok(())
}
```

**How account closing works:**
1. `#[validate(recipient)]` caches `funds_to` in the Context
2. `#[cleanup(arg = CloseAccount(()))]` runs after `process`:
   - Transfers all lamports from `proposal` to the cached recipient
   - Zeros out the account data (writes `0xFF` to discriminant)
3. The account is effectively closed

## Step 8: Putting It All Together

Your complete `src/lib.rs`:

```rust
use star_frame::prelude::*;

// === Program Definition ===

#[derive(StarFrameProgram)]
#[program(
    instruction_set = VotingInstructionSet,
    id = "Vote111111111111111111111111111111111111111",
    errors = VotingError,
)]
pub struct VotingProgram;

#[derive(InstructionSet)]
pub enum VotingInstructionSet {
    CreateProposal(CreateProposal),
    CastVote(CastVote),
    CloseProposal(CloseProposal),
}

#[star_frame_error]
pub enum VotingError {
    #[msg("Proposal is not active")]
    ProposalNotActive,
    #[msg("Invalid vote value")]
    InvalidVote,
    #[msg("Unauthorized")]
    Unauthorized,
}

// === Account Types ===

#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
#[program_account(seeds = ProposalSeeds)]
pub struct Proposal {
    pub admin: Pubkey,
    pub title: [u8; 64],
    pub title_len: u8,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub is_active: u8,
}

#[zero_copy(pod)]
#[derive(ProgramAccount, Default, Debug, Eq, PartialEq)]
#[program_account(seeds = VoteReceiptSeeds)]
pub struct VoteReceipt {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote: u8,
}

// === Seeds ===

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"PROPOSAL")]
pub struct ProposalSeeds {
    pub admin: Pubkey,
    pub proposal_id: u64,
}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"VOTE")]
pub struct VoteReceiptSeeds {
    pub proposal: Pubkey,
    pub voter: Pubkey,
}

// === Validation ===

pub struct ValidateProposalActive;

impl AccountValidate<ValidateProposalActive> for Proposal {
    fn validate_account(self_ref: &Self::Ptr, _arg: ValidateProposalActive) -> Result<()> {
        ensure!(self_ref.is_active == 1, VotingError::ProposalNotActive);
        Ok(())
    }
}

// === Instructions ===

// -- CreateProposal --

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct CreateProposal {
    #[ix_args(run)]
    pub proposal_id: u64,
    #[ix_args(run)]
    pub title: Vec<u8>,
}

#[derive(AccountSet)]
pub struct CreateProposalAccounts {
    #[validate(funder)]
    pub admin: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(ProposalSeeds {
            admin: *self.admin.pubkey(),
            proposal_id: 0, // Set in decode
        }),
    ))]
    pub proposal: Init<Seeded<Account<Proposal>>>,
    pub system_program: Program<System>,
}

#[star_frame_instruction]
fn CreateProposal(
    accounts: &mut CreateProposalAccounts,
    CreateProposal { proposal_id: _, title }: CreateProposal,
) -> Result<()> {
    let mut proposal = accounts.proposal.data_mut()?;
    let len = title.len().min(64);
    proposal.title[..len].copy_from_slice(&title[..len]);
    proposal.title_len = len as u8;
    proposal.admin = *accounts.admin.pubkey();
    proposal.is_active = 1;
    Ok(())
}

// -- CastVote --

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct CastVote {
    #[ix_args(run)]
    pub vote: u8,
}

#[derive(AccountSet)]
pub struct CastVoteAccounts {
    #[validate(funder)]
    pub voter: Signer<Mut<SystemAccount>>,
    #[validate(arg = ValidateProposalActive)]
    pub proposal: Mut<ValidatedAccount<Proposal>>,
    #[validate(arg = (
        Create(()),
        Seeds(VoteReceiptSeeds {
            proposal: *self.proposal.pubkey(),
            voter: *self.voter.pubkey(),
        }),
    ))]
    pub vote_receipt: Init<Seeded<Account<VoteReceipt>>>,
    pub system_program: Program<System>,
}

#[star_frame_instruction]
fn CastVote(accounts: &mut CastVoteAccounts, CastVote { vote }: CastVote) -> Result<()> {
    ensure!(vote <= 1, VotingError::InvalidVote);
    **accounts.vote_receipt.data_mut()? = VoteReceipt {
        voter: *accounts.voter.pubkey(),
        proposal: *accounts.proposal.pubkey(),
        vote,
    };
    let mut proposal = accounts.proposal.data_mut()?;
    if vote == 1 { proposal.yes_votes += 1; } else { proposal.no_votes += 1; }
    Ok(())
}

// -- CloseProposal --

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct CloseProposal;

#[derive(AccountSet)]
pub struct CloseProposalAccounts {
    #[validate(address = &self.proposal.data()?.admin)]
    pub admin: Signer,
    #[validate(recipient)]
    pub funds_to: Mut<SystemAccount>,
    #[cleanup(arg = CloseAccount(()))]
    pub proposal: Mut<Account<Proposal>>,
}

#[star_frame_instruction]
fn CloseProposal(accounts: &mut CloseProposalAccounts) -> Result<()> {
    accounts.proposal.data_mut()?.is_active = 0;
    Ok(())
}
```

## Step 9: Testing

Add to `Cargo.toml`:
```toml
[dev-dependencies]
mollusk-svm = "0.7"
solana-account = "3.0"
solana-keypair = "3.0"
solana-signer = "3.0"
```

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use mollusk_svm::{result::Check, Mollusk};
    use solana_account::Account as SolanaAccount;
    use star_frame::client::{DeserializeAccount, SerializeAccount};

    #[test]
    fn test_create_and_vote() -> Result<()> {
        if std::env::var("SBF_OUT_DIR").is_err() {
            println!("SBF_OUT_DIR not set, skipping");
            return Ok(());
        }

        let mollusk = Mollusk::new(&VotingProgram::ID, "voting_program");

        let admin = Pubkey::new_unique();
        let voter = Pubkey::new_unique();
        let proposal_id = 1u64;

        let seeds = ProposalSeeds { admin, proposal_id };
        let (proposal_pda, _) = Pubkey::find_program_address(
            &seeds.seeds(),
            &VotingProgram::ID,
        );

        let vote_seeds = VoteReceiptSeeds {
            proposal: proposal_pda,
            voter,
        };
        let (vote_receipt_pda, _) = Pubkey::find_program_address(
            &vote_seeds.seeds(),
            &VotingProgram::ID,
        );

        let mollusk = mollusk.with_context(HashMap::from_iter([
            (admin, SolanaAccount::new(1_000_000_000, 0, &System::ID)),
            (voter, SolanaAccount::new(1_000_000_000, 0, &System::ID)),
            (proposal_pda, SolanaAccount::new(0, 0, &System::ID)),
            (vote_receipt_pda, SolanaAccount::new(0, 0, &System::ID)),
            mollusk_svm::program::keyed_account_for_system_program(),
        ]));

        // Create proposal
        mollusk.process_and_validate_instruction(
            &VotingProgram::instruction(
                &CreateProposal {
                    proposal_id,
                    title: b"Should we adopt Star Frame?".to_vec(),
                },
                CreateProposalClientAccounts {
                    admin,
                    proposal: proposal_pda,
                    system_program: None,
                },
            )?,
            &[Check::success()],
        );

        // Cast vote
        mollusk.process_and_validate_instruction(
            &VotingProgram::instruction(
                &CastVote { vote: 1 },
                CastVoteClientAccounts {
                    voter,
                    proposal: proposal_pda,
                    vote_receipt: vote_receipt_pda,
                    system_program: None,
                },
            )?,
            &[Check::success()],
        );

        Ok(())
    }
}
```

## Step 10: Build and Deploy

```bash
# Build
cargo build-sbf

# Deploy to devnet
solana config set --url devnet
solana program deploy target/deploy/voting_program.so

# Generate IDL for client generation
cargo test --features idl -- generate_idl
```

## Next Steps

- Read the [API Reference](API_REFERENCE.md) for all available types and traits
- Check [Pitfalls](PITFALLS.md) before going to production
- Look at the [marketplace example](https://github.com/staratlasmeta/star_frame/tree/main/example_programs/marketplace) for a more complex program with SPL tokens and unsized types
