#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../apps/wfhub-mobile"
xcodegen generate "$@"
