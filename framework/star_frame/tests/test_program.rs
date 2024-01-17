#![cfg(feature = "idl")]
#![feature(ptr_metadata)]

use ixs::TestProgramInstructions;
use lazy_static::lazy_static;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use star_frame::idl::ty::TypeToIdl;
use star_frame::idl::{AccountToIdl, InstructionSetToIdl, ProgramToIdl};
use star_frame::program::{Program, ProgramIds};
use star_frame::program_account::ProgramAccount;
use star_frame::util::Network;
use star_frame::Result;
use star_frame_idl::account::{AccountId, IdlAccount};
use star_frame_idl::seeds::IdlSeeds;
use star_frame_idl::ty::{IdlField, IdlType, IdlTypeDef, TypeId};
use star_frame_idl::{DiscriminantId, IdlDefinition, IdlDefinitionReference, Version};

mod ixs;

use star_frame::declare_id;
use star_frame_proc::pubkey;

declare_id!("11111111111111111111111111111111");
const KEY: Pubkey = pubkey!("11111111111111111111111111111111");

#[test]
fn print_idl() {
    let idl = TestProgram::program_to_idl().unwrap();
    println!("{}", serde_json::to_string_pretty(&idl).unwrap());
}

lazy_static! {
    pub static ref PROGRAM_PUBKEY: Pubkey = Pubkey::new_unique();
    pub static ref DEV_PROGRAM_PUBKEY: Pubkey = Pubkey::new_unique();
    pub static ref TEST_PROGRAM_PUBKEYS: [(Network, &'static Pubkey); 3] = [
        (Network::MainNet, &PROGRAM_PUBKEY),
        (Network::DevNet, &DEV_PROGRAM_PUBKEY),
        (Network::TestNet, &DEV_PROGRAM_PUBKEY),
    ];
}

pub struct TestProgram;
impl Program for TestProgram {
    type InstructionSet<'a> = TestProgramInstructions<'a>;
    type InstructionDiscriminant = u32;

    fn program_id() -> ProgramIds {
        ProgramIds::Mapped(&*TEST_PROGRAM_PUBKEYS)
    }
}
impl ProgramToIdl for TestProgram {
    const VERSION: Version = Version {
        major: 0,
        minor: 1,
        patch: 0,
    };

    fn program_to_idl() -> Result<IdlDefinition> {
        let mut def = IdlDefinition {
            idl_std_version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
            version: Self::VERSION,
            name: "Test Program".to_string(),
            namespace: Self::idl_namespace().to_string(),
            description: "A test program for testing".to_string(),
            required_plugins: Default::default(),
            required_idl_definitions: Default::default(),
            program_ids: Self::program_id().into(),
            account_discriminant: DiscriminantId::U32,
            instruction_discriminant: DiscriminantId::U32,
            accounts: Default::default(),
            types: Default::default(),
            account_sets: Default::default(),
            instructions: Default::default(),
            extension_fields: Default::default(),
        };

        Self::InstructionSet::instruction_set_to_idl(&mut def)?;

        Ok(def)
    }

    fn idl_namespace() -> &'static str {
        "@staratlas/test-program"
    }
}

pub struct TestAccount1 {
    pub val: u32,
    pub val2: u64,
    pub val3: i8,
}
impl ProgramAccount for TestAccount1 {
    type OwnerProgram = TestProgram;

    fn discriminant() -> <Self::OwnerProgram as Program>::InstructionDiscriminant {
        1
    }
}
impl TypeToIdl for TestAccount1 {
    type AssociatedProgram = TestProgram;

    fn type_to_idl(idl_definition: &mut IdlDefinition) -> Result<IdlTypeDef> {
        let namespace = if idl_definition.namespace == Self::AssociatedProgram::idl_namespace() {
            let val = <u32 as TypeToIdl>::type_to_idl(idl_definition)?;
            let val2 = <u64 as TypeToIdl>::type_to_idl(idl_definition)?;
            let val3 = <i8 as TypeToIdl>::type_to_idl(idl_definition)?;
            let ty = IdlType {
                name: "Test Account 1".to_string(),
                description: "The first test account".to_string(),
                generics: vec![],
                type_def: IdlTypeDef::Struct(vec![
                    IdlField {
                        name: "val".to_string(),
                        description: "First Value".to_string(),
                        path_id: "val".to_string(),
                        type_def: val,
                        extension_fields: Default::default(),
                    },
                    IdlField {
                        name: "val 2".to_string(),
                        description: "The second Value".to_string(),
                        path_id: "val2".to_string(),
                        type_def: val2,
                        extension_fields: Default::default(),
                    },
                    IdlField {
                        name: "val 3".to_string(),
                        description: "The third value".to_string(),
                        path_id: "val3".to_string(),
                        type_def: val3,
                        extension_fields: Default::default(),
                    },
                ]),
                extension_fields: Default::default(),
            };
            idl_definition.types.insert("TestAccount1".to_string(), ty);
            None
        } else {
            idl_definition.required_idl_definitions.insert(
                Self::AssociatedProgram::idl_namespace().to_string(),
                IdlDefinitionReference {
                    version: Self::type_program_versions(),
                    namespace: Self::AssociatedProgram::idl_namespace().to_string(),
                },
            );
            Some(Self::AssociatedProgram::idl_namespace().to_string())
        };
        Ok(IdlTypeDef::IdlType(TypeId {
            namespace,
            type_id: "TestAccount1".to_string(),
            provided_generics: vec![],
            extension_fields: Default::default(),
        }))
    }
}
impl AccountToIdl for TestAccount1 {
    type AssociatedProgram = TestProgram;

    fn account_to_idl(idl_definition: &mut IdlDefinition) -> Result<AccountId> {
        let namespace = if idl_definition.namespace == Self::OwnerProgram::idl_namespace() {
            let ty = Self::type_to_idl(idl_definition)?;
            idl_definition.accounts.insert(
                "TestAccount1".to_string(),
                IdlAccount {
                    name: "Test Account 1".to_string(),
                    description: "The first Test account".to_string(),
                    discriminant: serde_json::to_value(Self::discriminant()).map_err(|e| {
                        msg!("Error serde_json to_value: {:?}", e);
                        // TODO: Change error?
                        ProgramError::Custom(1)
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
                    version: Self::account_program_versions(),
                    namespace: Self::OwnerProgram::idl_namespace().to_string(),
                },
            );
            Some(Self::OwnerProgram::idl_namespace().to_string())
        };
        Ok(AccountId {
            namespace,
            account_id: "TestAccount1".to_string(),
            extension_fields: Default::default(),
        })
    }
}

pub struct TestAccount2 {
    pub key: Pubkey,
    pub stuff: Pubkey,
    pub ty: [TestType; 12],
}

pub struct TestType {
    pub val1: u32,
    pub key: Pubkey,
}
