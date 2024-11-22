pub mod instructions;
mod state;

use crate::token::instructions::TokenInstructionSet;
use star_frame::prelude::*;
pub use state::*;

#[derive(Debug)]
pub struct TokenProgram;

impl StarFrameProgram for TokenProgram {
    type InstructionSet = TokenInstructionSet;
    type AccountDiscriminant = ();
    const PROGRAM_ID: Pubkey = spl_token::ID;
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
impl ProgramToIdl for TokenProgram {
    fn crate_metadata() -> star_frame::star_frame_idl::CrateMetadata {
        star_frame::star_frame_idl::CrateMetadata {
            version: star_frame::star_frame_idl::Version::new(7, 0, 0),
            name: "spl_token".to_string(),
            docs: vec![],
            description: None,
            homepage: None,
            license: None,
            repository: None,
        }
    }
}
