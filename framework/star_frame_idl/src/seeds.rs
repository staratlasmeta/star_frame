use crate::account::AccountId;
use crate::ty::IdlDefinedType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IdlSeeds {
    StoredAtHead(IdlSeedsDef),
    NotRequired { possible: Vec<IdlSeedsDef> },
    Plugin { plugin_id: String, seeds: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlSeedsDef {
    pub discriminator: String,
    /// Marker for seeded accounts that are required to be found (largest possible bump)
    pub require_find: bool,
    pub seeds: Vec<IdlSeed>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlSeed {
    pub name: String,
    pub description: String,
    pub ty: IdlSeedDef,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IdlSeedDef {
    /// UTF-8 encoded.
    Literal(String),
    Bytes(Vec<u8>),
    Account {
        valid_types: Vec<AccountId>,
    },
    Arg {
        ty: IdlDefinedType,
    },
    Plugin {
        plugin_id: String,
        seed: String,
    },
}
