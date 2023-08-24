mod account_with_remaining;
mod instruction_with_signers;
mod parsed_account;
mod rpc_ext;
mod token;

pub use account_with_remaining::*;
pub use instruction_with_signers::*;
pub use parsed_account::*;
pub use rpc_ext::*;
pub use token::*;

use solana_client::nonblocking::rpc_client::RpcClient;
use std::env::var;

/// Get a client for the test environment.
#[must_use]
pub fn get_client() -> RpcClient {
    let rpc_url = var("RPC_URL").unwrap_or_else(|_| "http://localhost:8899".to_string());
    RpcClient::new(rpc_url)
}
