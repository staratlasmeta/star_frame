#!/usr/bin/env bash
cargo build-sbf --tools-version v1.43
USE_BIN=true cargo test --profile test --lib test_that_it_works -- --nocapture