use crate::{
    IdlDefinition, IdlDiscriminant, IdlGeneric, IdlNamespace, ItemDescription, ItemInfo,
    ItemSource, Result,
};
use serde::{Deserialize, Serialize};

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
    pub namespace: Option<IdlNamespace>,
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
    Address,
    FixedPoint {
        ty: Box<IdlTypeDef>,
        frac: u8,
    },
    Option {
        ty: Box<IdlTypeDef>,
        /// Whether or not the enum is fixed size (null will be padded with zeros if so)
        fixed: bool,
    },
    RemainingBytes,
    List {
        len_ty: Box<IdlTypeDef>,
        item_ty: Box<IdlTypeDef>,
    },
    UnsizedList {
        len_ty: Box<IdlTypeDef>,
        offset_ty: Box<IdlTypeDef>,
        item_ty: Box<IdlTypeDef>,
    },
    Set {
        len_ty: Box<IdlTypeDef>,
        item_ty: Box<IdlTypeDef>,
    },
    Map {
        len_ty: Box<IdlTypeDef>,
        key_ty: Box<IdlTypeDef>,
        value_ty: Box<IdlTypeDef>,
    },
    Array(Box<IdlTypeDef>, usize),
    Struct(Vec<IdlStructField>),
    Enum {
        size: Box<IdlTypeDef>,
        variants: Vec<IdlEnumVariant>,
    },
}

impl IdlTypeId {
    pub fn get_defined<'a>(&self, idl_definition: &'a IdlDefinition) -> Result<&'a IdlType> {
        idl_definition
            .get_type(&self.source)
            .ok_or_else(|| crate::Error::TypeNotFound(self.source.clone()))
    }
}

impl IdlTypeDef {
    pub fn assert_defined(&self) -> Result<&IdlTypeId> {
        match self {
            IdlTypeDef::Defined(ref type_id) => Ok(type_id),
            _ => Err(crate::Error::InvalidTypeDef {
                expected: "defined type".to_string(),
                found: format!("{:?}", self),
            }),
        }
    }
}

impl Default for IdlTypeDef {
    fn default() -> Self {
        Self::Struct(vec![])
    }
}
