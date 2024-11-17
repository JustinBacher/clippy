#!/usr/bin/env bash
set -euo pipefail

cargo +nightly fmt --all
cargo check
cargo test
