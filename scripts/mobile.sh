#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../apps/wfhub-mobile"

archive() {
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
}

simulator() {
if command -v xcodegen >/dev/null; then
  xcodegen generate >/dev/null
fi

# Which simulator to use.
pick() { grep -oE '[0-9A-F]{8}-([0-9A-F]{4}-){3}[0-9A-F]{12}' | head -1; }
if [ -n "${WFHUB_SIMULATOR:-}" ]; then
  udid="$(xcrun simctl list devices available | grep -F "$WFHUB_SIMULATOR" | pick || true)"
  [ -n "$udid" ] || udid="$WFHUB_SIMULATOR"
else
  udid="$(xcrun simctl list devices available | grep -E '^ *iPhone 15 \(' | pick || true)"
fi
if [ -z "$udid" ]; then
  echo "error: iPhone 15 simulator not found; create one in Xcode or set WFHUB_SIMULATOR" >&2
  exit 1
fi
echo "Simulator: $(xcrun simctl list devices | grep "$udid" | sed 's/^ *//')"

xcodebuild \
  -project WFHub.xcodeproj \
  -scheme WFHub \
  -sdk iphonesimulator \
  -destination "id=$udid" \
  -derivedDataPath build/derived \
  CODE_SIGNING_ALLOWED=NO \
  -quiet \
  build

xcrun simctl boot "$udid" 2>/dev/null || true
xcrun simctl bootstatus "$udid" -b >/dev/null
xcrun simctl install "$udid" build/derived/Build/Products/Debug-iphonesimulator/WFHub.app
xcrun simctl terminate "$udid" com.wfhub.cagatayldzz 2>/dev/null || true
xcrun simctl launch "$udid" com.wfhub.cagatayldzz "$@"

# The window of the simulator: Simulator in older Xcode versions, DeviceHub in newer ones.
developer="$(xcode-select -p)"
for window in "$developer/Applications/Simulator.app" "$developer/../Applications/DeviceHub.app"; do
  if [ -d "$window" ]; then
    open "$window" --args -CurrentDeviceUDID "$udid"
    break
  fi
done
}

action="${1:-}"
case "$action" in
  archive|simulator) shift; "$action" "$@" ;;
  project) shift; xcodegen generate "$@" ;;
  *) echo "Usage: bash scripts/mobile.sh {archive|project|simulator} [options]" >&2; exit 2 ;;
esac
