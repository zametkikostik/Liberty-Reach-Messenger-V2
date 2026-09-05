# Liberty as a product

## User
1. Install APK
2. master password
3. QR contact
4. Chat / media / voice
5. Optional: Federation peers + Worker URL

## Operator
1. npx wrangler deploy
2. Optional libp2p bootstrap
3. Publish multiaddr + wake URL

## Build
```bash
./scripts/build_android_native.sh
./scripts/deploy_worker.sh
cd mobile && flutter build apk --release
```
