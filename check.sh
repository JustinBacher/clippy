#!/usr/bin/env bash
set -euo pipefail

cargo check --verbose
cargo +nightly fmt --all -- --check --verbose
cargo test --verbose
