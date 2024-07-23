use proc_macro2::TokenStream;
use sha2::{Digest, Sha256};
use std::str::FromStr;

pub const SIGHASH_GLOBAL_NAMESPACE: &str = "global";

// Anchor's sighash function
pub fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut hasher = Sha256::default();
    hasher.update(preimage.as_bytes());

    hasher.finalize().as_slice()[0..8]
        .try_into()
        .expect("Sha256 output is 32 bytes")
}

pub fn hash_tts(hash: &[u8; 8]) -> TokenStream {
    let hash_tts = format!("{:?}", hash);
    TokenStream::from_str(&hash_tts).expect("Hash should be valid tts")
}
