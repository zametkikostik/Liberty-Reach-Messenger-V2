#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../cloudflare-worker"
npm install
echo "Create KV: npx wrangler kv:namespace create LIBERTY_KV"
echo "Then: npx wrangler deploy"
