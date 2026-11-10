#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
echo "==> Liberty Messenger V2 self-host"
command -v docker >/dev/null || { echo "Install Docker first"; exit 1; }
export JWT_SECRET="${JWT_SECRET:-$(openssl rand -hex 32 2>/dev/null || echo liberty-fallback-secret-change-me)}"
mkdir -p ../uploads
docker compose up -d --build
echo "Health: http://localhost:8008/health"
echo "Stop:   docker compose down"
