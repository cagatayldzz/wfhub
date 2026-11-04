#!/bin/bash
set -euo pipefail

cd "$(dirname "$0")/.."
bash scripts/cf-build.sh
npx wrangler deploy "$@"
