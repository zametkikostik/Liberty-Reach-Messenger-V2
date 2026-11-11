# Liberty Desktop V2 (Tauri 2)

Native shell around the shared React frontend (`../frontend`).

E2EE stays in the webview (`frontend/src/crypto/e2ee.ts`). The Rust process never sees message plaintext.

## Layout

```
desktop-v2/
├── package.json          # @tauri-apps/cli scripts
├── README.md
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/default.json
    ├── icons/
    └── src/
        ├── main.rs       # desktop entry → liberty_desktop_lib::run()
        └── lib.rs        # commands + Builder
```

## Prerequisites

- Rust 1.75+ (1.77+ recommended for Tauri 2)
- Node 18+
- System libs: https://v2.tauri.app/start/prerequisites/
- CLI: `cargo install tauri-cli --version "^2"` **or** `npm i` in this folder

## Dev

```bash
# 1) Backend
cd ../server && cargo run
# → http://127.0.0.1:8008/health

# 2) Desktop (starts Vite via beforeDevCommand + native window)
cd ../desktop-v2
npm install          # once — pulls @tauri-apps/cli
npm run dev
# or: cargo tauri dev --manifest-path src-tauri/Cargo.toml
```

Frontend proxies `/api` and `/ws` to the server (see `frontend/vite.config.ts`).

Optional: `LIBERTY_SERVER_URL=http://127.0.0.1:8008 npm run dev`

## Production build

```bash
cd desktop-v2
npm run build
# artifacts under src-tauri/target/release/bundle/
```

## Native commands (invoke from webview)

| Command | Returns |
|---------|--------|
| `app_info` | name, version, e2ee/kyber/p2p flags, OS |
| `backend_url` | Axum base URL |
| `backend_health` | configured endpoint info |

Example (with `withGlobalTauri: true`):

```js
const info = await window.__TAURI__.core.invoke('app_info');
```

## Capabilities

`capabilities/default.json` grants core window/app + `shell:allow-open` for the `main` window only.
