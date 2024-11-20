use crate::account::IdlAccountId;
use crate::seeds::IdlFindSeeds;
use crate::ty::IdlTypeDef;
use crate::{serde_base58_pubkey_option, IdlDefinition};
use crate::{IdlGeneric, ItemInfo};
use crate::{ItemDescription, ItemSource};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlAccountSetId {
    pub source: ItemSource,
    pub provided_type_generics: Vec<IdlTypeDef>,
    pub provided_account_generics: Vec<IdlAccountSetDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccountSet {
    #[serde(flatten)]
    pub info: ItemInfo,
    pub type_generics: Vec<IdlGeneric>,
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
    pub optional: bool,
    pub is_init: bool,
    pub program_accounts: Vec<IdlAccountId>,
    pub seeds: Option<IdlFindSeeds>,
    #[serde(with = "serde_base58_pubkey_option")]
    pub address: Option<Pubkey>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlAccountSetDef {
    Defined(IdlAccountSetId),
    Single(IdlSingleAccountSet),
    Struct(Vec<IdlAccountSetStructField>),
    Many {
        account_set: Box<IdlAccountSetDef>,
        min: usize,
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
}
