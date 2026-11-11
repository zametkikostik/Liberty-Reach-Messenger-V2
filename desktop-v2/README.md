# Liberty Desktop V2 (Tauri 2)

Native shell around the React frontend.

## Prerequisites

- Rust 1.75+
- Node 18+
- System deps for Tauri: https://v2.tauri.app/start/prerequisites/

## Dev

```bash
# terminal 1 — API
cd ../server && cargo run

# terminal 2 — UI (or let Tauri beforeDevCommand start it)
cd ../frontend && npm install && npm run dev

# terminal 3 — desktop shell
cd src-tauri && cargo tauri dev
```

## Commands exposed to the webview

| Command | Purpose |
|---------|--------|
| `app_info` | version, e2ee/kyber flags |
| `open_backend_url` | base URL of Axum server |

E2EE stays in the webview (`frontend/src/crypto/e2ee.ts`). Native process never sees plaintext.
