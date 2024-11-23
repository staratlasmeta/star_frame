use borsh::{BorshDeserialize, BorshSerialize};
use star_frame::empty_star_frame_instruction;
use star_frame::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct AssociatedTokenProgram;

impl StarFrameProgram for AssociatedTokenProgram {
    type InstructionSet = AssociatedTokenInstructionSet;
    type AccountDiscriminant = ();
    /// See [`spl_associated_token_account::ID`].
    /// ```
    /// # use star_frame::program::StarFrameProgram;
    /// # use star_frame_spl::associated_token::AssociatedTokenProgram;
    /// assert_eq!(AssociatedTokenProgram::PROGRAM_ID, spl_associated_token_account::ID);
    /// ```
    const PROGRAM_ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
impl ProgramToIdl for AssociatedTokenProgram {
    fn crate_metadata() -> star_frame::star_frame_idl::CrateMetadata {
        star_frame::star_frame_idl::CrateMetadata {
            version: star_frame::star_frame_idl::Version::new(3, 0, 4),
            name: "associated_token".to_string(),
            docs: vec![],
            description: None,
            homepage: None,
            license: None,
            repository: None,
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, InstructionSet)]
#[ix_set(use_repr)]
#[repr(u8)]
pub enum AssociatedTokenInstructionSet {
    Create(Create),
    // CreateIdempotent(), todo
    // InitializeMultisig(), todo
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::{FindIdlSeeds, FindSeed};
    use star_frame::star_frame_idl::seeds::IdlFindSeed;
    #[derive(Debug, Clone)]
    pub struct FindAtaSeeds {
        pub wallet: FindSeed<Pubkey>,
        pub mint: FindSeed<Pubkey>,
    }
    impl FindIdlSeeds for FindAtaSeeds {
        fn find_seeds(&self) -> Result<Vec<IdlFindSeed>> {
            Ok(vec![
                Into::into(&self.wallet),
                IdlFindSeed::Const(TokenProgram::PROGRAM_ID.as_ref().to_vec()),
                Into::into(&self.mint),
            ])
        }
    }
}

use crate::token::TokenProgram;
#[cfg(all(feature = "idl", not(target_os = "solana")))]
use idl_impl::*;

// create
/// See [`spl_associated_token_account::instruction::AssociatedTokenAccountInstruction::Create`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = AssociatedTokenProgram)]
pub struct Create;
/// Accounts for the [`Create`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct CreateAccounts<'info> {
    pub funder: Mut<Signer<AccountInfo<'info>>>,
    #[idl(arg =
        Seeds(FindAtaSeeds {
            wallet: seed_path("wallet"),
            mint: seed_path("mint"),
        })
    )]
    pub token_account: Mut<AccountInfo<'info>>,
    pub wallet: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, SystemProgram>,
    pub token_program: Program<'info, TokenProgram>,
}
empty_star_frame_instruction!(Create, CreateAccounts);

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "idl")]
    #[test]
    fn print_token_idl() -> Result<()> {
        let idl = AssociatedTokenProgram::program_to_idl()?;
        println!("{}", star_frame::serde_json::to_string_pretty(&idl)?);
        Ok(())
    }
}
