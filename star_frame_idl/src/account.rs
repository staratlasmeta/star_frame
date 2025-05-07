use crate::seeds::IdlSeeds;
use crate::ty::IdlTypeId;
use crate::IdlDiscriminant;
use crate::{IdlNamespace, ItemSource};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccount {
    pub discriminant: IdlDiscriminant,
    pub type_id: IdlTypeId,
    pub seeds: Option<IdlSeeds>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlAccountId {
    pub namespace: Option<IdlNamespace>,
    pub source: ItemSource,
}
