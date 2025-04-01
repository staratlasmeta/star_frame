use crate::seeds::IdlSeeds;
use crate::ty::IdlTypeId;
use crate::IdlDiscriminant;
use crate::{serde_base58_pubkey_option, ItemSource};
use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdlAccount {
    pub discriminant: IdlDiscriminant,
    pub type_id: IdlTypeId,
    pub seeds: Option<IdlSeeds>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdlAccountId {
    #[serde(with = "serde_base58_pubkey_option")]
    pub namespace: Option<Pubkey>,
    pub source: ItemSource,
}
