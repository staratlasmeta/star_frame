extern crate alloc;

pub mod account;
pub mod account_discriminant;
pub mod account_set;
pub mod instruction;
pub mod seeds;
pub mod serde_impls;
pub mod ty;
#[cfg(feature = "verifier")]
pub mod verifier;

use crate::serde_impls::serde_base58_pubkey;
use account::IdlAccount;
use account_set::IdlAccountSet;
use instruction::IdlInstruction;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::fmt::Debug;
use ty::IdlType;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlDefinition {
    /// Version of the `IdlDefinition`
    pub idl_std_version: Version,
    /// Version of program
    pub version: Version,
    /// Name of program (human readable)
    pub name: String,
    /// Name of program (computer readable)
    pub namespace: String,
    /// Human readable description
    pub description: String,
    /// Id can be remapped by using a different key as the id.
    /// Plugin id example: `@staratlas/binary-heap`
    pub required_plugins: HashMap<String, Plugin>,
    pub required_idl_definitions: HashMap<String, IdlDefinitionReference>,
    pub program_ids: ProgramIds,
    pub account_discriminant: DiscriminantId,
    pub instruction_discriminant: DiscriminantId,
    pub accounts: HashMap<String, IdlAccount>,
    pub types: HashMap<String, IdlType>,
    pub account_sets: HashMap<String, IdlAccountSet>,
    pub instructions: HashMap<String, IdlInstruction>,
    // #[serde(with = "serde_as_option", skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}
impl IdlDefinition {
    pub fn add_account_if_missing(&mut self, id: &str, account: impl FnOnce() -> IdlAccount) {
        if !self.accounts.contains_key(id) {
            self.accounts.insert(id.to_string(), account());
        }
    }

    pub fn add_type_if_missing(&mut self, id: &str, ty: impl FnOnce() -> IdlType) {
        if !self.types.contains_key(id) {
            self.types.insert(id.to_string(), ty());
        }
    }

    pub fn add_account_set_if_missing(
        &mut self,
        id: &str,
        account_set: impl FnOnce() -> IdlAccountSet,
    ) {
        if !self.account_sets.contains_key(id) {
            self.account_sets.insert(id.to_string(), account_set());
        }
    }

    pub fn add_instruction_if_missing(
        &mut self,
        id: &str,
        instruction: impl FnOnce() -> IdlInstruction,
    ) {
        if !self.instructions.contains_key(id) {
            self.instructions.insert(id.to_string(), instruction());
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Plugin {
    pub id: String,
    pub version: SemVer,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlDefinitionReference {
    pub namespace: String,
    pub version: SemVer,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ExtensionClass {
    pub namespace: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SemVer {
    /// Same as rust `*`
    Wildcard,
    /// Same as rust `=1.2.3`
    Exact(Version),
    /// Same as rust `>=1.2.3, <1.3.0` or `>=1.2.3,<=1.2.4` if inclusive.
    /// If max is all 0s and inclusive 0.0.0 will be valid
    Range {
        min: Version,
        max: Version,
        inclusive: bool,
    },
    /// Same as rust `1.*` and `1`
    Major(u8),
    /// Same as rust `~1.2` and `1.2.*`
    MajorMinor { major: u8, minor: u8 },
    /// Same as rust `~1.2.3`
    MajorMinorPatch(Version),
}
impl SemVer {
    /// Follows solana semver not rust.
    pub fn from_version(version: Version) -> Self {
        Self::MajorMinorPatch(version)
    }

    pub fn max(&self, other: Self) -> Option<Self> {
        match (*self, other) {
            (x, y) if x == y => Some(x),
            (Self::Wildcard, y) => Some(y),
            (x, Self::Wildcard) => Some(x),
            (Self::Exact(x), y) => {
                if y.version_matches(x) {
                    Some(y)
                } else {
                    None
                }
            }
            (x, Self::Exact(y)) => {
                if x.version_matches(y) {
                    Some(x)
                } else {
                    None
                }
            }
            (x, y) => {
                let min = x.min_supported().min(y.min_supported());
                let max = x.max_supported().max(y.max_supported());
                if min <= max {
                    Some(SemVer::Range {
                        min,
                        max,
                        inclusive: true,
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn min_supported(&self) -> Version {
        match *self {
            SemVer::Wildcard => Version {
                major: 0,
                minor: 0,
                patch: 0,
            },
            SemVer::Exact(v) => v,
            SemVer::Range { min, .. } => min,
            SemVer::Major(major) => Version {
                major,
                minor: 0,
                patch: 0,
            },
            SemVer::MajorMinor { major, minor } => Version {
                major,
                minor,
                patch: 0,
            },
            SemVer::MajorMinorPatch(v) => v,
        }
    }

    pub fn max_supported(&self) -> Version {
        match *self {
            SemVer::Wildcard => Version {
                major: u8::MAX,
                minor: u8::MAX,
                patch: u8::MAX,
            },
            SemVer::Exact(v) => v,
            SemVer::Range { max, inclusive, .. } => {
                if inclusive {
                    max
                } else {
                    Version {
                        major: max
                            .major
                            .saturating_sub((max.minor == 0 && max.patch == 0) as u8),
                        minor: max.minor.saturating_sub((max.patch == 0) as u8),
                        patch: max.patch.saturating_sub(1),
                    }
                }
            }
            SemVer::Major(major) => Version {
                major,
                minor: u8::MAX,
                patch: u8::MAX,
            },
            SemVer::MajorMinor { major, minor } => Version {
                major,
                minor,
                patch: u8::MAX,
            },
            SemVer::MajorMinorPatch(v) => v,
        }
    }

    pub fn version_matches(&self, version: Version) -> bool {
        match self {
            SemVer::Wildcard => true,
            SemVer::Exact(v) => v == &version,
            SemVer::Range {
                min,
                max,
                inclusive,
            } => {
                if *inclusive {
                    min <= &version && &version <= max
                } else {
                    min <= &version && &version < max
                }
            }
            SemVer::Major(major) => major == &version.major,
            SemVer::MajorMinor { major, minor } => {
                major == &version.major && minor == &version.minor
            }
            SemVer::MajorMinorPatch(v) => {
                if v.major == 0 {
                    v.minor == version.minor && v.patch <= version.patch
                } else {
                    v.major == version.major && v.minor == version.minor && v.patch <= version.patch
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProgramIds {
    Mapped(HashMap<Network, NetworkKey>),
    AllNetworks(NetworkKey),
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct NetworkKey {
    #[serde(with = "serde_base58_pubkey")]
    pub key: Pubkey,
    // #[serde(with = "serde_as_option", skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}
impl From<Pubkey> for NetworkKey {
    fn from(value: Pubkey) -> Self {
        Self {
            key: value,
            extension_fields: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Network {
    MainNet,
    DevNet,
    TestNet,
    #[serde(untagged)]
    Custom(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum DiscriminantId {
    None,
    Anchor,
    U8,
    U16,
    U32,
    U64,
    U128,
    #[serde(untagged)]
    Plugin {
        plugin_id: String,
        disc_ty: String,
        // #[serde(with = "serde_as_option", skip_serializing_if = "HashMap::is_empty")]
        extension_fields: HashMap<ExtensionClass, Value>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlGeneric {
    pub name: String,
    pub description: String,
    pub generic_id: String,
    // #[serde(with = "serde_as_option", skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

// TODO: Fix this
// #[cfg(test)]
// mod test {
//     use crate::{
//         AccountId, DiscriminantId, IdlAccount, IdlAccountSet, IdlAccountSetDef,
//         idl_account_set_struct_field, IdlDefinedType, IdlDefinition, IdlField, IdlGeneric,
//         IdlRawInputAccount, IdlSeed, IdlSeedDef, IdlSeeds, IdlSeedsDef, IdlType, IdlTypeDef,
//         Network, TypeId, Version,
//     };
//     use anyhow::Result;
//     use solana_sdk::pubkey::Pubkey;
//     use std::fs::File;
//
//     #[test]
//     fn idl_example() -> Result<()> {
//         let idl = IdlDefinition {
//             idl_std_version: Version {
//                 major: 0,
//                 minor: 1,
//                 patch: 0,
//             },
//             version: Version {
//                 major: 1,
//                 minor: 0,
//                 patch: 0,
//             },
//             name: "Test Idl".to_string(),
//             description: "Test Idl Description".to_string(),
//             namespace: "@staratlas/test".to_string(),
//             program_ids: [
//                 (Network::MainNet, Pubkey::new_unique().into()),
//                 (Network::MainNet, Pubkey::default().into()),
//                 (
//                     Network::Custom("AtlasNet".to_string()),
//                     Pubkey::new_unique().into(),
//                 ),
//             ]
//             .into_iter()
//             .collect(),
//             account_discriminant: DiscriminantId::U8,
//             instruction_discriminant: DiscriminantId::U64,
//             accounts: vec![
//                 IdlAccount {
//                     name: "Test Account 1".to_string(),
//                     description: "Basic account".to_string(),
//                     id: "test1".to_string(),
//                     discriminant: serde_json::to_value(1u8)?,
//                     type_id: TypeId {
//                         namespace: None,
//                         type_id: "TestType".to_string(),
//                         provided_generics: vec![],
//                         extension_fields: Default::default(),
//                     },
//                     seeds: IdlSeeds::NotRequired { possible: vec![] },
//                     extension_fields: Default::default(),
//                 },
//                 IdlAccount {
//                     name: "Test Account 2".to_string(),
//                     description: "Account with seeds".to_string(),
//                     id: "test2".to_string(),
//                     discriminant: serde_json::to_value(2u8)?,
//                     type_id: TypeId {
//                         namespace: Some("Foreign".to_string()),
//                         type_id: "SomeType".to_string(),
//                         provided_generics: vec![],
//                         extension_fields: Default::default(),
//                     },
//                     seeds: IdlSeeds::StoredAtHead(IdlSeedsDef {
//                         discriminator: "test2".to_string(),
//                         require_find: true,
//                         seeds: vec![
//                             IdlSeed {
//                                 name: "Test 1".to_string(),
//                                 description: "Test 1 Account".to_string(),
//                                 ty: IdlSeedDef::Account {
//                                     valid_types: vec![AccountId {
//                                         namespace: None,
//                                         account_id: "test1".to_string(),
//                                         extension_fields: Default::default(),
//                                     }],
//                                 },
//                             },
//                             IdlSeed {
//                                 name: "Random Value".to_string(),
//                                 description: "Random Value to append".to_string(),
//                                 ty: IdlSeedDef::Arg {
//                                     ty: IdlDefinedType::U32,
//                                 },
//                             },
//                         ],
//                     }),
//                     extension_fields: Default::default(),
//                 },
//             ],
//             types: vec![
//                 IdlType {
//                     name: "Versioned Data".to_string(),
//                     description: "Wraps data with a version byte".to_string(),
//                     generics: vec![IdlGeneric {
//                         name: "Data".to_string(),
//                         description: "Data to wrap".to_string(),
//                         generic_id: "data".to_string(),
//                         extension_fields: Default::default(),
//                     }],
//                     type_id: "VersionedData".to_string(),
//                     type_def: IdlTypeDef::Struct(vec![IdlField {
//                         name: "Version".to_string(),
//                         description: "Version of the data".to_string(),
//                         path_id: "version".to_string(),
//                         type_def: IdlTypeDef::Defined(IdlDefinedType::U8),
//                         extension_fields: Default::default(),
//                     }]),
//                     extension_fields: Default::default(),
//                 },
//                 IdlType {
//                     name: "TestType".to_string(),
//                     description: "Test Type stuff".to_string(),
//                     generics: vec![],
//                     type_id: "TestType".into(),
//                     type_def: IdlTypeDef::IdlType(TypeId {
//                         namespace: None,
//                         type_id: "VersionedData".to_string(),
//                         provided_generics: vec![IdlTypeDef::Struct(vec![
//                             IdlField {
//                                 name: "Data1".to_string(),
//                                 description: "Data val 1".to_string(),
//                                 path_id: "data1".to_string(),
//                                 type_def: IdlTypeDef::Defined(IdlDefinedType::U64),
//                                 extension_fields: Default::default(),
//                             },
//                             IdlField {
//                                 name: "Data2".to_string(),
//                                 description: "Data val 2".to_string(),
//                                 path_id: "data2".to_string(),
//                                 type_def: IdlTypeDef::Defined(IdlDefinedType::Pubkey),
//                                 extension_fields: Default::default(),
//                             },
//                         ])],
//                         extension_fields: Default::default(),
//                     }),
//                     extension_fields: Default::default(),
//                 },
//             ],
//             account_sets: vec![IdlAccountSet {
//                 name: "Account Set 1".to_string(),
//                 description: "Account Set".to_string(),
//                 id: "set1".to_string(),
//                 type_generics: vec![],
//                 account_generics: vec![],
//                 def: IdlAccountSetDef::Struct(vec![idl_account_set_struct_field {
//                     name: "Funder".to_string(),
//                     description: "The funder for the account".to_string(),
//                     path: "funder".to_string(),
//                     account_set: IdlAccountSetDef::RawAccount(IdlRawInputAccount {
//                         possible_account_types: vec![],
//                         allow_zeroed: false,
//                         allow_uninitialized: true,
//                         signer: true,
//                         writable: true,
//                         extension_fields: Default::default(),
//                     }),
//                     extension_fields: Default::default(),
//                 }]),
//                 extension_fields: Default::default(),
//             }],
//             instructions: vec![],
//             extension_fields: [].into_iter().collect(),
//         };
//
//         // struct VersionedData<T> {
//         //     version: u8,
//         //     data: T,
//         // }
//         //
//         // type TestData = VersionedData<TestDataInner>;
//         // struct TestDataInner {
//         //     data1: u64,
//         //     data2: Pubkey,
//         // }
//
//         let path = "idl.json";
//         println!("Path: {:?}", path);
//         let file = File::create(path)?;
//         serde_json::to_writer_pretty(file, &idl)?;
//         Ok(())
//     }
// }
