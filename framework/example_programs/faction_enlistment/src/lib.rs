// #![allow(clippy::result_large_err)]

use bytemuck::Zeroable;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

#[derive(StarFrameProgram)]
#[program(
    instruction_set = FactionEnlistmentInstructionSet<'static>
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

// use star_frame::idl::InstructionSetToIdl;

#[star_frame_instruction_set]
pub enum FactionEnlistmentInstructionSet {
    ProcessEnlistPlayer(ProcessEnlistPlayerIx),
}

#[derive(
    Copy,
    Clone,
    Zeroable,
    Align1,
    CheckedBitPattern,
    NoUninit,
    BorshDeserialize,
    BorshSerialize,
    Default,
)]
#[borsh(crate = "borsh")]
#[repr(C, packed)]
pub struct ProcessEnlistPlayerIx {
    bump: u8,
    faction_id: FactionId,
}

impl StarFrameInstruction for ProcessEnlistPlayerIx {
    type SelfData<'a> = Self;

    type DecodeArg<'a> = ();
    type ValidateArg<'a> = u8;
    type RunArg<'a> = FactionId;
    type CleanupArg<'a> = ();
    type ReturnType = ();
    type Accounts<'b, 'c, 'info> = ProcessEnlistPlayer<'info>
    where 'info: 'b;

    fn data_from_bytes<'a>(bytes: &mut &'a [u8]) -> Result<Self::SelfData<'a>> {
        Self::deserialize(bytes).map_err(Into::into)
    }

    fn split_to_args<'a>(r: &'a Self::SelfData<'_>) -> SplitToArgsReturn<'a, Self> {
        SplitToArgsReturn {
            validate: r.bump,
            run: r.faction_id,
            ..Default::default()
        }
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
        init_create: CreateAccount::new(
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
#[derive(
    Debug,
    Align1,
    Copy,
    Clone,
    CheckedBitPattern,
    NoUninit,
    Eq,
    PartialEq,
    Zeroable, /*TypeToIdl, AccountToIdl*/
)]
#[repr(C, packed)]
// #[account(seeds = PlayerFactionAccountSeeds)]
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
)]
#[borsh(crate = "borsh")]
#[repr(u8)]
pub enum FactionId {
    #[default]
    MUD,
    ONI,
    Ustur,
}

unsafe impl Zeroable for FactionId {}

// TODO - Macro should derive this and with the idl feature enabled would also derive `AccountToIdl` and `TypeToIdl`
impl ProgramAccount for PlayerFactionData {
    type OwnerProgram = StarFrameDeclaredProgram;
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant =
        [47, 44, 255, 15, 103, 77, 139, 247];
}

impl SeededAccountData for PlayerFactionData {
    type Seeds = PlayerFactionAccountSeeds;
}

#[derive(Debug, GetSeeds)]
#[seed_const(b"FACTION_ENLISTMENT")]
pub struct PlayerFactionAccountSeeds {
    player_account: Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::checked::try_from_bytes;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::clock::Clock;
    use solana_sdk::signature::Signer;
    use star_frame::borsh::to_vec;
    use star_frame::itertools::Itertools;
    use star_frame::solana_program::instruction::AccountMeta;

    // #[tokio::test]
    async fn _banks_test() -> Result<()> {
        let program_test = ProgramTest::new(
            "faction_enlistment",
            StarFrameDeclaredProgram::PROGRAM_ID,
            processor!(FactionEnlistment::processor),
        );
        let test_context = program_test.start_with_context().await;
        let mut banks_client = test_context.banks_client;

        let player_account = test_context.payer;

        let seeds = PlayerFactionAccountSeeds {
            player_account: player_account.pubkey(),
        };
        let (faction_account, bump) =
            Pubkey::find_program_address(&seeds.seeds(), &StarFrameDeclaredProgram::PROGRAM_ID);
        let faction_id = FactionId::MUD;

        let enlist_ix = ProcessEnlistPlayerIx { bump, faction_id };
        let ix_data = [
            ProcessEnlistPlayerIx::DISCRIMINANT.to_vec(),
            to_vec(&enlist_ix)?,
        ]
        .into_iter()
        .flatten()
        .collect_vec();
        let accounts = vec![
            AccountMeta::new(faction_account, false),
            AccountMeta::new(player_account.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ];
        let ix = solana_sdk::instruction::Instruction::new_with_bytes(
            FactionEnlistment::PROGRAM_ID,
            &ix_data,
            accounts,
        );
        let mut tx = solana_sdk::transaction::Transaction::new_with_payer(
            &[ix],
            Some(&player_account.pubkey()),
        );
        tx.sign(
            &[&player_account],
            banks_client.get_latest_blockhash().await?,
        );

        let _txn = banks_client
            .process_transaction_with_metadata(tx.clone())
            .await?;

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
