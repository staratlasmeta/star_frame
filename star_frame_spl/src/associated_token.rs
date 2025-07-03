use crate::token::state::MintAccount;
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
    /// # use star_frame_spl::{token::state::MintAccount,associated_token::AssociatedToken};
    /// # use spl_associated_token_account::get_associated_token_address;
    /// # use pretty_assertions::assert_eq;
    /// # use star_frame::prelude::{KeyFor, Pubkey};
    /// let wallet = Pubkey::new_unique();
    /// let mint = KeyFor::<MintAccount>::new(Pubkey::new_unique());
    /// assert_eq!(
    ///     AssociatedToken::find_address(&wallet, &mint),
    ///     get_associated_token_address(&wallet, &mint.pubkey()),
    /// );
    /// ```
    pub fn find_address(wallet: &Pubkey, mint: &KeyFor<MintAccount>) -> Pubkey {
        Self::find_address_with_bump(wallet, mint).0
    }

    /// Find the associated token address for the given wallet and mint, with a bump.
    pub fn find_address_with_bump(wallet: &Pubkey, mint: &KeyFor<MintAccount>) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[wallet.as_ref(), Token::ID.as_ref(), mint.pubkey().as_ref()],
            &Self::ID,
        )
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

    use crate::token::state::MintAccount;
    use crate::token::Token;
    use star_frame::star_frame_idl::IdlDefinition;

    // todo: potentially support multiple token programs here
    #[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
    pub struct AssociatedTokenSeeds {
        pub wallet: Pubkey,
        pub mint: KeyFor<MintAccount>,
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
use star_frame::anyhow::{bail, Context as _};
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
    pub struct CreateAccounts {
        pub funder: Mut<Signer<AccountInfo>>,
        #[idl(arg =
            Seeds(FindAtaSeeds {
                wallet: seed_path("wallet"),
                mint: seed_path("mint"),
            })
        )]
        pub token_account: Mut<AccountInfo>,
        pub wallet: AccountInfo,
        pub mint: AccountInfo,
        pub system_program: Program<System>,
        pub token_program: Program<Token>,
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
    pub struct RecoverNestedAccounts {
        #[idl(arg =
            Seeds(FindAtaSeeds {
                wallet: seed_path("owner_ata"),
                mint: seed_path("nested_mint"),
            })
        )]
        pub nested_ata: Mut<AccountInfo>,
        pub nested_mint: AccountInfo,
        #[idl(arg =
            Seeds(FindAtaSeeds {
                wallet: seed_path("wallet"),
                mint: seed_path("nested_mint"),
            })
        )]
        pub destination_ata: Mut<AccountInfo>,
        #[idl(arg =
            Seeds(FindAtaSeeds {
                wallet: seed_path("wallet"),
                mint: seed_path("owner_mint"),
            })
        )]
        pub owner_ata: Mut<AccountInfo>,
        pub owner_mint: AccountInfo,
        pub wallet: Mut<Signer<AccountInfo>>,
        pub token_program: Program<Token>,
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
    pub struct AssociatedTokenAccount(
        #[single_account_set(skip_can_init_account, skip_can_init_seeds)] pub(crate) TokenAccount,
    );

    // TODO: should AssociatedTokenAccount's inner type be TokenAccount or AssociatedTokenAccount?
    // Having both is sorta okay, but if for example the account set was Option<AssociatedTokenAccount>, optional_key_for would
    // only return TokenAccount since thats the inner type right now.
    impl GetKeyFor<AssociatedTokenAccount> for AssociatedTokenAccount {
        fn key_for(&self) -> &KeyFor<AssociatedTokenAccount> {
            KeyFor::new_ref(self.pubkey())
        }
    }

    impl GetOptionalKeyFor<AssociatedTokenAccount> for AssociatedTokenAccount {
        fn optional_key_for(&self) -> &OptionalKeyFor<AssociatedTokenAccount> {
            self.key_for().into()
        }
    }

    impl AssociatedTokenAccount {
        /// Validates that the given account is an associated token account.
        pub fn validate_ata(&self, validate_ata: ValidateAta) -> Result<()> {
            let expected_address =
                AssociatedToken::find_address(validate_ata.wallet, validate_ata.mint);
            if self.pubkey() != &expected_address {
                bail!(
                    "Account {} is not the associated token account for wallet {} and mint {}",
                    self.pubkey(),
                    validate_ata.wallet,
                    validate_ata.mint
                );
            }
            Ok(())
        }
    }

    impl<A> CanInitSeeds<A> for AssociatedTokenAccount
    where
        Self: AccountSetValidate<A>,
    {
        fn init_seeds(&mut self, _arg: &A, _ctx: &Context) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    pub struct ValidateAta<'a> {
        pub wallet: &'a Pubkey,
        pub mint: &'a KeyFor<MintAccount>,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct InitAta<'a, WalletInfo, MintInfo> {
        pub wallet: &'a WalletInfo,
        pub mint: &'a MintInfo,
        pub system_program: Program<System>,
        pub token_program: Program<Token>,
    }

    impl<'a, WalletInfo, MintInfo> InitAta<'a, WalletInfo, MintInfo> {
        pub fn new(
            wallet: &'a WalletInfo,
            mint: &'a MintInfo,
            system_program: Program<System>,
            token_program: Program<Token>,
        ) -> Self {
            Self {
                wallet,
                mint,
                system_program,
                token_program,
            }
        }
    }

    impl<'a, WalletInfo, MintInfo> From<InitAta<'a, WalletInfo, MintInfo>> for ValidateAta<'a>
    where
        WalletInfo: SingleAccountSet,
        MintInfo: SingleAccountSet,
    {
        fn from(value: InitAta<'a, WalletInfo, MintInfo>) -> Self {
            Self {
                mint: KeyFor::new_ref(value.mint.pubkey()),
                wallet: value.wallet.pubkey(),
            }
        }
    }

    impl<'a, WalletInfo, MintInfo> CanInitAccount<InitAta<'a, WalletInfo, MintInfo>>
        for AssociatedTokenAccount
    where
        WalletInfo: SingleAccountSet,
        MintInfo: SingleAccountSet,
    {
        fn init_account<const IF_NEEDED: bool>(
            &mut self,
            arg: InitAta<'a, WalletInfo, MintInfo>,
            account_seeds: Option<Vec<&[u8]>>,
            ctx: &Context,
        ) -> Result<()> {
            let funder = ctx
                .get_funder()
                .context("Missing tagged `funder` for AssociatedTokenAccount `init_account`")?;
            self.init_account::<IF_NEEDED>((arg, funder), account_seeds, ctx)
        }
    }

    impl<'a, WalletInfo, MintInfo, Funder>
        CanInitAccount<(InitAta<'a, WalletInfo, MintInfo>, &Funder)> for AssociatedTokenAccount
    where
        WalletInfo: SingleAccountSet,
        MintInfo: SingleAccountSet,
        Funder: CanFundRent + ?Sized,
    {
        fn init_account<const IF_NEEDED: bool>(
            &mut self,
            (init_ata, funder): (InitAta<'a, WalletInfo, MintInfo>, &Funder),
            account_seeds: Option<Vec<&[u8]>>,
            ctx: &Context,
        ) -> Result<()> {
            if !funder.can_create_account() {
                bail!(
                    "Funder with key `{}` does not have the ability to create accounts. This is a logic bug in the program.",
                    funder.account_to_modify().pubkey(),
                );
            }
            if IF_NEEDED && self.owner_pubkey() == Token::ID {
                self.validate()?;
                self.validate_ata(init_ata.into())?;
                return Ok(());
            }
            if account_seeds.is_some() {
                bail!("Account seeds are not supported for Init<AssociatedTokenAccount>");
            }
            self.check_writable()?;
            let funder_seeds = funder.signer_seeds();
            let seeds: &[&[&[u8]]] = match &funder_seeds {
                Some(seeds) => &[seeds],
                None => &[],
            };

            AssociatedToken::cpi(
                &instructions::Create,
                instructions::CreateCpiAccounts {
                    funder: funder.account_to_modify(),
                    token_account: *self.account_info(),
                    wallet: *init_ata.wallet.account_info(),
                    mint: *init_ata.mint.account_info(),
                    system_program: *init_ata.system_program.account_info(),
                    token_program: *init_ata.token_program.account_info(),
                },
                ctx,
            )?
            .invoke_signed(seeds)?;
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
