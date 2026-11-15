#!/bin/bash
# Compiles the API Worker (Rust, WASM) into apps/wfhub-api/worker/dist, which is committed: Cloudflare's build has no Rust
# and compiling there takes minutes, so the Worker is built here, only when its Rust code changes (check-worker.sh fails
# a deploy when it is stale). Needs rustup, `worker-build` 0.8 (matches the `worker` crate) and the WASM target.
set -euo pipefail
cd "$(dirname "$0")/../apps/wfhub-api/worker"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
worker-build --version 2>/dev/null | grep -q "^worker-build 0.8\." || cargo install worker-build --version "^0.8" --locked --force
worker-build --release
rm -rf dist
mkdir dist
cp build/index.js build/index_bg.wasm dist/
bash ../../../scripts/worker-hash.sh > dist/source.hash
rm -rf build
ls -la dist
