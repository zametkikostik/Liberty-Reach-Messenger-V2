# Encryption in Liberty Messenger

## Can messages be intercepted?
Message content: **no** if E2EE works, no malware, and master password not given.
Metadata (peer IDs, timing, packet size, IP) may still be visible without Tor/mixnet.

## Layers
1. Vault at-rest: Argon2id + AEAD
2. E2EE: X3DH + Double Ratchet + AES-256-GCM / ChaCha20-Poly1305
3. Transport: Noise (libp2p)
4. Optional PQ hybrid: X25519 + Kyber

## Diffie-Hellman
X25519 ECDH in X3DH (triple DH), Double Ratchet, and PQ hybrid.

## Sealed Sender
WireMessage::SealedChat hides sender identity from transport observers.

## Kyber
```bash
cargo build --features post-quantum
```
