#!/bin/bash
# Builds the shared Rust core (crates/wfhub-core) and its UniFFI bindings for a client.
#   bash scripts/build-core.sh ios      -> apps/wfhub-ios/Generated  (WFHubCore.xcframework + Sources/wfhub_core.swift)
#   bash scripts/build-core.sh android  -> apps/wfhub-android/app/src/main/{jniLibs,generated}
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
manifest="$root/crates/Cargo.toml"
target_dir="$root/crates/target"

bindgen() { # <library> <language> <out-dir>
  # Library mode reads `cargo metadata` of the working directory.
  (cd "$root/crates" && cargo run --quiet -p uniffi-bindgen -- generate --library "$1" --language "$2" --out-dir "$3" --no-format)
}

host_library() {
  cargo build --quiet --manifest-path "$manifest" -p wfhub-core
  case "$(uname)" in Darwin) echo "$target_dir/debug/libwfhub_core.dylib" ;; *) echo "$target_dir/debug/libwfhub_core.so" ;; esac
}

ios() {
  local out="$root/apps/wfhub-ios/Generated" device=aarch64-apple-ios sim=aarch64-apple-ios-sim
  for target in $device $sim; do
    cargo build --quiet --release --manifest-path "$manifest" -p wfhub-core --target "$target"
  done
  rm -rf "$out"
  mkdir -p "$out/Sources" "$out/headers"
  bindgen "$(host_library)" swift "$out/headers.tmp"
  cp "$out/headers.tmp"/*.swift "$out/Sources/"
  cp "$out/headers.tmp"/*.h "$out/headers/"
  cp "$out/headers.tmp"/*.modulemap "$out/headers/module.modulemap"
  rm -rf "$out/headers.tmp"
  xcodebuild -create-xcframework \
    -library "$target_dir/$device/release/libwfhub_core.a" -headers "$out/headers" \
    -library "$target_dir/$sim/release/libwfhub_core.a" -headers "$out/headers" \
    -output "$out/WFHubCore.xcframework" >/dev/null
  rm -rf "$out/headers"
  echo "iOS core ready: $out"
}

android() {
  command -v cargo-ndk >/dev/null || { echo "error: cargo-ndk not found; run: cargo install cargo-ndk" >&2; exit 1; }
  local app="$root/apps/wfhub-android/app/src/main"
  rm -rf "$app/jniLibs" "$app/generated"
  cargo ndk --manifest-path "$manifest" -t arm64-v8a -t armeabi-v7a -t x86_64 -o "$app/jniLibs" build --release -p wfhub-core
  bindgen "$(host_library)" kotlin "$app/generated"
  echo "Android core ready: $app"
}

case "${1:-}" in
  ios|android) "$1" ;;
  *) echo "Usage: bash scripts/build-core.sh {ios|android}" >&2; exit 2 ;;
esac
