use crate::seeds::IdlSeeds;
use crate::ty::IdlTypeDef;
use crate::IdlDiscriminant;
use crate::{serde_base58_pubkey_option, ItemSource};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccount {
    pub discriminant: IdlDiscriminant,
    // info should be contained in IdlTypeDef
    pub type_def: IdlTypeDef,
    pub seeds: Option<IdlSeeds>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlAccountId {
    #[serde(with = "serde_base58_pubkey_option")]
    pub namespace: Option<Pubkey>,
    pub source: ItemSource,
}
