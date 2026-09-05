#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
if ! command -v flutter_rust_bridge_codegen &>/dev/null; then
  cargo install flutter_rust_bridge_codegen || true
fi
echo "FRB config: flutter_rust_bridge.yaml"
echo "Run codegen when toolchain ready"
