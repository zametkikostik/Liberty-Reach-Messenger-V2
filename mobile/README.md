# Liberty Mobile (React Native / Expo shell)

Same E2EE contract as web: client encrypts before POST (`lr1:` AES-GCM).

## Run (Expo)

```bash
npm install
EXPO_PUBLIC_API_URL=http://YOUR_LAN_IP:8008 npm start
```

Point the device at your Axum server (`server` crate on `:8008`).

## Production notes

- Replace `localStorage` key storage with Keychain / Keystore.
- Prefer `react-native-quick-crypto` (or equivalent) for AES-GCM on native.
- WebSocket: use `wss://` with the same JWT query param as the web client.
