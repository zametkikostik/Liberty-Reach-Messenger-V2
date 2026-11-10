# Liberty Messenger V2 — Quick Start

## Backend
```bash
cd server && cp .env.example .env && cargo run --release
```

## Web
```bash
cd frontend && npm install && npm run dev
```

## Crypto tests
```bash
cd messenger && cargo test --lib
```

## Docker
```bash
cd self-hosting && chmod +x install.sh && ./install.sh
```

## Bots
```bash
cd bots && cargo run --release
curl -X POST http://localhost:8081/api/v1/bots \
  -H 'Content-Type: application/json' \
  -d '{"username":"my_bot","name":"My Bot"}'
```

## E2EE
Client encrypts with AES-256-GCM before POST; server stores ciphertext only.
Export/import chat key in UI for multi-device.

## Workflows
CI/Release only via **Actions → Run workflow** (no auto-run on push).
