#![allow(clippy::result_large_err)]
use star_frame::account_set::mutable::Writable;
use star_frame::account_set::program::Program;
use star_frame::account_set::signer::Signer;
use star_frame::account_set::{AccountSet, SingleAccountSet};
use star_frame::anchor_replacement::account::Account;
use star_frame::program::system_program::SystemProgram;
use star_frame::program::StarFrameProgram;
use star_frame::solana_program::account_info::AccountInfo;
use star_frame::solana_program::pubkey::Pubkey;
use star_frame::util::Network;
use star_frame::Result;
use star_frame::{declare_id, pubkey};

// Declare the Program ID here to embed
#[cfg(feature = "prod")]
declare_id!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe");

#[cfg(not(feature = "prod"))]
declare_id!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc");

pub struct FactionEnlistment;

const fn search_for_network(program_ids: ProgramIds, network: Network) -> Option<Pubkey> {
    match program_ids {
        ProgramIds::Mapped(ids) => {
            let mut index = 0;
            loop {
                if index >= ids.len() {
                    break None;
                }

                let item_network = &ids[index];
                match (&item_network.0, &network) {
                    (Network::MainNet, Network::MainNet)
                    | (Network::DevNet, Network::DevNet)
                    | (Network::TestNet, Network::TestNet) => break Some(*item_network.1),
                    (Network::Custom(network1), Network::Custom(network2))
                        if network1 == network2 =>
                    {
                        break Some(*item_network.1)
                    }
                    (_, _) => (),
                }

                index += 1;
            }
        }
        ProgramIds::AllNetworks(id) => Some(*id),
    }
}

impl StarFrameProgram for FactionEnlistment {
    type InstructionSet<'a> = ();
    type InstructionDiscriminant = ();

    const PROGRAM_IDS: ProgramIds = ProgramIds::Mapped(&[
        (
            Network::MainNet,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Custom("atlasnet"),
            &pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc"),
        ),
    ]);

    const PROGRAM_ID: Pubkey =
        search_for_network(Self::PROGRAM_IDS, Network::Custom("atlasnet")).unwrap();
    type AccountDiscriminant = ();
    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = ();
}

star_frame::idl::declare_program_type!(FactionEnlistment);

pub mod faction_enlistment {
    use super::*;
    pub fn process_enlist_player(
        ctx: Context<ProcessEnlistPlayer>,
        _bump: u8, // we are keeping this for backwards compatibility
        faction_id: u8,
    ) -> Result<()> {
        match faction_id {
            0..=2 => {
                let player_faction_account_info = &mut ctx.accounts.player_faction_account;
                player_faction_account_info.owner = ctx.accounts.player_account.key();
                player_faction_account_info.enlisted_at_timestamp =
                    ctx.accounts.clock.unix_timestamp;
                player_faction_account_info.faction_id = faction_id;
                player_faction_account_info.bump =
                    *ctx.bumps.get("player_faction_account").unwrap();
                Ok(())
            }
            _ => Err(error!(FactionErrors::FactionTypeError)),
        }
    }
}

// #[instruction(_faction_id: u8)]

#[derive(AccountSet, Debug)]
pub struct ProcessEnlistPlayer<'info> {
    /// The player faction account
    #[account(
        init,
        payer = player_account,
        seeds = [b"FACTION_ENLISTMENT".as_ref(), player_account.key.as_ref()],
        bump,
        space = PlayerFactionData::LEN
    )]
    pub player_faction_account: Account<'info, PlayerFactionData>,
    /// The player account
    pub player_account: Signer<Writable<AccountInfo<'info>>>,

    /// Solana System program
    pub system_program: Program<'info, SystemProgram>,
}

#[derive(Debug)]
pub struct PlayerFactionData {
    pub owner: Pubkey,
    pub enlisted_at_timestamp: i64,
    pub faction_id: u8,
    pub bump: u8,
    pub _padding: [u64; 5],
}

pub const ANCHOR_DISC_LEN: usize = 8;

impl PlayerFactionData {
    pub const LEN: usize = ANCHOR_DISC_LEN
        + 32 // owner
        + 8 // enlisted_at_timestamp
        + 1 // faction_id
        + 1 // bump
        + 8 * 5; // _padding
}

// #[error_code]
pub enum FactionErrors {
    /// 6000
    // #[msg("Faction ID must be 0, 1, or 2.")]
    FactionTypeError,
}
