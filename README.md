# Liberty Messenger / Liberty Reach V2

**EN** · **RU** · **BG**

Federated P2P E2EE messenger — Hidden Vault, mesh (libp2p), Cloudflare wake (no Firebase).

---

## Releases / APK (GitHub Actions)

1. Open **Actions** → **Build Android APK** → **Run workflow**
2. Leave `create_release=true` and set tag e.g. `v0.1.0`
3. After build:
   - **Artifacts** — download APK
   - **Releases** — same APK attached to a release

Or push a tag:
```bash
git tag v0.1.0 && git push origin v0.1.0
```

Demo unlock: `master` | `duress` | other = decoy

---

## English

Sovereign messenger: E2EE (X3DH + Double Ratchet), vault, QR, media/voice, groups, Telegram/WhatsApp import, federation.

```bash
cd mobile && flutter pub get && flutter run
cd rust-core && cargo test
```

License: MIT

---

## Русский

Суверенный мессенджер: E2EE, vault, QR, медиа, голос, группы, импорт, федерация.

**APK:** Actions → Build Android APK → Run workflow → Artifacts / Releases.

```bash
cd mobile && flutter pub get && flutter run
```

---

## Български

Суверенен месинджър с E2EE, vault, QR, медия, федерация.

**APK:** Actions → Build Android APK → Run workflow.

---

## Status

Full messenger foundation on GitHub (crypto, vault, UI, worker, CI).  
Some local-only long originals may differ slightly from CI-uploaded copies.
