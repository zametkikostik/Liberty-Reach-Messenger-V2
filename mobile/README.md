# Liberty Mobile (Expo / React Native)

Same E2EE contract as web & desktop:

- Client encrypts with **AES-256-GCM** before POST (`lr1:` prefix)
- Server stores **ciphertext only**; WsHub fans out opaque payloads
- Chat keys in **Keychain / Keystore** via `expo-secure-store` (memory fallback on web)

## Setup

```bash
cd mobile
npm install
export EXPO_PUBLIC_API_URL=http://YOUR_LAN_IP:8008
npm start
```

## Modules

- `src/crypto/e2ee.ts` — encrypt/decrypt / key import-export
- `src/storage/secure.ts` — Keychain wrapper
- `src/api/client.ts` — auth, chats, messages, WS URL
- `src/App.tsx` — login, chat list, send with E2EE toggle

## Wire format

```
lr1: + base64( nonce[12] || ciphertext )
```
