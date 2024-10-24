use crate::account_set::IdlAccountSetDef;
use crate::ty::IdlTypeDef;
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
    /// The data the instruction expects. This should be an IdlTypeDef::Defined. The
    /// information about the instruction should be in the IdlTypeDef::Defined documentation.
    pub definition: IdlTypeDef,
}
