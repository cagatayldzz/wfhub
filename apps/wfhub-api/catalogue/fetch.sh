#!/bin/bash
# Downloads the WFCD warframe-items files the catalogue is made from into apps/wfhub-api/catalogue/sources (not committed).
set -euo pipefail
cd "$(dirname "$0")"
mkdir -p sources
base="https://raw.githubusercontent.com/WFCD/warframe-items/master/data/json"
for name in Arcanes Arch-Gun Arch-Melee Archwing Enemy Fish Gear Glyphs Melee Misc Mods Node Pets Primary Quests Railjack Relics \
  Resources Secondary Sentinels SentinelWeapons Sigils Skins Warframes; do
  curl -fsSL --retry 3 -o "sources/$name.json" "$base/$name.json"
  echo "$name"
done
