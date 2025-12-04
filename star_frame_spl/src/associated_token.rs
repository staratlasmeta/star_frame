use crate::token::{
    state::{MintAccount, TokenAccount},
    Token,
};
use borsh::{BorshDeserialize, BorshSerialize};
use star_frame::{derive_more, empty_star_frame_instruction, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct AssociatedToken;

impl AssociatedToken {
    /// Find the associated token address for the given wallet and mint.
    ///
    /// See [`spl_associated_token_account_interface::address::get_associated_token_address`].
    /// ```
    /// # use star_frame_spl::{token::state::MintAccount,associated_token::AssociatedToken};
    /// # use spl_associated_token_account_interface::address::get_associated_token_address;
    /// # use pretty_assertions::assert_eq;
    /// # use star_frame::prelude::{KeyFor, Address};
    /// let wallet = Address::new_unique();
    /// let mint = KeyFor::<MintAccount>::new(Address::new_unique());
    /// assert_eq!(
    ///     AssociatedToken::find_address(&wallet, &mint),
    ///     get_associated_token_address(&wallet, &mint.address()),
    /// );
    /// ```
    pub fn find_address(wallet: &Address, mint: &KeyFor<MintAccount>) -> Address {
        Self::find_address_with_bump(wallet, mint).0
    }

    /// Find the associated token address for the given wallet and mint, with a bump.
    pub fn find_address_with_bump(wallet: &Address, mint: &KeyFor<MintAccount>) -> (Address, u8) {
        Address::find_program_address(
            &[wallet.as_ref(), Token::ID.as_ref(), mint.address().as_ref()],
            &Self::ID,
        )
    }
}

impl StarFrameProgram for AssociatedToken {
    type InstructionSet = instructions::AssociatedTokenInstructionSet;
    type AccountDiscriminant = ();
    /// See [`spl_associated_token_account_interface::program::ID`].
    /// ```
    /// # use star_frame::program::StarFrameProgram;
    /// # use star_frame_spl::associated_token::AssociatedToken;
    /// assert_eq!(AssociatedToken::ID, spl_associated_token_account_interface::program::ID);
    /// ```
    const ID: Address = address!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
}

#[cfg(all(feature = "idl", not(target_os = "solana")))]
mod idl_impl {
    use super::*;
    use star_frame::{
        idl::{FindIdlSeeds, FindSeed, SeedsToIdl},
        star_frame_idl::seeds::{IdlFindSeed, IdlSeed, IdlSeeds},
    };

    use crate::token::{state::MintAccount, Token};
    use star_frame::star_frame_idl::IdlDefinition;

    // todo: potentially support multiple token programs here
    #[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
    pub struct AssociatedTokenSeeds {
        pub wallet: Address,
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
        fn seeds_to_idl(idl_definition: &mut IdlDefinition) -> star_frame::IdlResult<IdlSeeds> {
            Ok(IdlSeeds(vec![
                IdlSeed::Variable {
                    name: "wallet".to_string(),
                    description: vec![],
                    ty: <Address as TypeToIdl>::type_to_idl(idl_definition)?,
                },
                IdlSeed::Const(Token::ID.as_ref().to_vec()),
                IdlSeed::Variable {
                    name: "mint".to_string(),
                    description: vec![],
                    ty: <Address as TypeToIdl>::type_to_idl(idl_definition)?,
                },
            ]))
        }
    }

    impl ProgramToIdl for AssociatedToken {
        type Errors = ();
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
        pub wallet: FindSeed<Address>,
        pub mint: FindSeed<Address>,
    }
    impl FindIdlSeeds for FindAssociatedTokenSeeds {
        fn find_seeds(&self) -> star_frame::IdlResult<Vec<IdlFindSeed>> {
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
    /// See [`spl_associated_token_account_interface::instruction::AssociatedTokenAccountInstruction::Create`].
    #[derive(
        Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize,
    )]
    #[type_to_idl(program = AssociatedToken)]
    pub struct Create;
    /// Accounts for the [`Create`] instruction.
    #[derive(Debug, Clone, AccountSet)]
    pub struct CreateAccounts {
        pub funder: Mut<Signer>,
        #[idl(arg =
            Seeds(FindAtaSeeds {
                wallet: seed_path("wallet"),
                mint: seed_path("mint"),
            })
        )]
        pub token_account: Mut<AccountView>,
        pub wallet: AccountView,
        pub mint: AccountView,
        pub system_program: Program<System>,
        pub token_program: Program<Token>,
    }
    empty_star_frame_instruction!(Create, CreateAccounts);

    // create idempotent
    /// See [`spl_associated_token_account_interface::instruction::AssociatedTokenAccountInstruction::CreateIdempotent`].
    ///
    /// This instruction has an identical AccountSet to [`Create`].
    #[derive(
        Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize,
    )]
    #[type_to_idl(program = AssociatedToken)]
    pub struct CreateIdempotent;
    empty_star_frame_instruction!(CreateIdempotent, CreateAccounts);

    // recover nested
    /// See [`spl_associated_token_account_interface::instruction::AssociatedTokenAccountInstruction::RecoverNested`].
    #[derive(
        Copy, Clone, Debug, Eq, PartialEq, InstructionArgs, BorshDeserialize, BorshSerialize,
    )]
    #[type_to_idl(program = AssociatedToken)]
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
        pub nested_ata: Mut<AccountView>,
        pub nested_mint: AccountView,
        #[idl(arg =
            Seeds(FindAtaSeeds {
                wallet: seed_path("wallet"),
                mint: seed_path("nested_mint"),
            })
        )]
        pub destination_ata: Mut<AccountView>,
        #[idl(arg =
            Seeds(FindAtaSeeds {
                wallet: seed_path("wallet"),
                mint: seed_path("owner_mint"),
            })
        )]
        pub owner_ata: Mut<AccountView>,
        pub owner_mint: AccountView,
        pub wallet: Mut<Signer>,
        pub token_program: Program<Token>,
    }
    empty_star_frame_instruction!(RecoverNested, RecoverNestedAccounts);
}

pub mod state {
    use star_frame::{
        account_set::{
            modifiers::{CanInitAccount, CanInitSeeds},
            AccountSetValidate, CanFundRent,
        },
        data_types::{GetKeyFor, GetOptionalKeyFor},
        errors::ErrorCode,
    };

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
            KeyFor::new_ref(self.address())
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
            if self.address() != &expected_address {
                bail!(
                    ErrorCode::AddressMismatch,
                    "Account {} is not the associated token account for wallet {} and mint {}",
                    self.address(),
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
        pub wallet: &'a Address,
        pub mint: &'a KeyFor<MintAccount>,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct InitAta<'a, WalletInfo, MintInfo>
    where
        WalletInfo: SingleAccountSet,
        MintInfo: SingleAccountSet,
    {
        pub wallet: &'a WalletInfo,
        pub mint: &'a MintInfo,
        pub system_program: Program<System>,
        pub token_program: Program<Token>,
    }

    impl<'a, WalletInfo, MintInfo> InitAta<'a, WalletInfo, MintInfo>
    where
        WalletInfo: SingleAccountSet,
        MintInfo: SingleAccountSet,
    {
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
                mint: KeyFor::new_ref(value.mint.address()),
                wallet: value.wallet.address(),
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
            account_seeds: Option<&[&[u8]]>,
            ctx: &Context,
        ) -> Result<()> {
            let funder = ctx.get_funder().ok_or_else(|| {
                error!(
                    ErrorCode::EmptyFunderCache,
                    "Missing tagged `funder` for AssociatedTokenAccount `init_account`"
                )
            })?;
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
            account_seeds: Option<&[&[u8]]>,
            ctx: &Context,
        ) -> Result<()> {
            // SAFETY:
            // The reference is immediately used and dropped, so we don't need to worry about it being used after the function returns
            if IF_NEEDED && unsafe { self.account_info().owner() }.fast_eq(&Token::ID) {
                self.validate()?;
                self.validate_ata(init_ata.into())?;
                return Ok(());
            }
            if !funder.can_create_account() {
                let current_lamports = self.account_info().lamports();
                let required_rent = ctx
                    .get_rent()?
                    .minimum_balance(TokenAccount::LEN)
                    .saturating_sub(current_lamports);
                if required_rent > 0 {
                    // Funding rent prior to the CPI call results in assoicated token instruction ignoring funder, so we don't error out.
                    funder.fund_rent(self, required_rent, ctx)?;
                }
            }
            if account_seeds.is_some() {
                bail!(
                    ProgramError::InvalidSeeds,
                    "Account seeds are not supported for Init<AssociatedTokenAccount>"
                );
            }
            self.check_writable()?;
            let funder_seeds = funder.signer_seeds();
            let seeds: &[&[&[u8]]] = match &funder_seeds {
                Some(seeds) => &[seeds],
                None => &[],
            };

            AssociatedToken::cpi(
                instructions::Create,
                instructions::CreateCpiAccounts {
                    funder: funder.account_to_modify(),
                    token_account: *self.account_info(),
                    wallet: *init_ata.wallet.account_info(),
                    mint: *init_ata.mint.account_info(),
                    system_program: *init_ata.system_program.account_info(),
                    token_program: *init_ata.token_program.account_info(),
                },
                None,
            )
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
        std::println!("{}", star_frame::serde_json::to_string_pretty(&idl)?);
        Ok(())
    }
}
