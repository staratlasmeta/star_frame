mod instructions;
mod state;

pub use instructions::*;
use star_frame::prelude::*;
pub use state::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct TokenProgram;

impl StarFrameProgram for TokenProgram {
    type InstructionSet = TokenInstructionSet;
    type AccountDiscriminant = ();
    /// See [`spl_token::ID`].
    /// ```
    /// # use star_frame::program::StarFrameProgram;
    /// # use star_frame_spl::token::TokenProgram;
    /// assert_eq!(TokenProgram::PROGRAM_ID, spl_token::ID);
    /// ```
    const PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
impl ProgramToIdl for TokenProgram {
    fn crate_metadata() -> star_frame::star_frame_idl::CrateMetadata {
        star_frame::star_frame_idl::CrateMetadata {
            version: star_frame::star_frame_idl::Version::new(4, 0, 0),
            name: "spl_token".to_string(),
            docs: vec![],
            description: None,
            homepage: None,
            license: None,
            repository: None,
        }
    }
}
