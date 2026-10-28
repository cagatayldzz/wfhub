#!/bin/bash
# Builds the Rust core (crates/ffi) for the SDK Xcode is building for and puts
# libwfhub_ffi.a in build/rust/<platform>/, where project.yml tells the linker to look.
# Xcode runs this before compiling the Swift code; it can also be run by hand.
set -euo pipefail

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

here="$(cd "$(dirname "$0")/.." && pwd)"
root="$(cd "$here/../.." && pwd)"

platform="${PLATFORM_NAME:-iphonesimulator}"
archs="${ARCHS:-arm64}"

case "$platform" in
  iphoneos) triples=(aarch64-apple-ios) ;;
  iphonesimulator)
    triples=()
    for arch in $archs; do
      case "$arch" in
        arm64) triples+=(aarch64-apple-ios-sim) ;;
        x86_64) triples+=(x86_64-apple-ios) ;;
      esac
    done
    ;;
  *)
    echo "error: unsupported platform $platform" >&2
    exit 1
    ;;
esac

if ! command -v cargo >/dev/null; then
  echo "error: cargo not found; install Rust from https://rustup.rs" >&2
  exit 1
fi

# Xcode points these at the iOS SDK, which would break the build scripts of
# dependencies that run on the Mac.
unset SDKROOT LIBRARY_PATH
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-17.0}"

libs=()
for triple in "${triples[@]}"; do
  if ! rustup target list --installed | grep -qx "$triple"; then
    echo "error: Rust target $triple is missing; run: rustup target add $triple" >&2
    exit 1
  fi
  # Release even for debug builds: the sounds are synthesized at startup and that is slow unoptimized.
  cargo build --manifest-path "$root/Cargo.toml" -p wfhub-ffi --release --target "$triple"
  libs+=("$root/target/$triple/release/libwfhub_ffi.a")
done

out="$here/build/rust/$platform"
mkdir -p "$out"
if [ "${#libs[@]}" -eq 1 ]; then
  cp "${libs[0]}" "$out/libwfhub_ffi.a"
else
  lipo -create "${libs[@]}" -output "$out/libwfhub_ffi.a"
fi
echo "Rust core ready: $out/libwfhub_ffi.a"
