#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../cloudflare-worker"
npm install
echo "Create KV: npx wrangler kv:namespace create LIBERTY_KV"
echo "Put id into wrangler.toml then: npx wrangler deploy"
