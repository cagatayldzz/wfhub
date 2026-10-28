#!/bin/bash
# Builds the iPhone app for the simulator, starts it there and brings the simulator window forward.
# It uses iPhone 17 by default; set WFHUB_SIMULATOR to a name or id to pick another one.
# Arguments go to the app, for example
# `simulator.sh --example` opens level 20 right away.
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
cd "$here"

if command -v xcodegen >/dev/null; then
  xcodegen generate >/dev/null
fi

# Which simulator to use.
pick() { grep -oE '[0-9A-F]{8}-([0-9A-F]{4}-){3}[0-9A-F]{12}' | head -1; }
if [ -n "${WFHUB_SIMULATOR:-}" ]; then
  udid="$(xcrun simctl list devices available | grep -F "$WFHUB_SIMULATOR" | pick || true)"
  [ -n "$udid" ] || udid="$WFHUB_SIMULATOR"
else
  udid="$(xcrun simctl list devices available | grep -E '^ *iPhone 17 \(' | pick || true)"
fi
if [ -z "$udid" ]; then
  echo "error: iPhone 17 simulator not found; create one in Xcode or set WFHUB_SIMULATOR" >&2
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
xcrun simctl terminate "$udid" com.cagatayldzz.wfhub 2>/dev/null || true
xcrun simctl launch "$udid" com.cagatayldzz.wfhub "$@"

# The window of the simulator: Simulator in older Xcode versions, DeviceHub in newer ones.
developer="$(xcode-select -p)"
for window in "$developer/Applications/Simulator.app" "$developer/../Applications/DeviceHub.app"; do
  if [ -d "$window" ]; then
    open "$window" --args -CurrentDeviceUDID "$udid"
    break
  fi
done
