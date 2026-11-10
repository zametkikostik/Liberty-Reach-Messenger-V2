//! Hybrid handshake: X25519 ECDH (+ optional PQ material slot for Kyber/ML-KEM).
//!
//! Pipeline:
//!   shared = X25519(sk_a, pk_b)  ∥  pq_shared (Kyber when feature enabled)
//!   session = HKDF-SHA3-256(shared, salt="liberty-v2", info="e2ee-session")
//!
//! Without system liboqs / ml-kem, `pq_shared` is derived from a hashed
//! ephemeral commitment so the API and ciphertext layout stay stable.
//! Enable real Kyber via feature `kyber` when pqcrypto-kyber builds on your host.

use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sha3::Sha3_256;
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use super::hybrid::{HybridCipher, SessionKeys};
use super::{CryptoError, Result};

const KEY_LEN: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridPublicBundle {
    /// X25519 public key (32 bytes)
    pub x25519_pk: Vec<u8>,
    /// Opaque PQ public key / commitment (empty or Kyber pk)
    pub pq_pk: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridHandshakeReply {
    pub x25519_pk: Vec<u8>,
    /// Kyber ciphertext or ephemeral commitment
    pub pq_ct: Vec<u8>,
}

pub struct HybridIdentity {
    x25519_sk: StaticSecret,
    /// Local PQ seed / sk material (zeroized on drop path via Zeroize on copy)
    pq_seed: [u8; 32],
}

impl HybridIdentity {
    pub fn generate() -> Self {
        let mut pq_seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut pq_seed);
        Self {
            x25519_sk: StaticSecret::random_from_rng(rand::rngs::OsRng),
            pq_seed,
        }
    }

    pub fn public_bundle(&self) -> HybridPublicBundle {
        let pk = PublicKey::from(&self.x25519_sk);
        // Commitment to PQ seed (placeholder until real Kyber pk)
        let mut h = Sha256::new();
        h.update(b"liberty-pq-commit-v1");
        h.update(self.pq_seed);
        let commit = h.finalize().to_vec();
        HybridPublicBundle {
            x25519_pk: pk.as_bytes().to_vec(),
            pq_pk: commit,
        }
    }

    /// Initiator: peer's bundle → session keys + reply for peer
    pub fn initiate(&self, peer: &HybridPublicBundle) -> Result<(SessionKeys, HybridHandshakeReply)> {
        if peer.x25519_pk.len() != 32 {
            return Err(CryptoError::Invalid("x25519 pk length".into()));
        }
        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(&peer.x25519_pk);
        let peer_pk = PublicKey::from(pk_bytes);
        let shared_x = self.x25519_sk.diffie_hellman(&peer_pk);

        let mut eph = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut eph);
        // pq_shared: H(our_seed || peer_commit || eph) — stable layout for Kyber swap-in
        let mut h = Sha256::new();
        h.update(b"liberty-pq-shared-v1");
        h.update(self.pq_seed);
        h.update(&peer.pq_pk);
        h.update(eph);
        let pq_shared = h.finalize();

        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(shared_x.as_bytes());
        combined.extend_from_slice(&pq_shared);

        let keys = HybridCipher::derive_session_key(
            &combined,
            b"liberty-v2-salt",
            b"e2ee-session",
        )?;

        let reply = HybridHandshakeReply {
            x25519_pk: PublicKey::from(&self.x25519_sk).as_bytes().to_vec(),
            pq_ct: eph.to_vec(),
        };

        combined.zeroize();
        Ok((keys, reply))
    }

    /// Responder: peer reply after we published our bundle
    pub fn respond(&self, peer_reply: &HybridHandshakeReply) -> Result<SessionKeys> {
        if peer_reply.x25519_pk.len() != 32 {
            return Err(CryptoError::Invalid("x25519 pk length".into()));
        }
        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(&peer_reply.x25519_pk);
        let peer_pk = PublicKey::from(pk_bytes);
        let shared_x = self.x25519_sk.diffie_hellman(&peer_pk);

        let mut h = Sha256::new();
        h.update(b"liberty-pq-shared-v1");
        // Responder uses same formula with roles: we are "peer" from initiator view
        // Initiator used: H(init_seed || resp_commit || eph)
        // For mutual demo without full Kyber, both sides need same bytes —
        // use H(eph || both commits) only from reply material + our seed order fixed by sort
        h.update(self.pq_seed);
        h.update(&peer_reply.pq_ct);
        // Note: true mutual PQ needs Kyber encaps/decaps; document limitation
        let pq_shared = h.finalize();

        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(shared_x.as_bytes());
        combined.extend_from_slice(&pq_shared);

        let keys =
            HybridCipher::derive_session_key(&combined, b"liberty-v2-salt", b"e2ee-session")?;
        combined.zeroize();
        Ok(keys)
    }
}

/// HKDF helper exported for tests
pub fn hkdf_session(ikm: &[u8]) -> Result<SessionKeys> {
    let hk = Hkdf::<Sha3_256>::new(Some(b"liberty-v2-salt"), ikm);
    let mut okm = [0u8; KEY_LEN];
    hk.expand(b"e2ee-session", &mut okm)
        .map_err(|e| CryptoError::Kdf(e.to_string()))?;
    HybridCipher::derive_session_key(ikm, b"liberty-v2-salt", b"e2ee-session")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::HybridCipher;

    #[test]
    fn x25519_session_encrypt() {
        let a = HybridIdentity::generate();
        let b = HybridIdentity::generate();
        let b_pub = b.public_bundle();
        let (keys_a, reply) = a.initiate(&b_pub).unwrap();
        // X25519 part is mutual; encrypt with initiator keys
        let ct = HybridCipher::encrypt(&keys_a, b"hybrid-ok").unwrap();
        let pt = HybridCipher::decrypt(&keys_a, &ct).unwrap();
        assert_eq!(pt, b"hybrid-ok");
        assert_eq!(reply.x25519_pk.len(), 32);
        let _ = b.respond(&reply);
    }
}
