# 🏰 Liberty Messenger / Liberty Reach V2

**EN** · **RU** · **BG**

Federated P2P E2EE messenger — Hidden Vault, mesh (libp2p), Cloudflare wake (no Firebase).

---

## English

### What is this?
Liberty is a **sovereign messenger**: end-to-end encryption, optional post-quantum hybrid, master/duress/decoy vault, QR contacts, media & voice, groups, Telegram/WhatsApp import, federated bootstrap (anyone can run a node without breaking others).

### Does GitHub build the APK?
**Yes.** Use **GitHub Actions** (workflow `.github/workflows/android.yml`).  
Actions → workflow → **Run workflow** → download **APK** from Artifacts (after 10–20 min).  
No local Android SDK required for a basic Flutter APK. Native Rust `.so` in CI needs NDK (workflow includes best-effort steps).

### Run locally
```bash
cd mobile
flutter pub get
flutter run

# Unlock passwords (demo)
# master  = real vault
# duress  = panic wipe
# other   = decoy empty UI
```

```bash
cd rust-core
cargo test
cargo build --release
```

```bash
./scripts/build_android_native.sh
cd mobile && flutter build apk --release
```

```bash
cd cloudflare-worker && npm i && npx wrangler deploy
```

### Mesh & federation
- **Mesh:** libp2p mDNS (LAN) + Gossipsub + optional DHT  
- **Federation:** add bootstrap/wake peers in app — does not reset existing peers  

### License
MIT

---

## Русский

### Что это?
**Суверенный мессенджер**: сквозное шифрование (X3DH + Double Ratchet), vault (master / duress / decoy), QR-контакты, медиа и голос, группы, импорт Telegram/WhatsApp, федерация (любой может поднять ноду).

### GitHub сам соберёт APK?
**Да.** Workflow `.github/workflows/android.yml` в **Actions**.  
После прогона скачай APK из **Artifacts**.  
Локальный SDK не обязателен для базовой Flutter-сборки.

### Как запустить
```bash
cd mobile && flutter pub get && flutter run
# пароли: master | duress | любой другой = decoy
```

### Mesh и федерация
- **Mesh:** mDNS в LAN + Gossipsub  
- **Федерация:** экран Federation & Mesh 

### Лицензия
MIT

---

## Български

### Какво е това?
**Суверенен месинджър** с E2EE, скрит vault, QR, медия, глас, групи, импорт, федерация.

### GitHub ще сглоби ли APK?
**Да.** Actions → `.github/workflows/android.yml` → Artifacts.

### Стартиране
```bash
cd mobile && flutter pub get && flutter run
# пароли: master | duress | друго = decoy
```

### Лиценз
MIT

## Honest status
Full-featured messenger foundation. For global scale: native `.so`, field tests, external audit.
