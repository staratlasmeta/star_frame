use itertools::Itertools;
use proc_macro2::TokenStream;
use proc_macro_error2::abort;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use syn::{punctuated::Punctuated, token::Comma, LitStr};

pub const SIGHASH_GLOBAL_NAMESPACE: &str = "global";
pub const SIGHASH_ACCOUNT_NAMESPACE: &str = "account";

pub fn hash_str(s: &str) -> [u8; 8] {
    let mut hasher = Sha256::default();
    hasher.update(s.as_bytes());

    hasher.finalize().as_slice()[0..8]
        .try_into()
        .expect("Sha256 output is 32 bytes")
}

pub fn hash_tts(hash: &[u8; 8]) -> TokenStream {
    let hash_tts = format!("{hash:?}");
    TokenStream::from_str(&hash_tts).expect("Hash should be valid tts")
}

pub fn sighash_impl(args: Punctuated<LitStr, Comma>) -> TokenStream {
    if args.is_empty() {
        abort!(args, "sighash! requires at least one argument");
    }
    let strings = args.iter().map(|s| s.value()).join(":");
    hash_tts(&hash_str(&strings))
}
