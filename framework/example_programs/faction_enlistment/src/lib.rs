// #![allow(clippy::result_large_err)]

use bytemuck::Zeroable;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;

// Declare the Program ID here to embed
// #[cfg_attr(feature = "prod", program(Network::Mainnet))]
#[program(Network::MainnetBeta)]
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
            Network::MainnetBeta,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Devnet,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Localhost,
            &pubkey!("FACTNmq2FhA2QNTnGM2aWJH3i7zT3cND5CgvjYTjyVYe"),
        ),
        (
            Network::Custom("atlasnet"),
            &pubkey!("FLisTRH6dJnCK8AzTfenGJgHBPMHoat9XRc65Qpk7Yuc"),
        ),
    ]);
}

#[star_frame_instruction_set]
pub enum FactionEnlistmentInstructionSet {
    ProcessEnlistPlayer(ProcessEnlistPlayerIx),
}

#[derive(
    Copy, Clone, Zeroable, Align1, CheckedBitPattern, NoUninit, BorshDeserialize, BorshSerialize,
)]
#[borsh(crate = "borsh")]
#[repr(C, packed)]
pub struct ProcessEnlistPlayerIx {
    bump: u8,
    faction_id: FactionId,
}

impl FrameworkInstruction for ProcessEnlistPlayerIx {
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
    Debug, Copy, Clone, CheckedBitPattern, NoUninit, BorshDeserialize, BorshSerialize, Eq, PartialEq,
)]
#[borsh(crate = "borsh")]
#[repr(u8)]
pub enum FactionId {
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::checked::try_from_bytes;
    use solana_client::rpc_config::RpcTransactionConfig;
    use solana_sdk::commitment_config::CommitmentConfig;
    use solana_sdk::native_token::LAMPORTS_PER_SOL;
    use solana_sdk::signature::{Keypair, Signer};
    use star_frame::solana_program::instruction::AccountMeta;

    #[tokio::test]
    async fn init_stuff() -> Result<()> {
        let client = solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment(
            "http://localhost:8899".to_string(),
            CommitmentConfig::confirmed(),
        );

        let player_account = Keypair::new();
        let res = client
            .request_airdrop(&player_account.pubkey(), LAMPORTS_PER_SOL)
            .await
            .unwrap();
        client.poll_for_signature(&res).await.unwrap();

        let seeds = PlayerFactionAccountSeeds {
            player_account: player_account.pubkey(),
        };
        let (faction_account, bump) = Pubkey::find_program_address(&seeds.seeds(), &crate::ID);
        let faction_id = FactionId::MUD;

        // 1 for ix disc, 1 for
        let ix_data = [0, bump, faction_id as u8];
        let accounts = vec![
            AccountMeta::new(faction_account, false),
            AccountMeta::new(player_account.pubkey(), true),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ];
        let ix =
            solana_sdk::instruction::Instruction::new_with_bytes(crate::ID, &ix_data, accounts);
        let mut tx = solana_sdk::transaction::Transaction::new_with_payer(
            &[ix],
            Some(&player_account.pubkey()),
        );
        let rbh = client.get_latest_blockhash().await.unwrap();
        tx.sign(&[&player_account], rbh);
        let res = client.send_and_confirm_transaction(&tx).await?;
        let tx = client
            .get_transaction_with_config(
                &res,
                RpcTransactionConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                    ..Default::default()
                },
            )
            .await?;
        println!("Enlist txn: {res:?}");
        let clock = client.get_block_time(tx.slot).await?;

        let expected_faction_account = PlayerFactionData {
            owner: player_account.pubkey(),
            enlisted_at_timestamp: clock,
            faction_id,
            bump,
            _padding: [0; 5],
        };

        let faction_info = client.get_account(&faction_account).await?;
        assert_eq!(faction_info.data[0..8], PlayerFactionData::DISCRIMINANT);
        let new_faction: &PlayerFactionData = try_from_bytes(&faction_info.data[8..])?;
        assert_eq!(expected_faction_account, *new_faction);

        Ok(())
    }
}
