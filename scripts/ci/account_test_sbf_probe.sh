#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

toolchain_release="${SBF_TOOLCHAIN_RELEASE:-3.0.8}"
sh -c "$(curl -sSfL "https://release.anza.xyz/v${toolchain_release}/install")"

solana_bin_dir="$HOME/.local/share/solana/install/active_release/bin"
export PATH="$solana_bin_dir:$PATH"

solana_version="$(solana --version)"
echo "$solana_version"
if [[ "$solana_version" != *"${toolchain_release}"* ]]; then
  echo "Expected Solana CLI version to contain '${toolchain_release}'" >&2
  exit 1
fi

export CARGO_TARGET_DIR="${repo_root}/target/account_test_sbf_probe"
cargo build-sbf --manifest-path example_programs/account_test/Cargo.toml --features probe_ix

export SBF_OUT_DIR="${CARGO_TARGET_DIR}/deploy"
if [[ ! -f "${SBF_OUT_DIR}/account_test.so" ]]; then
  echo "Missing ${SBF_OUT_DIR}/account_test.so after cargo build-sbf" >&2
  exit 1
fi

cargo test -p account_test --features probe_ix tests::borsh_probe_inner_uninitialized_ix -- --exact --nocapture
cargo test -p account_test --features probe_ix tests::borsh_probe_inner_mut_uninitialized_ix -- --exact --nocapture
cargo test -p account_test --features probe_ix tests::borsh_probe_inner_mut_non_writable_ix -- --exact --nocapture
