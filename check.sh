#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fmt --all
cargo clippy -- -D warnings
cargo test
