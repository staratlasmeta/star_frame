use crate::seeds::IdlSeeds;
// use crate::serde_impls::serde_as_option;
use crate::ty::IdlTypeDef;
use crate::ExtensionClass;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccount {
    pub name: String,
    pub description: String,
    pub discriminant: Value,
    pub ty: IdlTypeDef,
    pub seeds: IdlSeeds,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountId {
    pub namespace: Option<String>,
    pub account_id: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}
