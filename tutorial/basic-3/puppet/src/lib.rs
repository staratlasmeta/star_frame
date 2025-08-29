//! Basic-3: Puppet Program (CPI Target)
//!
//! This is the "puppet" program that will be controlled by the puppet-master.
//! It demonstrates:
//! - Being a CPI target program
//! - Simple state management
//! - Public instruction interface for other programs
//!
//! Key concepts:
//! - Programs can be called by other programs
//! - Account ownership remains with this program
//! - Instructions are public interfaces

use star_frame::prelude::*;

/// The Puppet program - designed to be controlled via CPI
///
/// This program owns PuppetAccount data and exposes
/// instructions that other programs can call
#[derive(StarFrameProgram)]
#[program(
    instruction_set = PuppetInstructionSet,
    id = "Pupp3t1111111111111111111111111111111111111"
)]
pub struct PuppetProgram;

/// Instructions exposed by the Puppet program
/// These can be called directly OR via CPI from other programs
#[derive(InstructionSet)]
pub enum PuppetInstructionSet {
    Initialize(Initialize),
    SetData(SetData),
}

/// The Puppet's data account
///
/// This account is owned by the Puppet program, but can be
/// modified through CPI calls from authorized programs
#[derive(Align1, Pod, Zeroable, Default, Copy, Clone, Debug, Eq, PartialEq, ProgramAccount)]
#[program_account(seeds = PuppetSeeds)]
#[repr(C, packed)]
pub struct PuppetAccount {
    pub data: u64,
}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"puppet")]
pub struct PuppetSeeds {
    pub authority: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct Initialize;

#[derive(AccountSet)]
pub struct InitializeAccounts {
    #[validate(funder)]
    pub authority: Signer<Mut<SystemAccount>>,
    #[validate(arg = (
        Create(()),
        Seeds(PuppetSeeds { authority: *self.authority.pubkey() }),
    ))]
    pub puppet: Init<Seeded<Account<PuppetAccount>>>,
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
        **accounts.puppet.data_mut()? = PuppetAccount { data: 0 };
        Ok(())
    }
}

/// SetData instruction - Can be called via CPI
///
/// IMPORTANT: This instruction is public and can be called by any program.
/// In production, you'd add validation to restrict which programs can call it.
#[derive(BorshSerialize, BorshDeserialize, Debug, InstructionArgs)]
pub struct SetData {
    #[ix_args(&run)]
    pub data: u64,
}

/// Accounts for SetData - Note the simplicity
///
/// When called via CPI, the calling program provides these accounts.
/// The puppet account must be owned by this program.
#[derive(AccountSet)]
pub struct SetDataAccounts {
    pub puppet: Mut<Account<PuppetAccount>>,
}

/// SetData implementation - Executes whether called directly or via CPI
///
/// Star Frame handles CPI context automatically - this code doesn't
/// need to know if it's being called directly or through CPI
impl StarFrameInstruction for SetData {
    type ReturnType = ();
    type Accounts<'b, 'c> = SetDataAccounts;

    fn process(
        accounts: &mut Self::Accounts<'_, '_>,
        data: &u64,
        _ctx: &mut Context,
    ) -> Result<Self::ReturnType> {
        // This executes in the Puppet program's context
        // even when called via CPI
        let mut puppet = accounts.puppet.data_mut()?;
        puppet.data = *data;
        Ok(())
    }
}
