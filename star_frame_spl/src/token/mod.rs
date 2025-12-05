pub mod instructions;
pub mod state;

// Avoid name collisions with glob
use star_frame::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Token;

impl StarFrameProgram for Token {
    type InstructionSet = instructions::TokenInstructionSet;
    type AccountDiscriminant = ();
    /// See [`spl_token_interface::ID`].
    /// ```
    /// # use star_frame::program::StarFrameProgram;
    /// # use star_frame_spl::token::Token;
    /// assert_eq!(Token::ID, spl_token_interface::ID);
    /// ```
    const ID: Address = address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
impl ProgramToIdl for Token {
    type Errors = ();
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
