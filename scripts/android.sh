#!/bin/bash
# Android app (apps/wfhub-android). Needs the Android SDK (ANDROID_HOME), NDK r28+, JDK 17+, cargo-ndk and the Rust Android targets:
#   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android && cargo install cargo-ndk
#   bash scripts/android.sh build     debug APK  -> apps/wfhub-android/app/build/outputs/apk/debug
#   bash scripts/android.sh install   build and install on the connected device or emulator
#   bash scripts/android.sh release   release bundle (needs signing set up in your own Gradle properties)
set -euo pipefail
scripts="$(cd "$(dirname "$0")" && pwd)"
cd "$scripts/../apps/wfhub-android"

gradle() {
  if [ -x ./gradlew ]; then
    ./gradlew "$@"
  elif command -v gradle >/dev/null; then
    # First run: create the wrapper (gradle-wrapper.properties is committed, the jar and script are generated).
    command gradle wrapper --gradle-version 8.14.3 >/dev/null
    ./gradlew "$@"
  else
    echo "error: gradle not found; install it (brew install gradle) or open apps/wfhub-android in Android Studio" >&2
    exit 1
  fi
}

action="${1:-}"
case "$action" in
  build|install|release) bash "$scripts/build-core.sh" android ;;
esac
case "$action" in
  build) gradle assembleDebug ;;
  install) gradle installDebug ;;
  release) gradle bundleRelease ;;
  *) echo "Usage: bash scripts/android.sh {build|install|release}" >&2; exit 2 ;;
esac
