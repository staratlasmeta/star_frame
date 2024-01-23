use crate::account::AccountId;
// use crate::serde_impls::serde_as_option;
use crate::ty::IdlTypeDef;
use crate::{ExtensionClass, IdlGeneric};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountSetId {
    pub namespace: Option<String>,
    pub account_set_id: String,
    pub provided_type_generics: Vec<IdlTypeDef>,
    pub provided_account_generics: Vec<IdlAccountSetDef>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccountSet {
    pub name: String,
    pub description: String,
    pub type_generics: Vec<IdlGeneric>,
    pub account_generics: Vec<IdlGeneric>,
    pub def: IdlAccountSetDef,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IdlAccountSetDef {
    AccountSet(AccountSetId),
    AccountInfo,
    DataAccount {
        account: AccountId,
        init: Init,
    },
    Signer(Box<IdlAccountSetDef>),
    Writable(Box<IdlAccountSetDef>),
    Struct(Vec<IdlAccountSetStructField>),
    Many {
        account: Box<IdlAccountSetDef>,
        min: usize,
        max: Option<usize>,
    },
    /// One of the set defs in the vec
    Or(Vec<IdlAccountSetDef>),
    Generic {
        account_generic_id: String,
    },
    #[serde(untagged)]
    Plugin {
        plugin_id: String,
        account_set: String,
        provided_type_generics: Vec<IdlTypeDef>,
        provided_account_generics: Vec<IdlAccountSetDef>,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        extension_fields: HashMap<ExtensionClass, Value>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccountSetStructField {
    pub name: String,
    pub description: String,
    pub path: String,
    pub account_set: IdlAccountSetDef,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlRawInputAccount {
    pub possible_account_types: Vec<AccountId>,
    pub allow_zeroed: bool,
    pub allow_uninitialized: bool,
    pub signer: bool,
    pub writable: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extension_fields: HashMap<ExtensionClass, Value>,
}
