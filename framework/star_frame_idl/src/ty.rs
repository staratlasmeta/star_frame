use crate::{
    serde_base58_pubkey_option, IdlDefinition, IdlDiscriminant, ItemDescription, ItemSource,
};
use crate::{IdlGeneric, ItemInfo};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlType {
    #[serde(flatten)]
    pub info: ItemInfo,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub generics: Vec<IdlGeneric>,
    pub type_def: IdlTypeDef,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlTypeId {
    pub source: ItemSource,
    #[serde(with = "serde_base58_pubkey_option")]
    pub namespace: Option<Pubkey>,
    #[serde(skip_serializing_if = "crate::is_default", default)]
    pub provided_generics: Vec<IdlTypeDef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct IdlEnumVariant {
    pub name: String,
    pub discriminant: IdlDiscriminant,
    pub description: ItemDescription,
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

impl IdlTypeId {
    pub fn get_defined<'a>(
        &self,
        idl_definition: &'a IdlDefinition,
    ) -> anyhow::Result<&'a IdlType> {
        if idl_definition.types.contains_key(&self.source) {
            Ok(&idl_definition.types[&self.source])
        } else if idl_definition.external_types.contains_key(&self.source) {
            Ok(&idl_definition.external_types[&self.source])
        } else {
            bail!("Type not found in idl definition")
        }
    }
}

impl IdlTypeDef {
    pub fn assert_defined(&self) -> anyhow::Result<&IdlTypeId> {
        match self {
            IdlTypeDef::Defined(ref type_id) => Ok(type_id),
            _ => bail!("Expected defined type, found {:?}", self),
        }
    }
}

impl Default for IdlTypeDef {
    fn default() -> Self {
        Self::Struct(vec![])
    }
}
