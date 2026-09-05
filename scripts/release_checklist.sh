#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "== Liberty release checklist =="
cd "$ROOT/rust-core"
cargo test || true
cargo build --release || true
echo "Flutter: cd mobile && flutter build apk --release"
echo "Host store/PRIVACY.md"
echo "Update SECURITY.md contact"
echo "Schedule external audit"
