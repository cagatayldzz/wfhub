#!/bin/bash
# Downloads the WFCD warframe-worldstate-data files the name tables are made from into apps/wfhub-api/.sources/worldstate (not committed).
set -euo pipefail
cd "$(dirname "$0")/../apps/wfhub-api"
mkdir -p .sources/worldstate
base="https://raw.githubusercontent.com/WFCD/warframe-worldstate-data/master/data"
for name in solNodes missionTypes factionsData fissureModifiers sortieData syndicatesData upgradeTypes steelPath languages; do
  curl -fsSL --retry 3 -o ".sources/worldstate/$name.json" "$base/$name.json"
  echo "$name"
done
