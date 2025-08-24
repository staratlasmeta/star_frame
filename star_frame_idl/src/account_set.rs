use crate::{
    account::IdlAccountId, seeds::IdlFindSeeds, serde_base58_pubkey_option, ty::IdlTypeDef,
    IdlDefinition, IdlGeneric, ItemDescription, ItemInfo, ItemSource,
};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlAccountSetId {
    pub source: ItemSource,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub provided_type_generics: Vec<IdlTypeDef>,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub provided_account_generics: Vec<IdlAccountSetDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccountSet {
    #[serde(flatten)]
    pub info: ItemInfo,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub type_generics: Vec<IdlGeneric>,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub account_generics: Vec<IdlGeneric>,
    pub account_set_def: IdlAccountSetDef,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlAccountSetStructField {
    pub path: Option<String>,
    pub description: ItemDescription,
    pub account_set_def: IdlAccountSetDef,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct IdlSingleAccountSet {
    pub writable: bool,
    pub signer: bool,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub optional: bool,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub is_init: bool,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub program_accounts: Vec<IdlAccountId>,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub seeds: Option<IdlFindSeeds>,
    #[serde(
        with = "serde_base58_pubkey_option",
        skip_serializing_if = "crate::is_default",
        default
    )]
    pub address: Option<Pubkey>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlAccountSetDef {
    Defined(IdlAccountSetId),
    Single(IdlSingleAccountSet),
    Struct(Vec<IdlAccountSetStructField>),
    Many {
        account_set: Box<IdlAccountSetDef>,
        /// Minimum number of accounts, inclusive
        min: usize,
        /// Maximum number of accounts, inclusive. None means unbounded.
        max: Option<usize>,
    },
    /// One of the set defs in the vec
    Or(Vec<IdlAccountSetDef>),
}

impl IdlAccountSetDef {
    pub fn assert_defined(&self) -> anyhow::Result<&IdlAccountSetId> {
        match self {
            IdlAccountSetDef::Defined(id) => Ok(id),
            _ => bail!("Expected defined account set, found {:?}", self),
        }
    }

    pub fn get_defined<'a>(
        &self,
        idl_definition: &'a IdlDefinition,
    ) -> anyhow::Result<&'a IdlAccountSet> {
        let source = &self.assert_defined()?.source;
        idl_definition
            .account_sets
            .get(source)
            .ok_or_else(|| anyhow::anyhow!("Account set `{source}` not found in definition"))
    }

    pub fn empty_struct() -> Self {
        IdlAccountSetDef::Struct(vec![])
    }

    pub fn single(&mut self) -> anyhow::Result<&mut IdlSingleAccountSet> {
        match self {
            IdlAccountSetDef::Single(s) => Ok(s),
            set => bail!("Expected single account, found {:?}", set),
        }
    }

    pub fn assert_single(mut self) -> anyhow::Result<Self> {
        self.single()?;
        Ok(self)
    }

    pub fn with_single_address(mut self, address: Pubkey) -> anyhow::Result<Self> {
        let single = self.single()?;
        if let Some(old_address) = single.address {
            eprintln!("Warning: Overwriting address `{old_address}` in single account set with address `{address}`");
        }
        self.single()?.address = Some(address);
        Ok(self)
    }
}
