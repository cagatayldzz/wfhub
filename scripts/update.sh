#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

if ! cargo upgrade --version >/dev/null 2>&1; then
  echo "error: cargo-upgrade is required; install it with: cargo install cargo-edit --locked" >&2
  exit 1
fi

cargo upgrade --incompatible allow --pinned allow "$@"

# A dry run must not modify the lockfile either.
for arg in "$@"; do
  if [ "$arg" = "--dry-run" ] || [ "$arg" = "-n" ]; then
    exit 0
  fi
done
cargo update
