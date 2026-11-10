#!/bin/bash
# Compiles the API Worker (Rust, WASM) once and puts the result in apps/wfhub-api/worker/dist, which is committed: the
# Worker holds no data (it serves the static files of the site), so it only has to be compiled again when its Rust code
# changes. Deploying never compiles anything.
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
