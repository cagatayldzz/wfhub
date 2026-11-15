#!/bin/bash
# Fails when the committed Worker (apps/wfhub-api/worker/dist) is older than its Rust sources, so a deploy never ships a
# stale Worker. Fix: `npm run api:build` and commit apps/wfhub-api/worker/dist.
set -euo pipefail
cd "$(dirname "$0")/.."
dist=apps/wfhub-api/worker/dist
if [ ! -f "$dist/index_bg.wasm" ] || [ ! -f "$dist/source.hash" ] || [ "$(cat "$dist/source.hash")" != "$(bash scripts/worker-hash.sh)" ]; then
  echo "error: $dist is missing or older than the Rust sources. Run 'npm run api:build' and commit $dist." >&2
  exit 1
fi
echo "worker dist is up to date"
