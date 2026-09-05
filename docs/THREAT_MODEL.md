# Threat Model — Liberty Messenger

## Assets
- Message content, identity keys, session keys, contacts, master password

## Adversaries

### Casual inspection ("покажи телефон")
Duress / Decoy password → empty UI. Real data stays encrypted.

### Compelled password
Separate Duress password → irreversible wipe.

### Forensic extraction (Cellebrite / UFED)
Argon2id at-rest, zeroize, panic wipe, secure overwrite, hardware keystore when available.

### Network adversary
Noise + E2EE Double Ratchet. Content not readable. Metadata minimization ongoing.

### Malicious peer
Auth + signatures; rate limiting later.

## We do NOT claim
- Nation-state with prolonged physical access / lab implants
- Safety if user reveals master password
- Perfect security on fully compromised OS

## Principles
Fail closed, zeroize, real vs decoy DBs, password never stored, defence in depth.
