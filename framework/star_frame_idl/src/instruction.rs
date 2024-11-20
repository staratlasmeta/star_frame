use crate::account_set::IdlAccountSetDef;
use crate::ty::IdlTypeId;
use crate::IdlDiscriminant;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlInstruction {
    pub discriminant: IdlDiscriminant,
    #[serde(flatten)]
    pub definition: IdlInstructionDef,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlInstructionDef {
    pub account_set: IdlAccountSetDef,
    pub type_id: IdlTypeId,
}
