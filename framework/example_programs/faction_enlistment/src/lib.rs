#![allow(clippy::result_large_err)]

use bytemuck::{Pod, Zeroable};
use star_frame::anyhow::bail;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;
use star_frame::serialize::unsize::checked::Zeroed;

// Declare the Program ID here to embed

// #[cfg_attr(feature = "prod", program(Network::Mainnet))]
#[program(Network::Mainnet)]
#[cfg_attr(
    feature = "atlasnet",
    program(star_frame::util::Network::Custom("atlasnet"))
)]
pub struct FactionEnlistment;

impl StarFrameProgram for FactionEnlistment {
    type InstructionSet<'a> = FactionEnlistmentInstructionSet<'a>;
    type InstructionDiscriminant = u8;

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

#[instruction_set2]
pub enum FactionEnlistmentInstructionSet {
    ProcessEnlistPlayer(ProcessEnlistPlayerIx),
}

#[derive(Copy, Clone, Zeroable, Align1, Pod, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "borsh")]
#[repr(C, packed)]
pub struct ProcessEnlistPlayerIx {
    bump: u8,
    faction_id: u8,
}

impl FrameworkInstruction for ProcessEnlistPlayerIx {
    type SelfData<'a> = Self;

    type DecodeArg<'a> = ();
    type ValidateArg<'a> = u8;
    type RunArg<'a> = u8;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = ProcessEnlistPlayer<'info>
        where 'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        Self::deserialize(bytes).map_err(Into::into)
    }

    fn split_to_args<'a>(
        r: &'a Self::SelfData<'_>,
    ) -> (
        Self::DecodeArg<'a>,
        Self::ValidateArg<'a>,
        Self::RunArg<'a>,
        Self::CleanupArg<'a>,
    ) {
        ((), r.bump, r.faction_id, ())
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
                *account_set.player_faction_account.data_mut()? = PlayerFactionData {
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

#[derive(AccountSet)]
#[validate(arg = u8)]
#[account_set(skip_default_idl)]
pub struct ProcessEnlistPlayer<'info> {
    /// The player faction account
    #[validate(
        arg = Create(SeededInit {
            seeds: PlayerFactionAccountSeeds {
                player_account: *self.player_account.key()
            },
            init_create: CreateAccountWithArg::new(
                Zeroed,
                &self.system_program,
                &self.player_account,
            )
        })
    )]
    pub player_faction_account: SeededInitAccount<'info, PlayerFactionData>,
    /// The player account
    pub player_account: Writable<Signer<SystemAccount<'info>>>,
    /// Solana System program
    pub system_program: Program<'info, SystemProgram>,
}
#[derive(Debug, Align1, Copy, Clone, Pod, Zeroable /*TypeToIdl, AccountToIdl*/)]
#[repr(C, packed)]
pub struct PlayerFactionData {
    pub owner: Pubkey,
    pub enlisted_at_timestamp: i64,
    pub faction_id: u8,
    pub bump: u8,
    pub _padding: [u64; 5],
}

// TODO - Macro should derive this and with the idl feature enabled would also derive `AccountToIdl` and `TypeToIdl`
impl ProgramAccount for PlayerFactionData {
    type OwnerProgram = StarFrameDeclaredProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant =
        [47, 44, 255, 15, 103, 77, 139, 247];
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
