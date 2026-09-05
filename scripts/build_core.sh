#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../rust-core"
echo "Building liberty-core"
cargo build --release
ls -la target/release/libliberty_core* 2>/dev/null || true
echo Done
