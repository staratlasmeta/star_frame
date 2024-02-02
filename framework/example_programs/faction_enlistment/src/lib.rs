#![allow(clippy::result_large_err)]

use bytemuck::Zeroable;
use star_frame::account_set::data_account::AccountData;
use star_frame::account_set::mutable::Writable;
use star_frame::account_set::program::Program;
use star_frame::account_set::seeded_account::{
    GetSeeds, Seed, SeededAccountData, SeededDataAccount, Seeds,
};
use star_frame::account_set::signer::Signer;
use star_frame::account_set::{AccountSet, AccountToIdl};
use star_frame::align1::Align1;
use star_frame::anyhow::bail;
use star_frame::bytemuck::Pod;
use star_frame::idl::ty::TypeToIdl;
use star_frame::idl::ProgramToIdl;
use star_frame::instruction::{FrameworkInstruction, Instruction, InstructionSet};
use star_frame::program::system_program::SystemProgram;
use star_frame::program::{program, ProgramIds, StarFrameProgram};
use star_frame::program_account::ProgramAccount;
use star_frame::pubkey;
use star_frame::serialize::unsized_type::UnsizedType;
use star_frame::serialize::{FrameworkFromBytes, FrameworkSerialize};
use star_frame::solana_program::account_info::AccountInfo;
use star_frame::solana_program::program_error::ProgramError;
use star_frame::solana_program::pubkey::Pubkey;
use star_frame::star_frame_idl::{IdlDefinition, Version};
use star_frame::sys_calls::{SysCallInvoke, SysCalls};
use star_frame::util::Network;
use star_frame::Result;
use star_frame_idl::ty::{IdlType, TypeId};

use star_frame_idl::IdlDefinitionReference;

// Declare the Program ID here to embed

// #[cfg_attr(feature = "prod", program(Network::Mainnet))]
#[program(Network::Mainnet)]
#[cfg_attr(
    feature = "atlasnet",
    program(star_frame::util::Network::Custom("atlasnet"))
)]
pub struct FactionEnlistment;

impl StarFrameProgram for FactionEnlistment {
    type InstructionSet<'a> = ();
    type InstructionDiscriminant = ();

    type AccountDiscriminant = [u8; 8];

    const CLOSED_ACCOUNT_DISCRIMINANT: Self::AccountDiscriminant = [u8::MAX; 8];
    const PROGRAM_IDS: ProgramIds = ProgramIds::Mapped(&[
        (
            Network::Mainnet,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Custom("atlasnet"),
            &pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc"),
        ),
    ]);
}
impl ProgramToIdl for FactionEnlistment {
    const VERSION: Version = Version::zeroed();

    fn program_to_idl() -> Result<IdlDefinition> {
        todo!()
    }

    fn idl_namespace() -> &'static str {
        todo!()
    }
}

// }

pub enum FactionEnlistmentInstructionSet<'a> {
    ProcessEnlistPlayer(<ProcessEnlistPlayerIx as Instruction>::SelfData<'a>),
}

impl<'a> FrameworkSerialize for FactionEnlistmentInstructionSet<'a> {
    fn to_bytes(&self, _output: &mut &mut [u8]) -> Result<()> {
        todo!()
    }
}

unsafe impl<'a> FrameworkFromBytes<'a> for FactionEnlistmentInstructionSet<'a> {
    fn from_bytes(_bytes: &mut &'a [u8]) -> Result<Self> {
        todo!()
    }
}

impl<'a> InstructionSet<'a> for FactionEnlistmentInstructionSet<'a> {
    type Discriminant = ();

    fn handle_ix(
        self,
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        sys_calls: &mut impl SysCalls,
    ) -> Result<()> {
        match self {
            FactionEnlistmentInstructionSet::ProcessEnlistPlayer(ix) => {
                ProcessEnlistPlayerIx::run_ix_from_raw(&ix, program_id, accounts, sys_calls)
            }
        }
    }
}

#[derive(Copy, Clone, Zeroable, Align1, Pod)]
#[repr(C, packed)]
pub struct ProcessEnlistPlayerIx {
    _bump: u8,
    faction_id: u8,
}

impl FrameworkInstruction for ProcessEnlistPlayerIx {
    type SelfData<'a> = <Self as UnsizedType>::Ref<'a>;

    type DecodeArg<'a> = ();
    type ValidateArg<'a> = ();
    type RunArg<'a> = u8;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = ProcessEnlistPlayer<'info>
        where 'info: 'b;

    fn data_from_bytes<'a>(_bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        todo!()
    }

    fn split_to_args<'a>(
        _r: &'a <Self as UnsizedType>::Ref<'_>,
    ) -> (
        Self::DecodeArg<'a>,
        Self::ValidateArg<'a>,
        Self::RunArg<'a>,
        Self::CleanupArg<'a>,
    ) {
        todo!()
    }

    fn run_instruction<'b, 'info>(
        faction_id: Self::RunArg<'_>,
        _program_id: &Pubkey,
        account_set: &mut Self::Accounts<'b, '_, 'info>,
        sys_calls: &mut impl SysCallInvoke,
    ) -> Result<Self::ReturnType>
    where
        'info: 'b,
    {
        match faction_id {
            0..=2 => {
                let clock = sys_calls.get_clock()?;

                let bump = account_set.player_faction_account.access_seeds().bump;
                **account_set.player_faction_account.data_mut()? = PlayerFactionData {
                    owner: *account_set.player_account.key,
                    enlisted_at_timestamp: clock.unix_timestamp,
                    faction_id,
                    bump,
                    _padding: [0; 5],
                };
                Ok(())
            }
            _ => bail!(ProgramError::Custom(69)),
        }
    }
}

// }

// #[instruction(_faction_id: u8)]

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
    // Trailing comma is super important here
    #[validate(arg = Seeds(PlayerFactionAccountSeeds {
        player_account: *self.player_account.key
    }))]
    pub player_faction_account: SeededDataAccount<'info, PlayerFactionData>,
    /// The player account
    pub player_account: Signer<Writable<AccountInfo<'info>>>,

    /// Solana System program
    pub system_program: Program<'info, SystemProgram>,
}
#[derive(Debug, Align1, Copy, Clone, Pod, Zeroable, TypeToIdl, AccountToIdl)]
// #[derive(Debug, Align1, Copy, Clone, Pod, Zeroable)]
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

// TODO - Macro should derive this and with the idl feature enabled would also derive `AccountToIdl` and `TypeToIdl`
impl AccountData for PlayerFactionData {
    type OwnerProgram = SystemProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant = ();

    fn program_id() -> Pubkey {
        todo!()
    }
}

impl SeededAccountData for PlayerFactionData {
    type Seeds = PlayerFactionAccountSeeds;
}

#[derive(Debug)]
pub struct PlayerFactionAccountSeeds {
    // #[constant(FACTION_ENLISTMENT)]
    player_account: Pubkey,
}

// TODO - Macro this
impl GetSeeds for PlayerFactionAccountSeeds {
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
