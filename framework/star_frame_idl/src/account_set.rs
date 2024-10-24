use crate::account::IdlAccountId;
use crate::seeds::IdlFindSeeds;
use crate::ty::IdlTypeDef;
use crate::{IdlGeneric, ItemInfo};
use crate::{ItemDescription, ItemSource};
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
pub enum IdlAccountSetDef {
    Defined(IdlAccountSetId),
    SingleAccount(IdlSingleAccountSet),
    Signer(Box<IdlAccountSetDef>),
    Writable(Box<IdlAccountSetDef>),
    // todo: Add IdlFindSeeds to seeded
    SeededAccount(Box<IdlAccountSetDef>),
    ProgramAccount {
        account_set: Box<IdlAccountSetDef>,
        account_id: IdlAccountId,
    },
    Struct(Vec<IdlAccountSetStructField>),
    Many {
        account_set: Box<IdlAccountSetDef>,
        min: usize,
        max: Option<usize>,
    },
    /// One of the set defs in the vec
    Or(Vec<IdlAccountSetDef>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlAccountSetStructField {
    pub path: String,
    pub description: ItemDescription,
    pub account_set_def: IdlAccountSetDef,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlSingleAccountSet {
    pub program_accounts: Vec<IdlAccountId>,
    pub seeds: Option<IdlFindSeeds>,
    pub address: Option<Pubkey>,
    pub writable: bool,
    pub signer: bool,
    pub optional: bool,
}
