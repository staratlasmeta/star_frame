#![allow(clippy::result_large_err)]

use star_frame::account_set::mutable::Writable;
use star_frame::account_set::program::Program;
use star_frame::account_set::signer::Signer;
use star_frame::account_set::AccountSet;
use star_frame::anchor_replacement::account::Account;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
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

use star_frame::anchor_replacement::{AnchorCleanupArgs, AnchorValidateArgs};
use star_frame::idl::ty::TypeToIdl;
use star_frame::idl::{AccountToIdl, ProgramToIdl};
use star_frame::solana_program::program_error::ProgramError;
use star_frame::star_frame_idl::account::AccountId;
use star_frame::star_frame_idl::{IdlDefinition, Version};
use star_frame_idl::account::IdlAccount;
use star_frame_idl::seeds::IdlSeeds;

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
    #[validate(arg = AnchorValidateArgs::default())]
    #[cleanup(arg = AnchorCleanupArgs::default())]
    #[idl(arg = AnchorValidateArgs::default())]
    pub player_faction_account: Account<'info, PlayerFactionData>,
    /// The player account
    pub player_account: Signer<Writable<AccountInfo<'info>>>,

    /// Solana System program
    pub system_program: Program<'info, SystemProgram>,
}

impl AccountToIdl for PlayerFactionData {
    type AssociatedProgram = FactionEnlistment;

    fn account_to_idl(idl_definition: &mut IdlDefinition) -> Result<AccountId> {
        let namespace = if idl_definition.namespace == Self::OwnerProgram::idl_namespace() {
            let ty = Self::type_to_idl(idl_definition)?;
            idl_definition.accounts.insert(
                "PlayerFactionData".to_string(),
                IdlAccount {
                    name: "Player Faction Data".to_string(),
                    description: "The player faction data".to_string(),
                    discriminant: serde_json::to_value(Self::discriminant()).map_err(|e| {
                        star_frame::solana_program::msg!("Failed to cast to value: {:?}", e);
                        ProgramError::Custom(12)
                    })?,
                    ty,
                    seeds: IdlSeeds::NotRequired { possible: vec![] },
                    extension_fields: Default::default(),
                },
            );
            None
        } else {
            idl_definition.required_idl_definitions.insert(
                Self::OwnerProgram::idl_namespace().to_string(),
                IdlDefinitionReference {
                    namespace: Self::OwnerProgram::idl_namespace().to_string(),
                    version: Self::type_program_versions(),
                },
            );
            Some(Self::OwnerProgram::idl_namespace().to_string())
        };
        Ok(AccountId {
            namespace,
            account_id: "PlayerFactionData".to_string(),
            extension_fields: Default::default(),
        })
    }
}
//
// #[automatically_derived]
// impl<'info> ::star_frame::idl::AccountSetToIdl<'info, ()> for ProcessEnlistPlayer<'info> {
//     fn account_set_to_idl(
//         idl_definition: &mut ::star_frame::star_frame_idl::IdlDefinition,
//         arg: (),
//     ) -> ::star_frame::Result<::star_frame::star_frame_idl::account_set::IdlAccountSetDef> {
//         let __player_faction_account = <Account<'info, PlayerFactionData> as ::star_frame::idl::AccountSetToIdl<'info, _>>::account_set_to_idl(idl_definition, AnchorValidateArgs::default())?;
//         let __player_account =
//             <Signer<Writable<AccountInfo<'info>>> as ::star_frame::idl::AccountSetToIdl<
//                 'info,
//                 _,
//             >>::account_set_to_idl(idl_definition, ())?;
//         let __system_program = <Program<'info, SystemProgram> as ::star_frame::idl::AccountSetToIdl<'info, _>>::account_set_to_idl(idl_definition, ())?;
//         idl_definition.account_sets.insert(
//             "ProcessEnlistPlayer".to_string(),
//             ::star_frame::star_frame_idl::account_set::IdlAccountSet {
//                 name: "ProcessEnlistPlayer".to_string(),
//                 description: "".to_string(),
//                 type_generics: vec![],
//                 account_generics: vec![],
//                 def: ::star_frame::star_frame_idl::account_set::IdlAccountSetDef::Struct(vec![
//                     ::star_frame::star_frame_idl::account_set::IdlAccountSetStructField {
//                         name: "player_faction_account".to_string(),
//                         description: " The player faction account".to_string(),
//                         path: "player_faction_account".to_string(),
//                         account_set: __player_faction_account,
//                         extension_fields: Default::default(),
//                     },
//                     ::star_frame::star_frame_idl::account_set::IdlAccountSetStructField {
//                         name: "player_account".to_string(),
//                         description: " The player account".to_string(),
//                         path: "player_account".to_string(),
//                         account_set: __player_account,
//                         extension_fields: Default::default(),
//                     },
//                     ::star_frame::star_frame_idl::account_set::IdlAccountSetStructField {
//                         name: "system_program".to_string(),
//                         description: " Solana System program".to_string(),
//                         path: "system_program".to_string(),
//                         account_set: __system_program,
//                         extension_fields: Default::default(),
//                     },
//                 ]),
//                 extension_fields: Default::default(),
//             },
//         );
//         Ok(
//             ::star_frame::star_frame_idl::account_set::IdlAccountSetDef::AccountSet(
//                 ::star_frame::star_frame_idl::account_set::AccountSetId {
//                     namespace: None,
//                     account_set_id: "ProcessEnlistPlayer".to_string(),
//                     provided_type_generics: vec![],
//                     provided_account_generics: vec![],
//                     extension_fields: Default::default(),
//                 },
//             ),
//         )
//     }
// }

#[derive(Debug, BorshSerialize, BorshDeserialize, TypeToIdl)]
pub struct PlayerFactionData {
    pub owner: Pubkey,
    pub enlisted_at_timestamp: i64,
    pub faction_id: u8,
    pub bump: u8,
    pub _padding: [u64; 5],
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
