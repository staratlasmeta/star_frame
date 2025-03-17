use counter::CounterAccountData;
use star_frame::account_set::Account;
// use star_frame::anyhow::bail;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;
// use star_frame_spl::{
//     associated_token::AssociatedToken,
//     associated_token::AssociatedTokenAccount,
//     associated_token::InitAta,
//     token::InitMint,
//     token::{MintAccount, Token},
// };

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
pub struct SomeFields {
    sized1: u8,
    sized2: u8,
    #[unsized_start]
    unsized1: SomeUnsized,
    unsized2: List<u8>,
}

#[unsized_type(owned_attributes = [derive(PartialEq, Eq, Clone)])]
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

#[derive(StarFrameProgram)]
#[program(
    instruction_set = FactionEnlistmentInstructionSet,
)]
#[cfg_attr(
    feature = "prod",
    program(id = "FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc")
)]
#[cfg_attr(
    not(feature = "prod"),
    program(id = "FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc")
)]
pub struct FactionEnlistment;

#[derive(InstructionSet)]
pub enum FactionEnlistmentInstructionSet {
    ProcessEnlistPlayer(ProcessEnlistPlayerIx),
}

/// ProcessEnlistPlayerIx
#[derive(Clone, BorshDeserialize, BorshSerialize, Default, InstructionToIdl)]
#[borsh(crate = "borsh")]
#[repr(C)]
#[instruction_to_idl(program = FactionEnlistment)]
pub struct ProcessEnlistPlayerIx {
    /// The bump for PDA seeds
    bump: u8,
    /// New faction id for the player
    /// Some more docs
    faction_id: FactionId,
    // buncha_data: Vec<u8>,
}

impl StarFrameInstruction for ProcessEnlistPlayerIx {
    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = FactionId;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = ProcessEnlistPlayer<'info>;

    fn split_to_args<'a>(r: &mut Self) -> IxArgs<Self> {
        IxArgs::run(r.faction_id)
    }

    fn run_instruction<'info>(
        account_set: &mut Self::Accounts<'_, '_, 'info>,
        faction_id: Self::RunArg<'_>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self::ReturnType> {
        let clock = syscalls.get_clock()?;
        let bump = account_set.player_faction_account.access_seeds().bump;
        let mut player_faction_account_data = account_set.player_faction_account.data_mut()?;
        let mut exclusive = player_faction_account_data.exclusive();
        **exclusive = PlayerFactionDataSized {
            owner: *account_set.player_account.key,
            enlisted_at_timestamp: clock.unix_timestamp,
            faction_id,
            counter: Default::default(),
            bump,
            _padding: [0; 5],
        };
        // exclusive.set_v1(PlayerFactionDataInit {
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
        // let PlayerFactionDataAccountExclusive::V1(data) = &mut exclusive.get() else {
        //     bail!("Invalid exclusive state");
        // };
        exclusive.some_fields().foo()?;
        Ok(())
    }
}

#[derive(AccountSet)]
pub struct ProcessEnlistPlayer<'info> {
    /// The player faction account
    #[validate(arg = (Create(()),
    Seeds(PlayerFactionAccountSeeds {
        player_account: *self.player_account.key()
    })))]
    #[idl(
        arg = Seeds(FindPlayerFactionAccountSeeds {
            player_account: seed_path("player_account")
        })
    )]
    #[cleanup(arg = NormalizeRent(()))]
    pub player_faction_account: Init<Seeded<Account<'info, PlayerFactionData>>>,
    /// The player account
    #[account_set(funder)]
    pub player_account: Mut<Signer<SystemAccount<'info>>>,
    /// Solana System program
    pub system_program: Program<'info, System>,
    // pub token_program: Program<'info, Token>,
    // pub associated_token_program: Program<'info, AssociatedToken>,
    // #[validate(arg = Create(InitMint {
    //     decimals: 0,
    //     mint_authority: self.player_account.key(),
    //     freeze_authority: Some(self.player_account.key()),
    // }))]
    // pub mint: Init<Signer<MintAccount<'info>>>,
    // #[validate(arg = Create(InitAta {
    //     wallet: &self.player_account,
    //     mint: &self.mint,
    //     system_program: &self.system_program,
    //     token_program: &self.token_program
    // }))]
    // pub token_account: Init<AssociatedTokenAccount<'info>>,
}

// #[derive(
//     ProgramAccount, Debug, Align1, Copy, Clone, CheckedBitPattern, NoUninit, Eq, PartialEq, Zeroable,
// )]
// #[repr(C, packed)]
// #[program_account(seeds = PlayerFactionAccountSeeds)]
#[unsized_type(program_account, seeds = PlayerFactionAccountSeeds, owned_attributes = [derive(PartialEq, Eq, Clone)]
)]
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

// #[unsized_type(program_account, seeds = PlayerFactionAccountSeeds, owned_attributes = [derive(PartialEq, Eq, Clone)])]
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
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::account::Account;
    use solana_sdk::clock::Clock;
    use solana_sdk::signature::{Keypair, Signer};
    use star_frame::client::{DeserializeAccount, SerializeAccount};
    use star_frame::solana_program::native_token::LAMPORTS_PER_SOL;
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

    #[tokio::test]
    async fn banks_test() -> Result<()> {
        let program_test = if option_env!("USE_BIN").is_some() {
            let target_dir = std::env::current_dir()?
                .join("../../target/deploy")
                .canonicalize()?;
            std::env::set_var(
                "BPF_OUT_DIR",
                target_dir.to_str().expect("Failed to convert path to str"),
            );
            ProgramTest::new("faction_enlistment", StarFrameDeclaredProgram::ID, None)
        } else {
            ProgramTest::new(
                "faction_enlistment",
                StarFrameDeclaredProgram::ID,
                processor!(FactionEnlistment::processor),
            )
        };

        let mut test_context = program_test.start_with_context().await;
        let (player_account, (faction_account, bump)) = loop {
            let key = Keypair::new();
            let seeds = PlayerFactionAccountSeeds {
                player_account: key.pubkey(),
            };
            let player_faction =
                Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::ID);
            if player_faction.1 == 255 {
                let data = Account {
                    lamports: LAMPORTS_PER_SOL * 100,
                    ..Default::default()
                };
                test_context.set_account(&key.pubkey(), &data.into());
                break (key, player_faction);
            }
        };
        let mut banks_client = test_context.banks_client;

        let faction_id = FactionId::MUD;

        // let mint_keypair = Keypair::new();
        // let token_account =
        //     AssociatedToken::find_address(&player_account.pubkey(), &mint_keypair.pubkey());

        let ix = FactionEnlistment::instruction(
            &ProcessEnlistPlayerIx { bump, faction_id },
            ProcessEnlistPlayerClientAccounts {
                player_faction_account: faction_account,
                player_account: player_account.pubkey(),
                system_program: System::ID,
                // token_program: Token::ID,
                // associated_token_program: AssociatedToken::ID,
                // mint: mint_keypair.pubkey(),
                // token_account,
            },
        )?;

        let mut tx = solana_sdk::transaction::Transaction::new_with_payer(
            &[ix],
            Some(&player_account.pubkey()),
        );
        tx.sign(
            &[&player_account], // &mint_keypair],
            banks_client.get_latest_blockhash().await?,
        );

        let txn = banks_client
            .process_transaction_with_metadata(tx.clone())
            .await?;

        println!("{:#?}", txn);

        let clock = banks_client.get_sysvar::<Clock>().await?;
        // let expected_faction_account = PlayerFactionDataAccountOwned::V1(PlayerFactionDataOwned {
        let expected_faction_account = PlayerFactionDataOwned {
            owner: player_account.pubkey(),
            enlisted_at_timestamp: clock.unix_timestamp,
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
        // });

        let faction_info = banks_client.get_account(faction_account).await?.unwrap();
        let new_faction = PlayerFactionData::deserialize_account(&faction_info.data)?;
        let serialized_account = PlayerFactionData::serialize_account(PlayerFactionDataInit {
            sized: PlayerFactionDataSized {
                owner: player_account.pubkey(),
                enlisted_at_timestamp: clock.unix_timestamp,
                faction_id,
                counter: Default::default(),
                bump,
                _padding: [0; 5],
            },
            some_fields: SomeFieldsInit {
                sized: SomeFieldsSized {
                    sized1: 10,
                    sized2: 0,
                },
                unsized1: DefaultInit,
                unsized2: [5],
            },
        })?;
        assert_eq!(serialized_account, faction_info.data);
        assert_eq!(expected_faction_account, new_faction);
        Ok(())
    }
}
