#!/usr/bin/env bash
# Build Liberty messenger with modern Rust (1.85+) features.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "rustc: $(rustc --version)"

cargo build -p liberty-messenger --features ai
echo "OK: --features ai"

cargo build -p liberty-messenger --features p2p
echo "OK: --features p2p"

cargo build -p liberty-messenger --features "ai,p2p,kyber"
echo "OK: ai+p2p+kyber"

if [[ -n "${QWEN_API_KEY:-}" ]]; then
  cargo run -p liberty-messenger --example ai_translate --features ai -- "Hello from Liberty" ru
else
  echo "Skip live AI: set QWEN_API_KEY to test translate"
fi
