use crate::serde_base58_pubkey_option;
use crate::ty::IdlTypeDef;
use crate::ItemDescription;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct IdlFindSeeds {
    pub seeds: Vec<IdlFindSeed>,
    /// The program used to find the PDA. If None, the seeds should be for the program this instruction
    /// is being called on.
    #[serde(with = "serde_base58_pubkey_option")]
    pub program: Option<Pubkey>,
}

/// The only seeds we can reliably derive are ones that only rely on constants and account keys in
/// an instruction.
///
/// Using data from accounts would require fetching and parsing the account data, which
/// we leave to the user to implement if they desire. Instruction data isn't super useful either, since finding
/// seeds is done at the `AccountSet` level, which may not have access to the top level instruction data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlFindSeed {
    /// A constant seed
    Const(Vec<u8>),
    /// A seed that is derived from an account. This is relative to the same AccountSet that the seeded account is in
    AccountPath(String),
}

#[derive(Serialize, Deserialize, Deref, DerefMut, Clone, Debug, PartialEq, Eq)]
pub struct IdlSeeds(pub Vec<IdlSeed>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlSeed {
    Const(Vec<u8>),
    Variable {
        name: String,
        description: ItemDescription,
        ty: IdlTypeDef,
    },
}
