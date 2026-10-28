#!/bin/bash
# Archives the iPhone app (the same as Xcode's Product > Archive) and opens it in Xcode's Organizer,
# ready for Distribute App > App Store Connect > Upload.
#
# Every upload needs a build number that was not used before, so the script raises
# CURRENT_PROJECT_VERSION in project.yml by one first. Pass --no-bump to keep it.
# Other arguments are passed to xcodebuild.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
cd "$here"

if ! command -v xcodegen >/dev/null; then
  echo "error: xcodegen not found; run: brew install xcodegen" >&2
  exit 1
fi

if [ "${1:-}" = "--no-bump" ]; then
  shift
else
  current="$(grep -E 'CURRENT_PROJECT_VERSION:' project.yml | grep -oE '[0-9]+' | head -1)"
  next=$((current + 1))
  sed -i '' "s/CURRENT_PROJECT_VERSION: \"$current\"/CURRENT_PROJECT_VERSION: \"$next\"/" project.yml
  echo "Build number $current -> $next"
fi

xcodegen generate

# Archives in this folder show up in the Organizer by themselves.
folder="$HOME/Library/Developer/Xcode/Archives/$(date +%Y-%m-%d)"
mkdir -p "$folder"
archive="$folder/WFHub $(date '+%Y-%m-%d, %H.%M').xcarchive"

xcodebuild archive \
  -project WFHub.xcodeproj \
  -scheme WFHub \
  -destination 'generic/platform=iOS' \
  -archivePath "$archive" \
  -allowProvisioningUpdates \
  "$@"

echo "Archive ready: $archive"
# Opening an archive starts Xcode and shows it in the Organizer.
open "$archive"
