use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;
use star_frame_spl::associated_token::AssociatedTokenAccount;
use star_frame_spl::associated_token::AssociatedTokenProgram;
use star_frame_spl::associated_token::InitAta;
use star_frame_spl::token::InitMint;
use star_frame_spl::token::{MintAccount, TokenProgram};

#[derive(StarFrameProgram)]
#[program(
    instruction_set = FactionEnlistmentInstructionSet
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
        *account_set.player_faction_account.data_mut()? = PlayerFactionData {
            owner: *account_set.player_account.key,
            enlisted_at_timestamp: clock.unix_timestamp,
            faction_id,
            bump,
            _padding: [0; 5],
        };
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
    pub player_faction_account: Init<Seeded<DataAccount<'info, PlayerFactionData>>>,
    /// The player account
    #[account_set(funder)]
    pub player_account: Mut<Signer<SystemAccount<'info>>>,
    /// Solana System program
    pub system_program: Program<'info, SystemProgram>,
    pub token_program: Program<'info, TokenProgram>,
    pub associated_token_program: Program<'info, AssociatedTokenProgram>,
    #[validate(arg = Create(InitMint {
        decimals: 0,
        mint_authority: self.player_account.key(),
        freeze_authority: Some(self.player_account.key()),
    }))]
    pub mint: Init<Signer<MintAccount<'info>>>,
    #[validate(arg = Create(InitAta {
        wallet: &self.player_account, 
        mint: &self.mint, 
        system_program: &self.system_program, 
        token_program: &self.token_program
    }))]
    pub token_account: Init<AssociatedTokenAccount<'info>>,
}

#[derive(
    ProgramAccount, Debug, Align1, Copy, Clone, CheckedBitPattern, NoUninit, Eq, PartialEq, Zeroable,
)]
#[repr(C, packed)]
#[program_account(seeds = PlayerFactionAccountSeeds)]
pub struct PlayerFactionData {
    pub owner: Pubkey,
    pub enlisted_at_timestamp: i64,
    pub faction_id: FactionId,
    pub bump: u8,
    pub _padding: [u64; 5],
}

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
    use bytemuck::checked::try_from_bytes;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::account::Account;
    use solana_sdk::clock::Clock;
    use solana_sdk::signature::{Keypair, Signer};
    use star_frame::solana_program::native_token::LAMPORTS_PER_SOL;
    use star_frame_spl::token::TokenProgram;

    #[cfg(feature = "idl")]
    #[test]
    fn idl() {
        let idl = FactionEnlistment::program_to_idl().unwrap();
        println!(
            "{}",
            star_frame::serde_json::to_string_pretty(&idl).unwrap()
        );
    }

    #[tokio::test]
    async fn banks_test() -> Result<()> {
        let program_test = if option_env!("USE_BIN").is_some() {
            let target_dir = std::env::current_dir()?
                .join("../../../target/deploy")
                .canonicalize()?;
            std::env::set_var(
                "BPF_OUT_DIR",
                target_dir.to_str().expect("Failed to convert path to str"),
            );
            ProgramTest::new(
                "faction_enlistment",
                StarFrameDeclaredProgram::PROGRAM_ID,
                None,
            )
        } else {
            ProgramTest::new(
                "faction_enlistment",
                StarFrameDeclaredProgram::PROGRAM_ID,
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
                Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::PROGRAM_ID);
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

        let mint_keypair = Keypair::new();
        let token_account =
            AssociatedTokenProgram::find_address(&player_account.pubkey(), &mint_keypair.pubkey());

        let ix = FactionEnlistment::instruction(
            &ProcessEnlistPlayerIx { bump, faction_id },
            ProcessEnlistPlayerClientAccounts {
                player_faction_account: faction_account,
                player_account: player_account.pubkey(),
                system_program: SystemProgram::PROGRAM_ID,
                token_program: TokenProgram::PROGRAM_ID,
                associated_token_program: AssociatedTokenProgram::PROGRAM_ID,
                mint: mint_keypair.pubkey(),
                token_account,
            },
        )?;

        let mut tx = solana_sdk::transaction::Transaction::new_with_payer(
            &[ix],
            Some(&player_account.pubkey()),
        );
        tx.sign(
            &[&player_account, &mint_keypair],
            banks_client.get_latest_blockhash().await?,
        );

        let txn = banks_client
            .process_transaction_with_metadata(tx.clone())
            .await?;

        println!("{:#?}", txn);

        let clock = banks_client.get_sysvar::<Clock>().await?;
        let expected_faction_account = PlayerFactionData {
            owner: player_account.pubkey(),
            enlisted_at_timestamp: clock.unix_timestamp,
            faction_id,
            bump,
            _padding: [0; 5],
        };

        let faction_info = banks_client.get_account(faction_account).await?.unwrap();
        assert_eq!(faction_info.data[0..8], PlayerFactionData::DISCRIMINANT);
        let new_faction: &PlayerFactionData = try_from_bytes(&faction_info.data[8..])?;
        assert_eq!(expected_faction_account, *new_faction);
        Ok(())
    }
}
