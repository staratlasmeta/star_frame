#![allow(clippy::result_large_err)]

use bytemuck::Zeroable;
use star_frame::account_set::data_account::{AccountData, DataAccount};
use star_frame::account_set::mutable::Writable;
use star_frame::account_set::program::Program;
use star_frame::account_set::seeded_account::{Seed, SeededAccount, Seeds};
use star_frame::account_set::signer::Signer;
use star_frame::account_set::{AccountSet, AccountToIdl};
use star_frame::align1::Align1;
use star_frame::program::system_program::SystemProgram;
use star_frame::program::{ProgramIds, StarFrameProgram};
use star_frame::program_account::ProgramAccount;
use star_frame::solana_program::account_info::AccountInfo;
use star_frame::solana_program::pubkey::Pubkey;
use star_frame::util::Network;
use star_frame::Result;
use star_frame::{declare_id, pubkey};
use star_frame_idl::ty::{IdlType, TypeId};
use star_frame_idl::IdlDefinitionReference;

// Declare the Program ID here to embed
#[cfg(feature = "prod")]
declare_id!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe");

#[cfg(not(feature = "prod"))]
declare_id!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc");

pub struct FactionEnlistment;

impl StarFrameProgram for FactionEnlistment {
    type InstructionSet<'a> = ();
    type InstructionDiscriminant = ();

    type AccountDiscriminant = [u8; 8];

    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = [u8::MAX; 8];
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
}

star_frame::program::declare_program_type!(FactionEnlistment);

impl ProgramToIdl for FactionEnlistment {
    const VERSION: Version = Version::zeroed();

    fn program_to_idl() -> Result<IdlDefinition> {
        todo!()
    }

    fn idl_namespace() -> &'static str {
        todo!()
    }
}

// pub mod faction_enlistment {
//     use super::*;
//     pub fn process_enlist_player(
//         ctx: Context<ProcessEnlistPlayer>,
//         _bump: u8, // we are keeping this for backwards compatibility
//         faction_id: u8,
//     ) -> Result<()> {
//         match faction_id {
//             0..=2 => {
//                 let player_faction_account_info = &mut ctx.accounts.player_faction_account;
//                 player_faction_account_info.owner = ctx.accounts.player_account.key();
//                 player_faction_account_info.enlisted_at_timestamp =
//                     ctx.accounts.clock.unix_timestamp;
//                 player_faction_account_info.faction_id = faction_id;
//                 player_faction_account_info.bump =
//                     *ctx.bumps.get("player_faction_account").unwrap();
//                 Ok(())
//             }
//             _ => Err(error!(FactionErrors::FactionTypeError)),
//         }
//     }
// }

// #[instruction(_faction_id: u8)]

use star_frame::bytemuck::Pod;
use star_frame::idl::ty::TypeToIdl;
use star_frame::idl::ProgramToIdl;
use star_frame::star_frame_idl::{IdlDefinition, Version};

#[derive(AccountSet, Debug)]
// #[account_set(skip_default_idl)]
pub struct ProcessEnlistPlayer<'info> {
    /// The player faction account
    // #[account(
    //     init,
    //     payer = player_account,
    //     seeds = [b"FACTION_ENLISTMENT".as_ref(), player_account.key.as_ref()],
    //     bump,
    //     space = PlayerFactionData::LEN
    // )]
    // #[validate(arg = AnchorValidateArgs::default())]
    // #[cleanup(arg = AnchorCleanupArgs::default())]
    // pub player_faction_account: Account<'info, PlayerFactionData>,
    // TODO - How do we store/access the bump?
    // TODO - This isn't the right way to build this struct
    // #[validate(arg = (SeedsWithBump {
    //     seeds: PlayerFactionAccountSeeds {
    //         player_account: *self.player_account.key
    //     },
    //     bump: 255
    // }, ()))]
    #[validate(arg = (PlayerFactionAccountSeeds {
    player_account: *self.player_account.key
    }, ()))]
    pub player_faction_account:
        SeededAccount<DataAccount<'info, PlayerFactionData>, PlayerFactionAccountSeeds>,
    /// The player account
    pub player_account: Signer<Writable<AccountInfo<'info>>>,

    /// Solana System program
    pub system_program: Program<'info, SystemProgram>,
}
#[derive(Debug, Align1, Copy, Clone, Pod, Zeroable, TypeToIdl, AccountToIdl)]
#[repr(C, packed)]
// #[derive(AccountData)]
// #[owner_program(FactionEnlistment)]
pub struct PlayerFactionData {
    pub owner: Pubkey,
    pub enlisted_at_timestamp: i64,
    pub faction_id: u8,
    pub bump: u8,
    pub _padding: [u64; 5],
}

impl AccountData for PlayerFactionData {
    type OwnerProgram = SystemProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant = ();

    fn program_id() -> Pubkey {
        todo!()
    }
}

#[derive(Debug)]
pub struct PlayerFactionAccountSeeds {
    // #[constant(FACTION_ENLISTMENT)]
    player_account: Pubkey,
}

impl Seeds for PlayerFactionAccountSeeds {
    fn seeds(&self) -> Vec<&[u8]> {
        vec![b"FACTION_ENLISTMENT".as_ref(), self.player_account.seed()]
    }
}

/* TODO - Default implementation can assume anchor hash for discriminant,
maybe require manual implementation if you want something else for now? */
// Why can't you do multi line TODOs?
impl ProgramAccount for PlayerFactionData {
    type OwnerProgram = FactionEnlistment;

    fn discriminant() -> [u8; 8] {
        Default::default()
    }
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
