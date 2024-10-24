use crate::ty::IdlTypeDef;
use crate::ItemDescription;
use serde::{Deserialize, Serialize};

/// The only seeds we can reliably derive are ones that only rely on constants and account keys in
/// an instruction.
///
/// Using data from accounts would require fetching and parsing the account data, which
/// we leave to the user to implement if they desire. Instruction data isn't super useful either, since finding
/// seeds is done at the `AccountSet` level, which may not have access to the top level instruction data.
/*
todo:
 figure out how to integrate this into the AccountSet macro idl step. Potentially derive a new struct
 ex:
    #[derive(Debug, GetSeeds, Clone)]
    #[seed_const(b"TEST_CONST")]
    pub struct TestAccount {
        key: Pubkey,
    }
    That could create a TestAccountIdlFindSeeds struct that takes in Strings for all the paths and impls a trait that converts that into a
    vec of IdlFindSeeds. This could be converted to Anchor's `IdlPda` struct, and should be able to achieve similar functionality.
*/
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlFindSeeds {
    /// A constant seed
    Const(Vec<u8>),
    /// A seed that is derived from an account. This is relative to the same AccountSet that the seeded account is in
    AccountPath(String),
}

/*
todo:
  GetSeeds should derive
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlSeeds(pub Vec<IdlSeed>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum IdlSeed {
    Const(Vec<u8>),
    Variable(IdlVariableSeed),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IdlVariableSeed {
    pub name: String,
    pub description: ItemDescription,
    pub ty: IdlTypeDef,
}
