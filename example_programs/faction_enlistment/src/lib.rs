use counter::CounterAccountData;
use star_frame::account_set::Account;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;
// use star_frame_spl::{
//     associated_token::{
//         state::{AssociatedTokenAccount, InitAta},
//         AssociatedToken,
//     },
//     token::{
//         state::{InitMint, MintAccount},
//         Token,
//     },
// };

#[unsized_type]
pub struct SomeFields {
    sized1: u8,
    sized2: u8,
    #[unsized_start]
    unsized1: SomeUnsized,
    unsized2: List<u8>,
}

#[unsized_type]
pub struct SomeUnsized {
    #[unsized_start]
    unsized1: List<u8>,
    unsized2: List<u8>,
}

#[unsized_impl(tag = "1")]
impl SomeFields {
    #[exclusive]
    fn foo(&mut self) -> Result<()> {
        self.sized1 = 10;
        self.unsized2().push(5)?;
        Ok(())
    }
}

#[cfg(feature = "prod")]
const CFG_ID: Pubkey = pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc");

#[cfg(not(feature = "prod"))]
const CFG_ID: Pubkey = pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc");

#[derive(StarFrameProgram)]
#[program(
    instruction_set = FactionEnlistmentInstructionSet,
    id = CFG_ID
)]
pub struct FactionEnlistment;

#[derive(InstructionSet)]
pub enum FactionEnlistmentInstructionSet {
    ProcessEnlistPlayer(ProcessEnlistPlayerIx),
}

/// ProcessEnlistPlayerIx
#[derive(Clone, BorshDeserialize, BorshSerialize, Default, InstructionArgs)]
#[borsh(crate = "borsh")]
#[repr(C)]
#[instruction_to_idl(program = FactionEnlistment)]
pub struct ProcessEnlistPlayerIx {
    /// The bump for PDA seeds
    #[ix_args(run, decode)] // not used, but just to make sure this works
    bump: u8,
    /// New faction id for the player
    /// Some more docs
    #[ix_args(run)]
    faction_id: FactionId,
    // buncha_data: Vec<u8>,
}

impl StarFrameInstruction for ProcessEnlistPlayerIx {
    type ReturnType = ();
    type Accounts<'b, 'c> = ProcessEnlistPlayer;

    fn run_instruction(
        account_set: &mut Self::Accounts<'_, '_>,
        (_bump, faction_id): Self::RunArg<'_>,
        ctx: &mut impl Context,
    ) -> Result<Self::ReturnType> {
        let clock = ctx.get_clock()?;
        let bump = account_set.player_faction_account.access_seeds().bump;
        let mut player_faction_data = account_set.player_faction_account.data_mut()?;
        **player_faction_data = PlayerFactionDataSized {
            owner: *account_set.player_account.pubkey(),
            enlisted_at_timestamp: clock.unix_timestamp,
            faction_id,
            counter: Default::default(),
            bump,
            _padding: [0; 5],
        };
        // player_faction_data.set_v1(PlayerFactionDataInit {
        //     sized: PlayerFactionDataSized {
        //         owner: *account_set.player_account.key,
        //         enlisted_at_timestamp: clock.unix_timestamp,
        //         faction_id,
        //         counter: Default::default(),
        //         bump,
        //         _padding: [0; 5],
        //     },
        //     some_fields: DefaultInit,
        // })?;
        // let PlayerFactionDataAccountExclusive::V1(data) = &mut player_faction_data.get() else {
        //     bail!("Invalid player_faction_data state");
        // };
        player_faction_data.some_fields().foo()?;
        Ok(())
    }
}

#[derive(AccountSet)]
#[decode(arg = u8)]
pub struct ProcessEnlistPlayer {
    /// The player faction account
    #[validate(arg = (Create(()),
        Seeds(PlayerFactionAccountSeeds {
        player_account: *self.player_account.pubkey()
    })), requires = [player_account])] // the funder is set during validate, so we need the player account to be run first.
    #[idl(
        arg = Seeds(FindPlayerFactionAccountSeeds {
            player_account: seed_path("player_account")
        })
    )]
    #[cleanup(arg = NormalizeRent(()))]
    pub player_faction_account: Init<Seeded<Account<PlayerFactionData>>>,
    /// The player account
    #[account_set(funder)]
    pub player_account: Mut<Signer<SystemAccount>>,
    /// Solana System program
    pub system_program: Program<System>,
    // pub token_program: Program<Token>,
    // pub associated_token_program: Program<AssociatedToken>,
    // #[validate(arg = Create(InitMint {
    //     decimals: 0,
    //     mint_authority: self.player_account.pubkey(),
    //     freeze_authority: Some(self.player_account.pubkey()),
    // }))]
    // pub mint: Init<Signer<MintAccount>>,
    // #[validate(arg = Create(InitAta {
    //     wallet: &self.player_account,
    //     mint: &self.mint,
    //     system_program: self.system_program,
    //     token_program: self.token_program
    // }))]
    // pub token_account: Init<AssociatedTokenAccount>,
}

// #[derive(
//     ProgramAccount, Debug, Align1, Copy, Clone, CheckedBitPattern, NoUninit, Eq, PartialEq, Zeroable,
// )]
// #[repr(C, packed)]
// #[program_account(seeds = PlayerFactionAccountSeeds)]
#[unsized_type(program_account, seeds = PlayerFactionAccountSeeds)]
pub struct PlayerFactionData {
    pub owner: Pubkey,
    pub enlisted_at_timestamp: i64,
    pub faction_id: FactionId,
    pub bump: u8,
    pub counter: CounterAccountData,
    pub _padding: [u64; 5],
    #[unsized_start]
    some_fields: SomeFields,
}

// #[unsized_type(program_account, seeds = PlayerFactionAccountSeeds)]
// #[repr(u8)]
// pub enum PlayerFactionDataAccount {
//     V1(PlayerFactionData),
//     #[default_init]
//     V2(PlayerFactionData),
// }

#[derive(
    Debug,
    Copy,
    Clone,
    CheckedBitPattern,
    NoUninit,
    BorshDeserialize,
    BorshSerialize,
    Eq,
    PartialEq,
    Default,
    TypeToIdl,
)]
#[borsh(crate = "borsh", use_discriminant = true)]
#[repr(u8)]
pub enum FactionId {
    #[default]
    MUD,
    ONI,
    Ustur,
}
unsafe impl Zeroable for FactionId {}

#[derive(Debug, GetSeeds, Clone)]
#[get_seeds(seed_const = b"FACTION_ENLISTMENT")]
pub struct PlayerFactionAccountSeeds {
    player_account: Pubkey,
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::{collections::HashMap, env};

    use super::*;
    use mollusk_svm::{program::keyed_account_for_system_program, result::Check, Mollusk};
    use pretty_assertions::assert_eq;
    use solana_account::Account as SolanaAccount;
    use star_frame::client::{DeserializeAccount, SerializeAccount};
    // use star_frame_spl::token::Token;

    #[cfg(feature = "idl")]
    #[test]
    fn idl() {
        let idl: star_frame::star_frame_idl::ProgramNode = FactionEnlistment::program_to_idl()
            .unwrap()
            .try_into()
            .unwrap();
        let idl_json = star_frame::serde_json::to_string_pretty(&idl).unwrap();
        println!("{idl_json}",);
        std::fs::write("idl.json", &idl_json).unwrap();
    }

    #[test]
    fn test_ix() -> Result<()> {
        if env::var("SBF_OUT_DIR").is_err() {
            println!("SBF_OUT_DIR is not set, skipping test");
            return Ok(());
        }
        let mut mollusk = Mollusk::new(&FactionEnlistment::ID, "faction_enlistment");
        mollusk_svm_programs_token::token::add_program(&mut mollusk);
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

        let faction_id = FactionId::MUD;
        const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

        let player_account = Pubkey::new_unique();
        let (player_faction_account, bump) =
            PlayerFactionData::find_program_address(&PlayerFactionAccountSeeds { player_account });

        // let mint = Pubkey::new_unique();
        // let (token_account, bump) = AssociatedToken::find_address_with_bump(&player_account, &mint);

        let mut account_store: HashMap<Pubkey, SolanaAccount> = HashMap::from_iter([
            (
                player_account,
                SolanaAccount::new(LAMPORTS_PER_SOL, 0, &System::ID),
            ),
            (player_faction_account, SolanaAccount::default()),
            keyed_account_for_system_program(),
            // (token_account, SolanaAccount::default()),
            // (mint, SolanaAccount::default()),
            // mollusk_svm_programs_token::token::keyed_account(),
            // mollusk_svm_programs_token::associated_token::keyed_account(),
        ]);
        let mollusk = mollusk.with_context(account_store);

        let ix = FactionEnlistment::instruction(
            &ProcessEnlistPlayerIx { bump, faction_id },
            ProcessEnlistPlayerClientAccounts {
                player_faction_account,
                player_account,
                system_program: System::ID,
                // token_program: Token::ID,
                // associated_token_program: AssociatedToken::ID,
                // mint,
                // token_account,
            },
        )?;

        let clock_timestamp = mollusk.mollusk.sysvars.clock.unix_timestamp;
        let expected_faction_account = PlayerFactionDataOwned {
            owner: player_account,
            enlisted_at_timestamp: clock_timestamp,
            faction_id,
            counter: Default::default(),
            bump,
            _padding: [0; 5],
            some_fields: SomeFieldsOwned {
                sized1: 10,
                sized2: 0,
                unsized1: SomeUnsizedOwned {
                    unsized1: vec![],
                    unsized2: vec![],
                },
                unsized2: vec![5],
            },
        };
        let res = mollusk.process_and_validate_instruction(
            &ix,
            &[
                Check::success(),
                Check::account(&player_faction_account)
                    .data(&PlayerFactionData::serialize_account(
                        expected_faction_account,
                    )?)
                    .owner(&FactionEnlistment::ID)
                    .build(),
            ],
        );

        Ok(())
    }
}
