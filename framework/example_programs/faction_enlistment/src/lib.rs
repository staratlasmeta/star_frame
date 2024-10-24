// #![allow(clippy::result_large_err)]

use bytemuck::Zeroable;
use star_frame::borsh;
use star_frame::borsh::{BorshDeserialize, BorshSerialize};
use star_frame::prelude::*;
use star_frame::star_frame_idl::account::{IdlAccount, IdlAccountId};
use star_frame::star_frame_idl::account_set::{
    IdlAccountSet, IdlAccountSetDef, IdlAccountSetId, IdlAccountSetStructField,
};
use star_frame::star_frame_idl::seeds::{IdlSeed, IdlSeeds, IdlVariableSeed};
use star_frame::star_frame_idl::ty::{IdlEnumVariant, IdlType, IdlTypeDef, IdlTypeId};
use star_frame::star_frame_idl::{item_source, IdlDefinition, ItemInfo, Version};

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

impl ProgramToIdl for FactionEnlistment {
    fn version() -> Version {
        Version::new(0, 0, 1)
    }
}

#[derive(InstructionSet)]
pub enum FactionEnlistmentInstructionSet {
    ProcessEnlistPlayer(ProcessEnlistPlayerIx),
}

#[automatically_derived]
impl InstructionSetToIdl for FactionEnlistmentInstructionSet {
    #[allow(clippy::let_unit_value)]
    fn instruction_set_to_idl(idl_definition: &mut IdlDefinition) -> Result<()> {
        {
            // type ArgTy = ();
            // let arg: ArgTy = ();
            let definition = <ProcessEnlistPlayerIx as InstructionToIdl<_>>::instruction_to_idl(
                idl_definition,
                (),
            )?;
            let discriminant =
                <ProcessEnlistPlayerIx as InstructionDiscriminant<Self>>::discriminant_bytes();
            idl_definition.add_instruction(definition, discriminant)?;
        }
        Ok(())
    }
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
    type CleanupArg<'a> = ();
    type ReturnType = ();
    // type RunArg<'a> = (FactionId, &'a Vec<u8>);
    type RunArg<'a> = FactionId;
    type Accounts<'b, 'c, 'info> = ProcessEnlistPlayer<'info>;
    // type ReturnType = usize;

    fn split_to_args<'a>(r: &Self) -> IxArgs<Self> {
        IxArgs::run(r.faction_id)
    }

    fn run_instruction<'info>(
        account_set: &mut Self::Accounts<'_, '_, 'info>,
        faction_id: Self::RunArg<'_>,
        syscalls: &mut impl SyscallInvoke<'info>,
    ) -> Result<Self::ReturnType> {
        // let cloned_account = account_set.player_account.clone();
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
#[account_set(skip_default_idl)]
pub struct ProcessEnlistPlayer<'info> {
    /// The player faction account
    #[validate(arg = (Create(CreateAccount::new(&self.system_program, &self.player_account)),
    Seeds(PlayerFactionAccountSeeds {
        player_account: *self.player_account.key()
    })))]
    pub player_faction_account: Init<Seeded<DataAccount<'info, PlayerFactionData>>>,
    /// The player account
    #[account_set(funder)]
    pub player_account: Writable<Signer<SystemAccount<'info>>>,
    /// Solana System program
    #[account_set(system_program)]
    pub system_program: Program<'info, SystemProgram>,
}

#[automatically_derived]
impl<'info> AccountSetToIdl<'info, ()> for ProcessEnlistPlayer<'info> {
    fn account_set_to_idl(idl_definition: &mut IdlDefinition, arg: ()) -> Result<IdlAccountSetDef> {
        let source = item_source::<Self>();
        let account_set_def = IdlAccountSetDef::Struct(vec![
            IdlAccountSetStructField {
                path: "player_faction_account".to_string(),
                description: vec![],
                account_set_def:
                    <Init<Seeded<DataAccount<'info, PlayerFactionData>>>>::account_set_to_idl(
                        idl_definition,
                        (),
                    )?,
            },
            IdlAccountSetStructField {
                path: "player_account".to_string(),
                description: vec![],
                account_set_def: <Writable<Signer<SystemAccount<'info>>>>::account_set_to_idl(
                    idl_definition,
                    (),
                )?,
            },
            IdlAccountSetStructField {
                path: "system_program".to_string(),
                description: vec![],
                account_set_def: <Program<'info, SystemProgram>>::account_set_to_idl(
                    idl_definition,
                    (),
                )?,
            },
        ]);
        let account_set = IdlAccountSet {
            info: ItemInfo::new::<Self>("ProcessEnlistPlayer", vec![]),
            account_set_def,
            type_generics: vec![],
            account_generics: vec![],
        };

        idl_definition.add_account_set(account_set);
        Ok(IdlAccountSetDef::Defined(IdlAccountSetId {
            source,
            provided_type_generics: vec![],
            provided_account_generics: vec![],
        }))
    }
}

#[derive(
    Debug, Align1, Copy, Clone, CheckedBitPattern, NoUninit, Eq, PartialEq, Zeroable, TypeToIdl,
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

impl TypeToIdl for FactionId {
    type AssociatedProgram = crate::StarFrameDeclaredProgram;
    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        let source = item_source::<Self>();
        let type_def = IdlTypeDef::Enum(vec![
            IdlEnumVariant {
                name: "MUD".to_string(),
                discriminant: vec![0],
                type_def: None,
            },
            IdlEnumVariant {
                name: "ONI".to_string(),
                discriminant: vec![1],
                type_def: None,
            },
            IdlEnumVariant {
                name: "USTUR".to_string(),
                discriminant: vec![2],
                type_def: None,
            },
        ]);
        let idl_type = IdlType {
            info: ItemInfo {
                name: "FactionId".to_string(),
                description: vec![],
                source: source.clone(),
            },
            type_def,
            generics: vec![],
        };
        let namespace = idl_definition.add_type(idl_type, Self::AssociatedProgram::PROGRAM_ID);
        Ok(IdlTypeDef::Defined(IdlTypeId {
            namespace,
            source,
            provided_generics: vec![],
        }))
    }
}

unsafe impl Zeroable for FactionId {}

// TODO - Macro should derive this and with the idl feature enabled would also derive `AccountToIdl` and `TypeToIdl`
impl ProgramAccount for PlayerFactionData {
    const DISCRIMINANT: <Self::OwnerProgram as StarFrameProgram>::AccountDiscriminant =
        [47, 44, 255, 15, 103, 77, 139, 247];
}

impl HasOwnerProgram for PlayerFactionData {
    type OwnerProgram = FactionEnlistment;
}

impl HasSeeds for PlayerFactionData {
    type Seeds = PlayerFactionAccountSeeds;
}

impl AccountToIdl for PlayerFactionData {
    fn account_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlAccountId> {
        let source = item_source::<Self>();
        let idl_account = IdlAccount {
            discriminant: Self::discriminant_bytes(),
            seeds: Some(IdlSeeds(vec![
                IdlSeed::Const(b"FACTION_ENLISTMENT".into()),
                IdlSeed::Variable(IdlVariableSeed {
                    name: "player_account".to_string(),
                    description: vec![],
                    ty: <Pubkey>::type_to_idl(idl_definition)?,
                }),
            ])),
            type_def: <PlayerFactionData>::type_to_idl(idl_definition)?,
        };
        let namespace =
            idl_definition.add_account(idl_account, Self::AssociatedProgram::PROGRAM_ID)?;
        Ok(IdlAccountId { namespace, source })
    }
}

#[derive(Debug, GetSeeds, Clone)]
#[seed_const(b"FACTION_ENLISTMENT")]
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
    use star_frame::borsh::to_vec;
    use star_frame::itertools::Itertools;
    use star_frame::solana_program::instruction::AccountMeta;
    use star_frame::solana_program::native_token::LAMPORTS_PER_SOL;

    #[test]
    fn idl() {
        let idl = FactionEnlistment::program_to_idl().unwrap();
        println!("{}", serde_json::to_string_pretty(&idl).unwrap());
    }

    #[tokio::test]
    async fn banks_test() -> Result<()> {
        const SBF_FILE: bool = false;
        let program_test = if SBF_FILE {
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

        let enlist_ix = ProcessEnlistPlayerIx {
            bump,
            faction_id,
            // buncha_data,
        };
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

        let txn = banks_client
            .process_transaction_with_metadata(tx.clone())
            .await?;

        println!("{:?}", txn);

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
