use anyhow::Result;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{Signature, Signer, SignerError};
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;
use std::collections::HashSet;
use std::convert::Infallible;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

/// A dynamic signer.
pub type DynSigner<'a> = dyn Signer + Send + Sync + 'a;

/// A keypair ref that can be hashed.
#[repr(transparent)]
pub struct HashableSigner<'a>(pub &'a DynSigner<'a>);
impl<'a> HashableSigner<'a> {
    /// Get the inner keypair.
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn into_inner(self) -> &'a DynSigner<'a> {
        self.0
    }
}
impl<'a> Hash for HashableSigner<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.pubkey().hash(state);
    }
}
impl<'a> PartialEq for HashableSigner<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.pubkey() == other.0.pubkey()
    }
}
impl<'a> Eq for HashableSigner<'a> where Pubkey: Eq {}
impl<'a> Debug for HashableSigner<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.pubkey().fmt(f)
    }
}
impl<'a> From<&'a DynSigner<'a>> for HashableSigner<'a> {
    fn from(value: &'a DynSigner<'a>) -> Self {
        Self(value)
    }
}
impl<'a, S> From<&'a S> for HashableSigner<'a>
where
    S: 'a + Signer + Send + Sync,
{
    fn from(value: &'a S) -> Self {
        Self(value)
    }
}

/// A set of signers.
#[derive(Debug)]
pub struct DynSignerSet<'a>(HashSet<HashableSigner<'a>>);
impl<'a> From<HashSet<HashableSigner<'a>>> for DynSignerSet<'a> {
    fn from(value: HashSet<HashableSigner<'a>>) -> Self {
        Self(value)
    }
}
impl<'a> Signers for DynSignerSet<'a> {
    fn pubkeys(&self) -> Vec<Pubkey> {
        self.0.iter().map(|h| h.0.pubkey()).collect()
    }

    fn try_pubkeys(&self) -> std::result::Result<Vec<Pubkey>, SignerError> {
        self.0.iter().map(|h| h.0.try_pubkey()).collect()
    }

    fn sign_message(&self, message: &[u8]) -> Vec<Signature> {
        self.0.iter().map(|h| h.0.sign_message(message)).collect()
    }

    fn try_sign_message(&self, message: &[u8]) -> std::result::Result<Vec<Signature>, SignerError> {
        self.0
            .iter()
            .map(|h| h.0.try_sign_message(message))
            .collect()
    }

    fn is_interactive(&self) -> bool {
        self.0.iter().any(|h| h.0.is_interactive())
    }
}

/// An instruction with the signers required to execute it.
#[derive(Debug)]
pub struct InstructionWithSigners<'a> {
    /// The dummy funder of the instruction to be replaced.
    pub dummy_funder: Pubkey,
    /// The instruction to execute.
    pub ix: Instruction,
    /// The signers required to execute the instruction.
    pub signers: HashSet<HashableSigner<'a>>,
}
impl<'a> InstructionWithSigners<'a> {
    /// Try to build an instruction with signers with custom error type.
    pub fn try_build_with_err<I, E>(
        builder: impl FnOnce(Pubkey) -> Result<(Instruction, I), E>,
    ) -> Result<Self, E>
    where
        I: IntoIterator<Item = &'a DynSigner<'a>>,
    {
        let dummy_funder = Pubkey::new_unique();
        let (ix, signers) = builder(dummy_funder)?;

        Ok(Self {
            dummy_funder,
            ix,
            signers: signers.into_iter().map(Into::into).collect(),
        })
    }

    /// Build an instruction with signers.
    pub fn build<I>(builder: impl FnOnce(Pubkey) -> (Instruction, I)) -> Self
    where
        I: IntoIterator<Item = &'a DynSigner<'a>>,
    {
        Self::try_build_with_err::<_, Infallible>(|dummy_funder| Ok(builder(dummy_funder))).unwrap()
    }

    /// Try to build an instruction with signers.
    pub fn try_build<I>(builder: impl FnOnce(Pubkey) -> Result<(Instruction, I)>) -> Self
    where
        I: IntoIterator<Item = &'a DynSigner<'a>>,
    {
        Self::try_build_with_err(builder).unwrap()
    }
}

/// Builds a transaction from instructions and a provided recent blockhash
pub fn build_transaction<'a>(
    instructions: impl IntoIterator<Item = InstructionWithSigners<'a>>,
    funder: &'a DynSigner,
    recent_blockhash: solana_program::hash::Hash,
) -> Transaction {
    let ixs = instructions.into_iter();
    let mut instructions = Vec::with_capacity(ixs.size_hint().0);
    let mut signers = HashSet::new();
    signers.insert(HashableSigner(funder));

    for ix in ixs {
        let mut ix: InstructionWithSigners = ix;
        ix.ix.accounts.iter_mut().for_each(|acc| {
            if acc.pubkey == ix.dummy_funder {
                acc.pubkey = funder.pubkey();
            }
        });
        instructions.push(ix.ix);
        signers.extend(ix.signers);
    }

    Transaction::new_signed_with_payer(
        &instructions,
        Some(&funder.pubkey()),
        &DynSignerSet::from(signers),
        recent_blockhash,
    )
}
