# Liberty Reach Messenger V2

Private messenger: client-side E2EE (AES-256-GCM), WebSocket fan-out, hybrid X25519 handshake, Axum backend.

**Repo:** https://github.com/zametkikostik/Liberty-Reach-Messenger-V2

## Status

| Layer | Status |
|-------|--------|
| Auth (Argon2 + JWT) | Done |
| Chats / messages API | Done — body stored as ciphertext only |
| Client E2EE (`lr1:` prefix) | Done |
| WebSocket room fan-out | Done |
| Hybrid handshake (X25519 + PQ slot) | In tree (`kyber_hybrid.rs`) |
| libp2p Gossipsub | Next |
| Mobile / Tauri UI | Scaffold |

## Quick start

```bash
cd server && cp .env.example .env && cargo run --release
cd frontend && npm i && npm run dev
```

CI workflows run **only** via Actions → Run workflow (no push triggers).

## License

MIT
