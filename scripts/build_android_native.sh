#!/usr/bin/env bash
# Build the native Rust core for Android and place it where the Flutter
# app bundles it (mobile/android/app/src/main/jniLibs/<abi>/libliberty_core.so).
# Without this step the APK has NO native core and cannot encrypt.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/rust-core"

echo "==> Host build (sanity check)"
cargo build --release

command -v cargo-ndk >/dev/null 2>&1 || {
  echo "ERROR: cargo-ndk is required (cargo install cargo-ndk) and an Android NDK"
  echo "       must be available (ANDROID_NDK_HOME or ndk bundled with Android Studio)."
  exit 1
}

echo "==> Cross-compile for Android ABIs"
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 \
  -o "$ROOT/mobile/android/app/src/main/jniLibs" build --release

echo "==> Result:"
find "$ROOT/mobile/android/app/src/main/jniLibs" -name '*.so' -exec ls -la {} \;
echo "Done. Now: cd mobile && flutter run  (or flutter build apk)"
