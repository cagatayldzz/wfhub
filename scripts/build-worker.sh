#!/bin/bash
# Compiles the API Worker (Rust, WASM) into apps/wfhub-api/worker/dist (not committed). wrangler runs this itself as the
# `build` command of wrangler.jsonc on every `wrangler deploy` / `wrangler dev`, so Cloudflare's build compiles the Worker.
# Cloudflare's build image has no Rust, so it is installed (minimal profile, with the WASM target) when `cargo` is missing.
set -euo pipefail
cd "$(dirname "$0")/../apps/wfhub-api/worker"

export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v cargo >/dev/null; then
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable --target wasm32-unknown-unknown
fi
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
# worker-build must match the `worker` crate of worker/Cargo.toml (0.8).
worker-build --version 2>/dev/null | grep -q "^worker-build 0.8\." || cargo install worker-build --version "^0.8" --locked --force
worker-build --release
rm -rf dist
mkdir dist
cp build/index.js build/index_bg.wasm dist/
rm -rf build
ls -la dist
