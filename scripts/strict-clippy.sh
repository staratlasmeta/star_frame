#!/usr/bin/env bash

red() { echo -n $'\e[1;31m'"$1"$'\e[0m'; }
ABS_PATH=$(realpath "$1")

cargo_err="warning: profiles for the non root package will be ignored"
if (cd "$ABS_PATH" && RUSTFLAGS="-Dwarnings" cargo clippy 2>&1 | tee /dev/stderr | grep -q "$cargo_err"); then
  echo
  echo "$(red "Error"): Cargo.toml build profile detected in non root package!"
  exit 1
fi
