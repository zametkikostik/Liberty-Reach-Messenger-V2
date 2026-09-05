# External audit package

## Scope for auditors
1. `rust-core/src/crypto/` — X3DH, Double Ratchet, AEAD, PQ hybrid, sealed sender
2. `rust-core/src/security/` — vault, wipe, keystore interface
3. FFI boundary `api/mod.rs` — secret handling across Dart/Rust
4. Android Keystore / iOS Keychain integration
5. Threat model `docs/THREAT_MODEL.md` vs implementation gaps

## Build
```bash
cd rust-core
cargo test
cargo test --features post-quantum
cargo build --release
```

## Known non-goals for v0.1
- Full RFC 9420 MLS
- Metadata-perfect anonymity
- Resistance to compromised OS / hardware lab attacks

## Deliverables requested
- Findings (Critical/High/Medium/Low/Info)
- Reproduction notes
- Recommended fixes with priority
