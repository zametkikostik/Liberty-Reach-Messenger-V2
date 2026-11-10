#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
echo "Liberty Messenger V2 tree under $ROOT"
find "$ROOT" -type f ! -path '*/target/*' ! -path '*/node_modules/*' | sort | wc -l
echo "Required roots:"
for d in messenger server frontend bots smart-contracts self-hosting cloudflare migration-tool docs .github; do
  test -d "$ROOT/$d" && echo "  OK $d" || echo "  MISSING $d"
done
