//! Basic-2: Access Control & Authority
//!
//! This example demonstrates:
//! - Authority-based access control
//! - Account validation using the AccountValidate trait
//! - ValidatedAccount wrapper for compile-time safety
//! - Counter pattern with restricted access
//!
//! Key concepts:
//! - AccountValidate trait for custom validation logic
//! - ValidatedAccount<T> ensures validation before use
//! - Authority pattern for ownership
//! - ensure! macro for validation checks

use star_frame::{anyhow::ensure, prelude::*};

/// The main program struct
/// Demonstrates a simple counter with authority-based access control
#[derive(StarFrameProgram)]
#[program(
    instruction_set = CounterInstructionSet,
    id = "B2sic22222222222222222222222222222222222222"
)]
pub struct CounterProgram;

/// Instruction set defining available operations
/// Initialize: Creates a new counter with an authority
/// Increment: Increases the counter (only authority can call)
#[derive(InstructionSet)]
pub enum CounterInstructionSet {
    Initialize(Initialize),
    Increment(Increment),
}

/// On-chain counter account structure
///
/// Fields:
/// - authority: The only pubkey allowed to increment the counter
/// - count: The current counter value
///
/// This account uses PDA derivation based on the authority's pubkey
#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = CounterSeeds)]
#[repr(C, packed)]
pub struct CounterAccount {
    pub authority: Pubkey,
    pub count: u64,
}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"counter")]
pub struct CounterSeeds {
    pub authority: Pubkey,
}

/// Account validation implementation
///
/// This is a KEY STAR FRAME PATTERN:
/// - Validation logic is part of the type system
/// - ValidatedAccount<T> wrapper ensures this runs before access
/// - Compile-time guarantee that only valid accounts are used
///
/// This replaces Anchor's `has_one` constraint with explicit logic
impl AccountValidate<&Pubkey> for CounterAccount {
    fn validate_account(self_ref: &Self::Ref<'_>, authority: &Pubkey) -> Result<()> {
        ensure!(
            authority == &self_ref.authority,
            "Invalid authority: expected {}, got {}",
            self_ref.authority,
            authority
        );
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize;

/// Accounts required for initialization
///
/// Account modifiers:
/// - Signer: Ensures the account signed the transaction
/// - Mut: Account will be modified
/// - Init: Account will be created
/// - Seeded: Account uses PDA derivation
#[derive(AccountSet)]
pub struct InitializeAccounts {
    /// The authority who will own the counter (pays for account creation)
    #[validate(funder)]
    pub authority: Signer<Mut<SystemAccount>>,
    /// The counter account to be created (PDA based on authority)
    #[validate(arg = (
        Create(()),
        Seeds(CounterSeeds { authority: *self.authority.pubkey() }),
    ))]
    pub counter: Init<Seeded<Account<CounterAccount>>>,
    /// System program for account creation
    pub system_program: Program<System>,
}

impl StarFrameInstruction for Initialize {
    type ReturnType = ();
    type Accounts<'b, 'c> = InitializeAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        _run_arg: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        **accounts.counter.data_mut()? = CounterAccount {
            authority: *accounts.authority.pubkey(),
            count: 0,
        };
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone, InstructionArgs)]
pub struct Increment;

/// Accounts required for incrementing
///
/// KEY PATTERN: ValidatedAccount<CounterAccount>
/// - The validate attribute passes authority.pubkey() to AccountValidate
/// - ValidatedAccount wrapper ensures validation happens before access
/// - Type system guarantees only the authority can increment
#[derive(AccountSet, Debug)]
pub struct IncrementAccounts {
    /// The authority attempting to increment (must match counter.authority)
    pub authority: Signer,
    /// The counter to increment (validated against authority)
    #[validate(arg = self.authority.pubkey())]
    pub counter: Mut<ValidatedAccount<CounterAccount>>,
}

/// Increment instruction implementation
///
/// Note: We can safely increment because ValidatedAccount<T> ensures
/// that AccountValidate::validate_account() has already succeeded.
/// This is compile-time enforcement of runtime validation!
impl StarFrameInstruction for Increment {
    type ReturnType = ();
    type Accounts<'b, 'c> = IncrementAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        _run_arg: Self::RunArg<'_>,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        // Safe to modify - validation already passed
        let mut counter = accounts.counter.data_mut()?;
        counter.count += 1;
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
