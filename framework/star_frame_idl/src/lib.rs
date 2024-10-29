extern crate alloc;
extern crate core;

pub mod account;
pub mod account_set;
pub mod instruction;
pub mod seeds;
pub mod serde_impls;
pub mod ty;
#[cfg(feature = "verifier")]
pub mod verifier;

use crate::instruction::IdlInstructionDef;
use crate::serde_impls::serde_base58_pubkey;
use crate::serde_impls::serde_base58_pubkey_option;
use account::IdlAccount;
use account_set::IdlAccountSet;
use instruction::IdlInstruction;
pub use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use std::any::type_name;
use std::collections::HashMap;
use std::fmt::Debug;
use ty::IdlType;

pub fn idl_spec_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("Invalid package version. This should never happen.")
}

pub type IdlDiscriminant = Vec<u8>;

/// A source of an item in the IDL, found using the `item_source` function
pub type ItemSource = String;
pub type IdlNamespace = Pubkey;
pub type ItemDescription = Vec<String>;

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
    /// Version of program
    pub version: Version,
    // todo: crate metadata?
    // #[serde(flatten)]
    // pub crate_metadata: CrateMetadata,
    // todo: figure out required_idl_definitions
    pub required_idl_definitions: HashMap<String, IdlDefinitionReference>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CrateMetadata {
    pub name: String,
    pub description: Option<String>,
    pub repository: Option<String>,
}

impl Default for IdlMetadata {
    fn default() -> Self {
        Self {
            version: Version::new(0, 0, 0),
            // crate_metadata: CrateMetadata {
            //     name: String::default(),
            //     description: None,
            //     repository: None,
            // },
            required_idl_definitions: HashMap::default(),
            idl_spec: idl_spec_version(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct IdlDefinition {
    #[serde(with = "serde_base58_pubkey")]
    pub address: Pubkey,
    pub metadata: IdlMetadata,
    pub instructions: HashMap<ItemSource, IdlInstruction>,
    pub account_sets: HashMap<ItemSource, IdlAccountSet>,
    pub accounts: HashMap<ItemSource, IdlAccount>,
    pub types: HashMap<ItemSource, IdlType>,
    pub external_types: HashMap<ItemSource, IdlType>,
}

impl IdlDefinition {
    pub fn add_instruction(
        &mut self,
        definition: IdlInstructionDef,
        discriminant: IdlDiscriminant,
    ) -> anyhow::Result<()> {
        let source = definition.data.assert_defined()?.source.clone();
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
        namespace: Pubkey,
    ) -> anyhow::Result<Option<IdlNamespace>> {
        let source = account.type_def.assert_defined()?.source.clone();
        if namespace == self.address {
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
        if namespace == self.address {
            self.types.entry(source).or_insert(ty);
            None
        } else {
            self.external_types.entry(source).or_insert(ty);
            Some(namespace)
        }
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
// todo: figure out what to serde is_default
#[allow(dead_code)]
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
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
