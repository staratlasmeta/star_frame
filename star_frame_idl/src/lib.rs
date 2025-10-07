extern crate alloc;
extern crate codama_nodes;
extern crate core;

mod codama;
pub use codama::*;
pub mod account;
pub mod account_set;
pub mod instruction;
pub mod seeds;
pub mod serde_impls;
pub mod ty;
#[cfg(feature = "verifier")]
pub mod verifier;

use crate::{
    instruction::IdlInstructionDef,
    serde_impls::{serde_base58_address, serde_base58_address_option},
};
use account::IdlAccount;
use account_set::IdlAccountSet;
use instruction::IdlInstruction;
pub use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use solana_address::Address;
use std::{any::type_name, collections::BTreeMap};
use ty::IdlType;

pub fn idl_spec_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("Invalid package version. This should never happen.")
}

pub type IdlDiscriminant = Vec<u8>;

pub type Result<T> = std::result::Result<T, Error>;

/// A source of an item in the IDL, found using the `item_source` function
pub type ItemSource = String;
pub type ItemDescription = Vec<String>;
pub type IdlNamespace = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemInfo {
    pub name: String,
    #[serde(skip)]
    pub source: ItemSource,
    pub description: ItemDescription,
}

impl ItemInfo {
    pub fn new<T>(name: &str, description: ItemDescription) -> Self {
        Self {
            name: name.into(),
            description,
            source: item_source::<T>(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlMetadata {
    /// Version of the `IdlDefinition`
    pub idl_spec: Version,
    #[serde(flatten)]
    pub crate_metadata: CrateMetadata,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    // todo: figure out required_idl_definitions
    pub required_idl_definitions: BTreeMap<String, IdlDefinitionReference>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CrateMetadata {
    /// Version of the program
    pub version: Version,
    /// Name of the program
    pub name: String,
    pub docs: ItemDescription,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub repository: Option<String>,
}

impl Default for CrateMetadata {
    fn default() -> Self {
        Self {
            version: Version::new(0, 0, 0),
            name: String::new(),
            docs: Vec::new(),
            description: None,
            homepage: None,
            license: None,
            repository: None,
        }
    }
}

impl Default for IdlMetadata {
    fn default() -> Self {
        Self {
            idl_spec: idl_spec_version(),
            crate_metadata: Default::default(),
            required_idl_definitions: BTreeMap::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IdlDefinition {
    #[serde(with = "serde_base58_address")]
    pub address: Address,
    pub metadata: IdlMetadata,
    pub instructions: BTreeMap<ItemSource, IdlInstruction>,
    pub account_sets: BTreeMap<ItemSource, IdlAccountSet>,
    pub accounts: BTreeMap<ItemSource, IdlAccount>,
    pub types: BTreeMap<ItemSource, IdlType>,
    pub external_types: BTreeMap<ItemSource, IdlType>,
    pub errors: Vec<ErrorNode>,
}

impl IdlDefinition {
    pub fn namespace(&self) -> IdlNamespace {
        self.metadata.crate_metadata.name.clone()
    }

    pub fn add_instruction(
        &mut self,
        definition: IdlInstructionDef,
        discriminant: IdlDiscriminant,
    ) -> Result<()> {
        let source = definition.type_id.source.clone();
        let idl_instruction = IdlInstruction {
            definition,
            discriminant,
        };
        self.instructions.entry(source).or_insert(idl_instruction);
        Ok(())
    }

    pub fn add_account(
        &mut self,
        account: IdlAccount,
        namespace: IdlNamespace,
    ) -> Result<Option<IdlNamespace>> {
        let source = account.type_id.source.clone();
        if namespace == self.namespace() {
            self.accounts.entry(source).or_insert(account);
            Ok(None)
        } else {
            Ok(Some(namespace))
        }
    }

    pub fn add_account_set(&mut self, set: IdlAccountSet) {
        let item_source = set.info.source.clone();
        self.account_sets.entry(item_source).or_insert(set);
    }

    pub fn add_type(&mut self, ty: IdlType, namespace: IdlNamespace) -> Option<IdlNamespace> {
        let source = ty.info.source.clone();
        if namespace == self.namespace() {
            self.types.entry(source).or_insert(ty);
            None
        } else {
            self.external_types.entry(source).or_insert(ty);
            Some(namespace)
        }
    }

    pub fn get_type(&self, source: &ItemSource) -> Option<&IdlType> {
        self.types
            .get(source)
            .or_else(|| self.external_types.get(source))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlDefinitionReference {
    pub version: Version,
    // todo: package name here too?
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlGeneric {
    pub name: String,
    pub description: String,
    pub generic_id: String,
}

/// Gets the type name stripped of generics
#[must_use]
pub fn item_source<T: ?Sized>() -> String {
    let mut to_return = String::new();
    let mut open_count = 0;
    for char in type_name::<T>().chars() {
        if char == '<' {
            open_count += 1;
        }
        if open_count == 0 {
            to_return.push(char);
        }
        if char == '>' {
            open_count -= 1;
        }
        assert!(open_count >= 0, "Mismatched generics in type name");
    }

    to_return
}

// Serde helper function
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid discriminant: {0}")]
    InvalidDiscriminant(String),
    #[error("Invalid type definition: expected {expected}, found {found}")]
    InvalidTypeDef { expected: String, found: String },
    #[error("Failed to convert to Codama: {0}")]
    CodamaConversion(String),
    #[error("Expected defined account set, found {0}")]
    ExpectedDefinedAccountSet(String),
    #[error("Expected single account set, found {0}")]
    ExpectedSingleAccountSet(String),
    #[error("Type not found in IDL definition: {0}")]
    TypeNotFound(String),
    #[error("Account set not found in IDL definition: {0}")]
    AccountSetNotFound(String),
    #[error("Missing name on named field for struct")]
    MissingNameOnNamedField,
    #[error("All 'Many' account sets must come at the end of the instruction's AccountSet")]
    ManyAccountSetsMustComeLast,
    #[error("Remaining accounts cannot have default values: {0}")]
    RemainingAccountsCannotHaveDefaults(String),
    #[error("Generic types are not supported in Codama")]
    GenericTypesNotSupported,
    #[error("IDL type definition not yet supported for enum variants: {0}")]
    UnsupportedEnumVariantType(String),
    #[error("Discriminant is too large. Max length: {0}")]
    DiscriminantTooLarge(usize),
    #[error("Expected number type node, found {0}")]
    ExpectedNumberTypeNode(String),
    #[error("Only struct account types are supported in Codama at the moment. Found: {0}")]
    UnsupportedAccountType(String),
    #[error("Only struct account sets are supported with Codama: {0}")]
    UnsupportedAccountSetType(String),
    #[error("Many sets must be made of single sets for Codama")]
    ManySetsMustBeSingle,
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Custom Error: {0}")]
    Custom(String),
}

#[cfg(test)]
mod test {
    use crate::idl_spec_version;

    /// Tests that the idl_spec_version function doesn't panic
    #[test]
    fn test_idl_spec_version() {
        idl_spec_version();
    }

    // todo: add example idl maybe?
}
