//! Basic-1: Data Storage and Updates
//!
//! This example demonstrates:
//! - Creating and initializing accounts with PDAs
//! - Storing data on-chain
//! - Updating stored data
//! - Using seed-based account derivation
//!
//! Key concepts:
//! - InstructionSet for routing
//! - ProgramAccount with seeds
//! - StarFrameInstruction implementation
//! - Account validation with Init and Seeded wrappers

use star_frame::prelude::*;

// Main program struct - the entry point for our program
#[derive(StarFrameProgram)]
#[program(
    instruction_set = BasicInstructionSet,
    id = "B1sic11111111111111111111111111111111111111"
)]
pub struct BasicProgram;

// Instruction routing enum - defines all instructions this program accepts
#[derive(InstructionSet)]
pub enum BasicInstructionSet {
    Initialize(Initialize),
    Update(Update),
}

// On-chain account structure
// Key traits:
// - Align1, Pod, Zeroable: Enable zero-copy deserialization
// - ProgramAccount: Marks this as an account owned by our program
// - repr(C, packed): Ensures predictable memory layout
#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = DataSeeds)]
#[repr(C, packed)]
pub struct DataAccount {
    pub data: u64,
}

// Seeds for PDA derivation
// This struct defines how to derive the account address deterministically
#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"data")]
pub struct DataSeeds {
    pub authority: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize {
    #[ix_args(&run)]
    pub initial_value: u64,
}

#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub authority: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(DataSeeds { authority: *self.authority.pubkey() }),
    ))]
    pub data_account: Init<Seeded<Account<DataAccount>>>,
    pub system_program: Program<System>,
}

// Instruction implementation
// This is where the actual logic happens
impl StarFrameInstruction for Initialize {
    type ReturnType = ();
    type Accounts<'b, 'c> = InitializeAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        initial_value: &u64,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        // Direct memory write using data_mut() for zero-copy performance
        **accounts.data_account.data_mut()? = DataAccount {
            data: *initial_value,
        };
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Update {
    #[ix_args(&run)]
    pub value: u64,
}

#[derive(AccountSet)]
pub struct UpdateAccounts {
    pub authority: Signer,
    #[validate(arg = Seeds(DataSeeds { authority: *self.authority.pubkey() }))]
    pub data_account: Mut<Seeded<Account<DataAccount>>>,
}

impl StarFrameInstruction for Update {
    type ReturnType = ();
    type Accounts<'b, 'c> = UpdateAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        value: &u64,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        let mut data = accounts.data_account.data_mut()?;
        data.data = *value;
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
