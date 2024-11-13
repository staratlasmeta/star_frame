#!/usr/bin/env bash
cargo build-sbf --tools-version v1.41
USE_BIN=true cargo test --color=always --profile test --lib tests::banks_test --no-fail-fast --all-features --manifest-path /home/sammy/star-atlas/star_frame_working/star_frame/framework/example_programs/faction_enlistment/Cargo.toml -- --nocapture