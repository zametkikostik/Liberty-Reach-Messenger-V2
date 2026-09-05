#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/rust-core"
cargo build --release || true
if command -v cargo-ndk &>/dev/null; then
  cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 \
    -o "$ROOT/mobile/android/app/src/main/jniLibs" build --release
fi
cd "$ROOT/mobile"
flutter pub get || true
echo Done
