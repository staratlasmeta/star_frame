use crate::account_set::IdlAccountSetDef;
// use crate::serde_impls::serde_as_option;
use crate::ty::IdlTypeDef;
use crate::ExtensionClass;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlInstruction {
    pub name: String,
    pub description: String,
    pub discriminant: Value,
    pub definition: IdlInstructionDef,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlInstructionDef {
    pub account_set: IdlAccountSetDef,
    pub data: IdlTypeDef,
}
