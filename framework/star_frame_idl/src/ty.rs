use crate::account::AccountId;
// use crate::serde_impls::serde_as_option;
use crate::{ExtensionClass, IdlGeneric};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeId {
    pub namespace: Option<String>,
    pub type_id: String,
    pub provided_generics: Vec<IdlTypeDef>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlType {
    pub name: String,
    pub description: String,
    pub generics: Vec<IdlGeneric>,
    pub type_def: IdlTypeDef,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlTypeDef {
    IdlType(TypeId),
    Generic {
        generic_id: String,
    },
    Defined(IdlDefinedType),
    PubkeyFor {
        valid_account_types: Vec<AccountId>,
    },
    Array {
        item_ty: Box<IdlTypeDef>,
        size: usize,
    },
    BorshVec {
        item_ty: Box<IdlTypeDef>,
        len_ty: Box<IdlTypeDef>,
    },
    BorshOption {
        item_ty: Box<IdlTypeDef>,
    },
    Struct(Vec<IdlField>),
    #[serde(untagged)]
    Plugin {
        plugin_id: String,
        ty: String,
        provided_generics: Vec<IdlTypeDef>,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        extension_fields: HashMap<ExtensionClass, Value>,
    },
}
impl Default for IdlTypeDef {
    fn default() -> Self {
        Self::Struct(vec![])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlField {
    pub name: String,
    pub description: String,
    pub path_id: String,
    pub type_def: IdlTypeDef,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum IdlDefinedType {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    BorshBool,
    BorshString,
}
