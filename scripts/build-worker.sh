#!/bin/bash
# Compiles the API Worker (Rust, WASM) into apps/wfhub-api/worker/dist (not committed). wrangler runs this itself as the
# `build` command of wrangler.jsonc on every `wrangler deploy` / `wrangler dev`, so Cloudflare's build compiles the Worker.
set -euo pipefail
cd "$(dirname "$0")/../apps/wfhub-api/worker"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
command -v worker-build >/dev/null || cargo install worker-build
worker-build --release
rm -rf dist
mkdir dist
cp build/index.js build/index_bg.wasm dist/
rm -rf build
ls -la dist
