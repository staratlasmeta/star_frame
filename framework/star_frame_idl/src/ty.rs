use crate::account::IdlAccountId;
use crate::{serde_base58_pubkey_option, IdlDiscriminant, ItemDescription, ItemSource};
use crate::{IdlGeneric, ItemInfo};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlType {
    #[serde(flatten)]
    pub info: ItemInfo,
    pub generics: Vec<IdlGeneric>,
    pub type_def: IdlTypeDef,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlTypeId {
    pub source: ItemSource,
    #[serde(with = "serde_base58_pubkey_option")]
    pub namespace: Option<Pubkey>,
    pub provided_generics: Vec<IdlTypeDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct IdlEnumVariant {
    pub name: String,
    pub discriminant: IdlDiscriminant,
    pub type_def: Option<IdlTypeDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlStructField {
    pub path: Option<String>,
    pub description: ItemDescription,
    pub type_def: IdlTypeDef,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlTypeDef {
    Defined(IdlTypeId),
    Generic(String),
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
    U64,
    I64,
    F64,
    U128,
    I128,
    String,
    Pubkey,
    OptionalPubkey,
    PubkeyFor {
        id: IdlAccountId,
        optional: bool,
    },
    FixedPoint {
        ty: Box<IdlTypeDef>,
        frac: u8,
    },
    Option(Box<IdlTypeDef>),
    List {
        len_ty: Box<IdlTypeDef>,
        item_ty: Box<IdlTypeDef>,
    },
    Array(Box<IdlTypeDef>, usize),
    Struct(Vec<IdlStructField>),
    Enum(Vec<IdlEnumVariant>),
}

impl IdlTypeDef {
    pub fn assert_defined(&self) -> anyhow::Result<&IdlTypeId> {
        match self {
            IdlTypeDef::Defined(ref type_id) => Ok(type_id),
            _ => bail!("Expected defined type"),
        }
    }
}

impl Default for IdlTypeDef {
    fn default() -> Self {
        Self::Struct(vec![])
    }
}
