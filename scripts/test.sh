#!/bin/bash
set -euo pipefail

bunx tsc --noEmit
bun test apps/wfhub-web/api
