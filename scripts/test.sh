#!/bin/bash
set -euo pipefail

bunx tsc --noEmit
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
