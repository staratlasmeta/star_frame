//! Basic-3: Puppet Master Program (CPI Caller)
//!
//! This program demonstrates Cross-Program Invocation (CPI) in Star Frame.
//! It calls the Puppet program to modify its data.
//!
//! Key concepts:
//! - Importing types from another program
//! - Creating CPI calls with Star Frame's Cpi struct
//! - Passing accounts to the target program
//! - Program composition and modularity
//!
//! This replaces Anchor's CpiContext pattern with a more explicit approach

use puppet::{PuppetAccount, PuppetProgram, SetData as PuppetSetData};
use star_frame::prelude::*;

/// The Puppet Master program - Controls the Puppet via CPI
///
/// This demonstrates how programs can compose and interact
/// on Solana through Cross-Program Invocation
#[derive(StarFrameProgram)]
#[program(
    instruction_set = PuppetMasterInstructionSet,
    id = "Mast3r3333333333333333333333333333333333333"
)]
pub struct PuppetMasterProgram;

/// PuppetMaster's instruction set
/// PullStrings will invoke the Puppet program via CPI
#[derive(InstructionSet)]
pub enum PuppetMasterInstructionSet {
    PullStrings(PullStrings),
}

/// PullStrings instruction - Demonstrates CPI
///
/// This instruction will call Puppet::SetData via CPI,
/// showing how one program can control another
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct PullStrings {
    #[ix_args(&run)]
    pub data: u64,
}

/// Accounts for PullStrings
///
/// Note: We need both:
/// - The puppet account (owned by Puppet program)
/// - The Puppet program itself (for CPI)
#[derive(AccountSet)]
pub struct PullStringsAccounts {
    /// The puppet account to modify (owned by Puppet program)
    pub puppet: Mut<Account<PuppetAccount>>,
    /// The Puppet program we'll invoke via CPI
    pub puppet_program: Program<puppet::PuppetProgram>,
}

/// PullStrings implementation - Shows CPI in action
///
/// KEY STAR FRAME CPI PATTERN:
/// 1. Create a Cpi struct with target program, instruction, and accounts
/// 2. Call invoke() to execute the CPI
/// 3. Star Frame handles all the low-level details
///
/// This is more explicit than Anchor's CpiContext but gives more control
impl StarFrameInstruction for PullStrings {
    type ReturnType = ();
    type Accounts<'b, 'c> = PullStringsAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        data: &u64,
        ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        // Create the CPI call using the new CpiBuilder pattern
        // This is Star Frame's equivalent to Anchor's CpiContext
        PuppetProgram::cpi(
            &PuppetSetData { data: *data }, // Instruction data
            puppet::SetDataCpiAccounts {
                // CPI accounts for the target instruction
                puppet: *accounts.puppet.account_info(),
            },
            ctx,
        )?
        .invoke()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "idl")]
    #[test]
    fn generate_idl() -> Result<()> {
        use codama_nodes::{NodeTrait, ProgramNode};
        let idl = StarFrameDeclaredProgram::program_to_idl()?;
        let codama_idl: ProgramNode = idl.try_into()?;
        let idl_json = codama_idl.to_json()?;
        std::fs::write("idl.json", &idl_json)?;
        Ok(())
    }
}
