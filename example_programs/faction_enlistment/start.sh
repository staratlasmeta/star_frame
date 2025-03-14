#!/usr/bin/env bash
cargo build-sbf --tools-version v1.43
USE_BIN=true cargo test --profile test --lib banks_test -- --nocapture