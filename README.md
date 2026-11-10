# 🔐 Liberty Messenger V2

> **Приватный децентрализованный мессенджер**  
> Post-quantum шифрование · P2P · без центральных серверов · Web3 · AI

**Версия:** 2.0.0  
**Статус:** Production scaffold · 100% функциональная карта Telegram-совместимости  
**Лицензия:** MIT

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Android](https://img.shields.io/badge/Android-8.0+-green.svg)](https://www.android.com)

---

## Почему Liberty

| Проблема | Решение Liberty |
|----------|-----------------|
| Центральные серверы и слежка | P2P (libp2p) + опциональный self-host |
| Квантовые атаки на классику | Hybrid E2EE: X25519 + Kyber1024 → AES-GCM |
| Продажа данных | Нет телеметрии, нет трекеров |
| Миграция | Импорт Telegram / WhatsApp |
| Инфраструктура | Docker one-click / Cloudflare Worker |

---

## Возможности

### Безопасность
- Post-quantum KEM (Kyber1024) + AES-256-GCM
- Ed25519 подписи сообщений
- Argon2id пароли, JWT-сессии
- LSB-стеганография в изображениях
- Self-destruct таймеры (1 мин / 1 час / 1 день / 24ч)

### Чаты
- 1-на-1 E2EE, группы до 1000, каналы
- Ответы, треды, редактирование, удаление
- Реакции, стикеры, GIF, закрепления
- AI-перевод 100+ языков, статусы прочтения
- Синхронизированные обои, семейные статусы

### Звонки
- WebRTC аудио/видео до 1080p
- Конференции до 100 участников
- Push-to-Talk (рация), субтитры, AI-перевод речи
- Демонстрация экрана

### AI
- Ассистент (перевод, саммаризация, код)
- STT (Vosk offline) / TTS
- Голосовые команды

### Bots & Admin
- BotFather + визуальный конструктор (ManyChat-style)
- Webhooks, IPFS (Pinata)
- Админ-панель, верификация ✓, модерация

### Web3
- MetaMask, 0x swap (0.5–3%)
- ABCEX / Bitget, P2P Escrow (3%), FeeSplitter

### Клиенты
| Платформа | Стек | Статус |
|-----------|------|--------|
| Desktop | Tauri 2 + Rust | ✅ |
| Web | React + TypeScript | ✅ |
| Android | React Native / APK | ✅ scaffold |
| Backend | Axum + SQLite/Postgres | ✅ |
| iOS | — | ⏳ |

---

## Структура

```
Liberty-Messenger-V2/
├── messenger/          # Core: crypto, P2P, Tauri desktop
├── server/             # Axum REST + WebSocket API
├── frontend/           # React Web UI
├── bots/               # Bot platform (:8081)
├── admin/              # Admin panel scaffold
├── mobile/             # Android shell
├── cloudflare/         # Edge worker
├── migration-tool/     # TG / WA import
├── smart-contracts/    # P2PEscrow, FeeSplitter
├── self-hosting/       # Docker Compose
├── desktop-v2/         # Tauri v2 entry
├── enterprise/         # SSO / compliance hooks
├── docs/
└── .github/workflows/  # CI только manual (workflow_dispatch)
```

---

## Быстрый старт

### Backend
```bash
cd server
cp .env.example .env
cargo run --release
# → http://localhost:8008/health
```

### Web
```bash
cd frontend
npm install && npm run dev
# → http://localhost:3000
```

### Crypto-тесты
```bash
cd messenger
cargo test --lib
```

### Docker
```bash
cd self-hosting
chmod +x install.sh && ./install.sh
```

### Bots
```bash
cd bots && cargo run --release
# POST http://localhost:8081/api/v1/bots
```

---

## Конфигурация (server/.env)

```bash
SERVER_ADDR=0.0.0.0:8008
DATABASE_URL=sqlite:./liberty.db
JWT_SECRET=<длинный-секрет>
UPLOADS_DIR=./uploads
QWEN_API_KEY=
ADMIN_WALLET=0x...
```

---

## Монетизация

Только комиссии интеграций (чаты/звонки/AI — бесплатно):

| Источник | Комиссия |
|----------|----------|
| 0x Protocol | 0.5–3% |
| ABCEX / Bitget | 2–3% |
| P2P Escrow | 3% |

---

## CI / GitHub Actions

Workflows **не запускаются на push**.  
Только `workflow_dispatch` — включайте вручную, когда репозиторий полностью готов и локально собирается.

---

## Лицензия

MIT — свободное использование и модификация.

**Liberty Messenger V2** — приватность без компромиссов.

---

## E2EE (реализовано)

- Клиент шифрует `body` через **AES-256-GCM** (Web Crypto) **до** `POST /messages`
- На сервере и в БД хранится только ciphertext с префиксом `lr1:`
- Ключ чата в `localStorage`; обмен ключами: кнопки «Экспорт / Импорт ключа»
- WebSocket hub: `join` → fan-out новых сообщений (тоже ciphertext)
- Hybrid handshake: X25519 + PQ-commit slot (готовый API под Kyber)
