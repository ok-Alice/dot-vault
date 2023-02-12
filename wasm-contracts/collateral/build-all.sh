#!/usr/bin/env bash

set -eu

cargo +nightly contract build --manifest-path sign-transfer/Cargo.toml
cargo +nightly contract build --manifest-path oracle/Cargo.toml
cargo +nightly contract build
