#!/bin/bash
# Prints a checksum of everything the compiled API Worker is built from (the Rust sources and Cargo files of the API and
# of the Worker). build-worker.sh stores it next to the compiled Worker; check-worker.sh compares it.
set -euo pipefail
cd "$(dirname "$0")/../apps/wfhub-api"
export LC_ALL=C
{ find src worker/src -name '*.rs' | sort; echo Cargo.toml; echo Cargo.lock; echo worker/Cargo.toml; echo worker/Cargo.lock; } | xargs cat | cksum | cut -d' ' -f1
