#!/bin/bash
# Archive locally for distribution through Xcode.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
cd "$here"
xcodegen generate
archive="$here/build/WFHub.xcarchive"
xcodebuild archive -project WFHub.xcodeproj -scheme WFHub \
  -destination 'generic/platform=iOS' -archivePath "$archive" "$@"
echo "Archive ready: $archive"
open "$archive"
