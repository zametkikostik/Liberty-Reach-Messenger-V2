# Security Audit Checklist

## Crypto
- [ ] X3DH shared secrets match initiator/responder
- [ ] Double Ratchet encrypt/decrypt, skipped keys window
- [ ] AEAD AES-GCM + ChaCha20-Poly1305
- [ ] PQ hybrid (feature post-quantum)
- [ ] Sealed sender open/seal
- [ ] Zeroize on drop for secrets

## Vault
- [ ] Master → Real, other → Decoy, Duress → Panic
- [ ] Panic wipes real/decoy/identity files
- [ ] Argon2id params reasonable

## Platform
- [ ] Android Keystore wrap/unwrap
- [ ] iOS Keychain + CryptoKit
- [ ] FLAG_SECURE / screenshot protection

## Network
- [ ] No plaintext on wake relay
- [ ] Federation peer update non-destructive

## Process
- [ ] No secrets in git
- [ ] SECURITY.md contact set
- [ ] External audit before mass marketing
