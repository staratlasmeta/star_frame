use crate::token::{state::TokenAccount, Token};
use borsh::{BorshDeserialize, BorshSerialize};
use star_frame::derive_more;
use star_frame::empty_star_frame_instruction;
use star_frame::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct AssociatedToken;

impl AssociatedToken {
    /// Find the associated token address for the given wallet and mint.
    ///
    /// See [`spl_associated_token_account::get_associated_token_address`].
    /// ```
    /// # use star_frame_spl::associated_token::AssociatedToken;
    /// # use spl_associated_token_account::get_associated_token_address;
    /// # use pretty_assertions::assert_eq;
    /// # use star_frame::prelude::Pubkey;
    /// let wallet = Pubkey::new_unique();
    /// let mint = Pubkey::new_unique();
    /// assert_eq!(
    ///     AssociatedToken::find_address(&wallet, &mint),
    ///     get_associated_token_address(&wallet, &mint),
    /// );
    /// ```
    pub fn find_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(
            &[wallet.as_ref(), Token::ID.as_ref(), mint.as_ref()],
            &Self::ID,
        )
        .0
    }
}

impl StarFrameProgram for AssociatedToken {
    type InstructionSet = instructions::AssociatedTokenInstructionSet;
    type AccountDiscriminant = ();
    /// See [`spl_associated_token_account::ID`].
    /// ```
    /// # use star_frame::program::StarFrameProgram;
    /// # use star_frame_spl::associated_token::AssociatedToken;
    /// assert_eq!(AssociatedToken::ID, spl_associated_token_account::ID);
    /// ```
    const ID: Pubkey = pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::idl::{FindIdlSeeds, FindSeed, SeedsToIdl};
    use star_frame::star_frame_idl::seeds::{IdlFindSeed, IdlSeed, IdlSeeds};

    use crate::token::Token;
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
            vec![self.wallet.seed(), Token::ID.as_ref(), self.mint.seed()]
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
                IdlSeed::Const(Token::ID.as_ref().to_vec()),
                IdlSeed::Variable {
                    name: "mint".to_string(),
                    description: vec![],
                    ty: <Pubkey as TypeToIdl>::type_to_idl(idl_definition)?,
                },
            ]))
        }
    }

    impl ProgramToIdl for AssociatedToken {
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
                IdlFindSeed::Const(Token::ID.as_ref().to_vec()),
                Into::into(&self.mint),
            ])
        }
    }
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
pub use idl_impl::*;
use star_frame::anyhow::{bail, Context};
use star_frame::derive_more::{Deref, DerefMut};

pub mod instructions {
    pub use super::*;

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
    #[derive(
        Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize,
    )]
    #[instruction_to_idl(program = AssociatedToken)]
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
        pub system_program: Program<'info, System>,
        pub token_program: Program<'info, Token>,
    }
    empty_star_frame_instruction!(Create, CreateAccounts);

    // create idempotent
    /// See [`spl_associated_token_account::instruction::AssociatedTokenAccountInstruction::CreateIdempotent`].
    ///
    /// This instruction has an identical AccountSet to [`Create`].
    #[derive(
        Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize,
    )]
    #[instruction_to_idl(program = AssociatedToken)]
    pub struct CreateIdempotent;
    empty_star_frame_instruction!(CreateIdempotent, CreateAccounts);

    // recover nested
    /// See [`spl_associated_token_account::instruction::AssociatedTokenAccountInstruction::RecoverNested`].
    #[derive(
        Copy, Clone, Debug, Eq, PartialEq, InstructionToIdl, BorshDeserialize, BorshSerialize,
    )]
    #[instruction_to_idl(program = AssociatedToken)]
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
        pub token_program: Program<'info, Token>,
    }
    empty_star_frame_instruction!(RecoverNested, RecoverNestedAccounts);
}

pub mod state {
    use super::*;

    #[derive(AccountSet, Debug, Clone, Deref, DerefMut)]
    #[validate(
        id = "validate_ata",
        arg = ValidateAta<'a>,
        generics = [<'a>],
        extra_validation = self.validate_ata(arg)
    )]
    pub struct AssociatedTokenAccount<'info>(
        #[single_account_set(skip_can_init_account, skip_can_init_seeds)]
        pub(crate)  TokenAccount<'info>,
    );

    // TODO: should AssociatedTokenAccount's inner type be TokenAccount or AssociatedTokenAccount?
    // Having both is sorta okay, but if for example the account set was Option<AssociatedTokenAccount>, optional_key_for would
    // only return TokenAccount since thats the inner type right now.
    impl GetKeyFor<AssociatedTokenAccount<'static>> for AssociatedTokenAccount<'_> {
        fn key_for(&self) -> KeyFor<AssociatedTokenAccount<'static>> {
            KeyFor::new(*self.key())
        }
    }

    impl GetOptionalKeyFor<AssociatedTokenAccount<'static>> for AssociatedTokenAccount<'_> {
        fn optional_key_for(&self) -> OptionalKeyFor<AssociatedTokenAccount<'static>> {
            self.key_for().into()
        }
    }

    impl AssociatedTokenAccount<'_> {
        /// Validates that the given account is an associated token account.
        pub fn validate_ata(&self, validate_ata: ValidateAta) -> Result<()> {
            let expected_address =
                AssociatedToken::find_address(validate_ata.wallet, validate_ata.mint);
            if self.key() != &expected_address {
                bail!(
                    "Account {} is not the associated token account for wallet {} and mint {}",
                    self.key(),
                    validate_ata.wallet,
                    validate_ata.mint
                );
            }
            Ok(())
        }
    }

    impl<'info, A> CanInitSeeds<'info, A> for AssociatedTokenAccount<'info>
    where
        Self: AccountSetValidate<'info, A>,
    {
        fn init_seeds(&mut self, _arg: &A, _syscalls: &impl SyscallInvoke<'info>) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    pub struct ValidateAta<'a> {
        pub wallet: &'a Pubkey,
        pub mint: &'a Pubkey,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct InitAta<'a, 'info, WalletInfo, MintInfo> {
        pub wallet: &'a WalletInfo,
        pub mint: &'a MintInfo,
        pub system_program: &'a Program<'info, System>,
        pub token_program: &'a Program<'info, Token>,
    }

    impl<'a, 'info, WalletInfo, MintInfo> InitAta<'a, 'info, WalletInfo, MintInfo> {
        pub fn new(
            wallet: &'a WalletInfo,
            mint: &'a MintInfo,
            system_program: &'a Program<'info, System>,
            token_program: &'a Program<'info, Token>,
        ) -> Self {
            Self {
                wallet,
                mint,
                system_program,
                token_program,
            }
        }
    }

    impl<'a, 'info, WalletInfo, MintInfo> From<InitAta<'a, 'info, WalletInfo, MintInfo>>
        for ValidateAta<'a>
    where
        WalletInfo: SingleAccountSet<'info>,
        MintInfo: SingleAccountSet<'info>,
        'info: 'a,
    {
        fn from(value: InitAta<'a, 'info, WalletInfo, MintInfo>) -> Self {
            Self {
                mint: value.mint.key(),
                wallet: value.wallet.key(),
            }
        }
    }

    impl<'info, 'a, WalletInfo, MintInfo>
        CanInitAccount<'info, InitAta<'a, 'info, WalletInfo, MintInfo>>
        for AssociatedTokenAccount<'info>
    where
        WalletInfo: SingleAccountSet<'info>,
        MintInfo: SingleAccountSet<'info>,
    {
        fn init_account<const IF_NEEDED: bool>(
            &mut self,
            arg: InitAta<'a, 'info, WalletInfo, MintInfo>,
            account_seeds: Option<Vec<&[u8]>>,
            syscalls: &impl SyscallInvoke<'info>,
        ) -> Result<()> {
            let funder = syscalls
                .get_funder()
                .context("Missing tagged `funder` for AssociatedTokenAccount `init_account`")?;
            self.init_account::<IF_NEEDED>((arg, funder), account_seeds, syscalls)
        }
    }

    impl<'info, 'a, WalletInfo, MintInfo, Funder>
        CanInitAccount<'info, (InitAta<'a, 'info, WalletInfo, MintInfo>, &Funder)>
        for AssociatedTokenAccount<'info>
    where
        Funder: SignedAccount<'info> + WritableAccount<'info>,
        WalletInfo: SingleAccountSet<'info>,
        MintInfo: SingleAccountSet<'info>,
    {
        fn init_account<const IF_NEEDED: bool>(
            &mut self,
            arg: (InitAta<'a, 'info, WalletInfo, MintInfo>, &Funder),
            account_seeds: Option<Vec<&[u8]>>,
            syscalls: &impl SyscallInvoke<'info>,
        ) -> Result<()> {
            if IF_NEEDED && self.owner() == &Token::ID {
                self.validate()?;
                self.validate_ata(arg.0.into())?;
                return Ok(());
            }
            if account_seeds.is_some() {
                bail!("Account seeds are not supported for Init<AssociatedTokenAccount>");
            }
            self.check_writable()?;
            let (init_ata, funder) = arg;
            let funder_seeds = funder.signer_seeds();
            let seeds: &[&[&[u8]]] = match &funder_seeds {
                Some(seeds) => &[seeds],
                None => &[],
            };

            AssociatedToken::cpi(
                &instructions::Create,
                instructions::CreateCpiAccounts {
                    funder: funder.account_info_cloned(),
                    token_account: self.account_info_cloned(),
                    wallet: init_ata.wallet.account_info_cloned(),
                    mint: init_ata.mint.account_info_cloned(),
                    system_program: init_ata.system_program.account_info_cloned(),
                    token_program: init_ata.token_program.account_info_cloned(),
                },
            )?
            .invoke_signed(seeds, syscalls)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "idl")]
    #[test]
    fn print_token_idl() -> Result<()> {
        let idl = AssociatedToken::program_to_idl()?;
        println!("{}", star_frame::serde_json::to_string_pretty(&idl)?);
        Ok(())
    }
}
