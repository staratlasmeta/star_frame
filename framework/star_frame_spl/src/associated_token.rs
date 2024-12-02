use crate::token::TokenProgram;
use borsh::{BorshDeserialize, BorshSerialize};
use star_frame::empty_star_frame_instruction;
use star_frame::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct AssociatedTokenProgram;

impl AssociatedTokenProgram {
    /// Find the associated token address for the given wallet and mint.
    ///
    /// See [`spl_associated_token_account::get_associated_token_address`].
    /// ```
    /// # use star_frame_spl::associated_token::AssociatedTokenProgram;
    /// # use spl_associated_token_account::get_associated_token_address;
    /// # use pretty_assertions::assert_eq;
    /// # use star_frame::prelude::Pubkey;
    /// let wallet = Pubkey::new_unique();
    /// let mint = Pubkey::new_unique();
    /// assert_eq!(
    ///     AssociatedTokenProgram::find_address(&wallet, &mint),
    ///     get_associated_token_address(&wallet, &mint),
    /// );
    /// ```
    pub fn find_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[
                wallet.as_ref(),
                TokenProgram::PROGRAM_ID.as_ref(),
                mint.as_ref(),
            ],
            &Self::PROGRAM_ID,
        )
        .0
    }
}

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
mod idl_impl {
    use super::*;
    use star_frame::idl::{FindIdlSeeds, FindSeed, SeedsToIdl};
    use star_frame::star_frame_idl::seeds::{IdlFindSeed, IdlSeed, IdlSeeds};

    use crate::token::TokenProgram;
    use star_frame::star_frame_idl::IdlDefinition;

    // todo: potentially support multiple token programs here
    #[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
    pub struct AssociatedTokenSeeds {
        pub wallet: Pubkey,
        pub mint: Pubkey,
    }

    pub type AtaSeeds = AssociatedTokenSeeds;
    pub type FindAtaSeeds = FindAssociatedTokenSeeds;

    impl GetSeeds for AssociatedTokenSeeds {
        fn seeds(&self) -> Vec<&[u8]> {
            vec![
                self.wallet.seed(),
                TokenProgram::PROGRAM_ID.as_ref(),
                self.mint.seed(),
            ]
        }
    }

    impl SeedsToIdl for AssociatedTokenSeeds {
        fn seeds_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlSeeds> {
            Ok(IdlSeeds(vec![
                IdlSeed::Variable {
                    name: "wallet".to_string(),
                    description: vec![],
                    ty: <Pubkey as TypeToIdl>::type_to_idl(idl_definition)?,
                },
                IdlSeed::Const(TokenProgram::PROGRAM_ID.as_ref().to_vec()),
                IdlSeed::Variable {
                    name: "mint".to_string(),
                    description: vec![],
                    ty: <Pubkey as TypeToIdl>::type_to_idl(idl_definition)?,
                },
            ]))
        }
    }

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

    #[derive(Debug, Clone)]
    pub struct FindAssociatedTokenSeeds {
        pub wallet: FindSeed<Pubkey>,
        pub mint: FindSeed<Pubkey>,
    }
    impl FindIdlSeeds for FindAssociatedTokenSeeds {
        fn find_seeds(&self) -> Result<Vec<IdlFindSeed>> {
            Ok(vec![
                Into::into(&self.wallet),
                IdlFindSeed::Const(TokenProgram::PROGRAM_ID.as_ref().to_vec()),
                Into::into(&self.mint),
            ])
        }
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use idl_impl::*;

#[derive(Copy, Debug, Clone, PartialEq, Eq, InstructionSet)]
#[ix_set(use_repr)]
#[repr(u8)]
pub enum AssociatedTokenInstructionSet {
    Create(Create),
    CreateIdempotent(CreateIdempotent),
    RecoverNested(RecoverNested),
}

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

// create idempotent
/// See [`spl_associated_token_account::instruction::AssociatedTokenAccountInstruction::CreateIdempotent`].
///
/// This instruction has an identical AccountSet to [`Create`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = AssociatedTokenProgram)]
pub struct CreateIdempotent;
empty_star_frame_instruction!(CreateIdempotent, CreateAccounts);

// recover nested
/// See [`spl_associated_token_account::instruction::AssociatedTokenAccountInstruction::RecoverNested`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize)]
#[instruction_to_idl(program = AssociatedTokenProgram)]
pub struct RecoverNested;
/// Accounts for the [`RecoverNested`] instruction.
#[derive(Debug, Clone, AccountSet)]
pub struct RecoverNestedAccounts<'info> {
    #[idl(arg =
        Seeds(FindAtaSeeds {
            wallet: seed_path("owner_ata"),
            mint: seed_path("nested_mint"),
        })
    )]
    pub nested_ata: Mut<AccountInfo<'info>>,
    pub nested_mint: AccountInfo<'info>,
    #[idl(arg =
        Seeds(FindAtaSeeds {
            wallet: seed_path("wallet"),
            mint: seed_path("nested_mint"),
        })
    )]
    pub destination_ata: Mut<AccountInfo<'info>>,
    #[idl(arg =
        Seeds(FindAtaSeeds {
            wallet: seed_path("wallet"),
            mint: seed_path("owner_mint"),
        })
    )]
    pub owner_ata: Mut<AccountInfo<'info>>,
    pub owner_mint: AccountInfo<'info>,
    pub wallet: Mut<Signer<AccountInfo<'info>>>,
    pub token_program: Program<'info, TokenProgram>,
}
empty_star_frame_instruction!(RecoverNested, RecoverNestedAccounts);

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "idl")]
    #[test]
    fn print_token_idl() -> Result<()> {
        let idl = AssociatedTokenProgram::program_to_idl()?;
        println!("{}", star_frame::serde_json::to_string_pretty(&idl)?);
        Ok(())
    }
}
