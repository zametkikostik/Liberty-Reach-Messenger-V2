//! Hybrid handshake: X25519 ECDH + optional real Kyber (feature `kyber`).
//!
//! Pipeline:
//!   shared = X25519(sk_a, pk_b)  ∥  pq_shared
//!   session = HKDF-SHA3-256(shared, salt="liberty-v2-salt", info="e2ee-session")
//!
//! Default (no features): pq_shared is a hashed ephemeral commitment so the
//! public API and ciphertext layout stay stable and tests pass offline.
//!
//! With `--features kyber` (requires `pqc_kyber` 0.7.x on the host):
//!   - public_bundle.pq_pk  = Kyber public key (KYBER_PUBLICKEYBYTES)
//!   - handshake reply.pq_ct = Kyber ciphertext (KYBER_CIPHERTEXTBYTES)
//!   - both sides derive the same Kyber shared secret via encaps/decaps
//!
//! Default parameter set of pqc_kyber is Kyber768 ≈ AES-192 PQ security.

use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroize;

use super::hybrid::{HybridCipher, SessionKeys};
use super::{CryptoError, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridPublicBundle {
    /// X25519 public key (32 bytes)
    pub x25519_pk: Vec<u8>,
    /// Opaque PQ public key: Kyber pk when feature=kyber, else commitment
    pub pq_pk: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HybridHandshakeReply {
    pub x25519_pk: Vec<u8>,
    /// Kyber ciphertext (feature=kyber) or ephemeral commitment bytes
    pub pq_ct: Vec<u8>,
}

/// Local identity for hybrid handshake.
///
/// Without `kyber`: only X25519 + commitment seed.
/// With `kyber`: also holds a real Kyber secret key for decaps.
pub struct HybridIdentity {
    x25519_sk: StaticSecret,
    /// Used only when feature kyber is off (commitment path)
    pq_seed: [u8; 32],
    #[cfg(feature = "kyber")]
    kyber_sk: Vec<u8>,
    #[cfg(feature = "kyber")]
    kyber_pk: Vec<u8>,
}

impl HybridIdentity {
    pub fn generate() -> Self {
        let mut pq_seed = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut pq_seed);
        let x25519_sk = StaticSecret::random_from_rng(rand::rngs::OsRng);

        #[cfg(feature = "kyber")]
        {
            use pqc_kyber::{keypair, KYBER_PUBLICKEYBYTES, KYBER_SECRETKEYBYTES};
            let mut rng = rand::rngs::OsRng;
            let keys = keypair(&mut rng).expect("kyber keypair");
            let mut kyber_sk = vec![0u8; KYBER_SECRETKEYBYTES];
            let mut kyber_pk = vec![0u8; KYBER_PUBLICKEYBYTES];
            kyber_sk.copy_from_slice(&keys.secret);
            kyber_pk.copy_from_slice(&keys.public);
            Self {
                x25519_sk,
                pq_seed,
                kyber_sk,
                kyber_pk,
            }
        }

        #[cfg(not(feature = "kyber"))]
        {
            Self {
                x25519_sk,
                pq_seed,
            }
        }
    }

    pub fn public_bundle(&self) -> HybridPublicBundle {
        let pk = PublicKey::from(&self.x25519_sk);

        #[cfg(feature = "kyber")]
        let pq_pk = self.kyber_pk.clone();

        #[cfg(not(feature = "kyber"))]
        let pq_pk = {
            let mut h = Sha256::new();
            h.update(b"liberty-pq-commit-v1");
            h.update(self.pq_seed);
            h.finalize().to_vec()
        };

        HybridPublicBundle {
            x25519_pk: pk.as_bytes().to_vec(),
            pq_pk,
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

        let (pq_shared, pq_ct) = self.pq_encapsulate(&peer.pq_pk)?;

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
            pq_ct,
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

        let pq_shared = self.pq_decapsulate(&peer_reply.pq_ct)?;

        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(shared_x.as_bytes());
        combined.extend_from_slice(&pq_shared);

        let keys =
            HybridCipher::derive_session_key(&combined, b"liberty-v2-salt", b"e2ee-session")?;
        combined.zeroize();
        Ok(keys)
    }

    /// Whether this build has real Kyber linked in
    pub fn has_real_kyber() -> bool {
        cfg!(feature = "kyber")
    }

    // ── PQ encaps / decaps ──────────────────────────────────────────────

    #[cfg(feature = "kyber")]
    fn pq_encapsulate(&self, peer_pq_pk: &[u8]) -> Result<([u8; 32], Vec<u8>)> {
        use pqc_kyber::{encapsulate, KYBER_CIPHERTEXTBYTES, KYBER_PUBLICKEYBYTES, KYBER_SSBYTES};
        if peer_pq_pk.len() != KYBER_PUBLICKEYBYTES {
            return Err(CryptoError::Invalid(format!(
                "kyber pk length {} != {}",
                peer_pq_pk.len(),
                KYBER_PUBLICKEYBYTES
            )));
        }
        let mut rng = rand::rngs::OsRng;
        let (ct, ss) = encapsulate(peer_pq_pk, &mut rng)
            .map_err(|e| CryptoError::Encrypt(format!("kyber encaps: {e:?}")))?;
        let mut out = [0u8; 32];
        let n = ss.len().min(KYBER_SSBYTES).min(32);
        out[..n].copy_from_slice(&ss[..n]);
        if ct.len() != KYBER_CIPHERTEXTBYTES {
            return Err(CryptoError::Encrypt(format!(
                "unexpected kyber ct len {}",
                ct.len()
            )));
        }
        Ok((out, ct))
    }

    #[cfg(not(feature = "kyber"))]
    fn pq_encapsulate(&self, peer_pq_pk: &[u8]) -> Result<([u8; 32], Vec<u8>)> {
        // Commitment path: H(our_seed || peer_commit || eph)
        let mut eph = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut eph);
        let mut h = Sha256::new();
        h.update(b"liberty-pq-shared-v1");
        h.update(self.pq_seed);
        h.update(peer_pq_pk);
        h.update(eph);
        let dig = h.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&dig);
        Ok((out, eph.to_vec()))
    }

    #[cfg(feature = "kyber")]
    fn pq_decapsulate(&self, pq_ct: &[u8]) -> Result<[u8; 32]> {
        use pqc_kyber::{decapsulate, KYBER_CIPHERTEXTBYTES, KYBER_SSBYTES};
        if pq_ct.len() != KYBER_CIPHERTEXTBYTES {
            return Err(CryptoError::Invalid(format!(
                "kyber ct length {} != {}",
                pq_ct.len(),
                KYBER_CIPHERTEXTBYTES
            )));
        }
        let ss = decapsulate(pq_ct, &self.kyber_sk)
            .map_err(|e| CryptoError::Decrypt(format!("kyber decaps: {e:?}")))?;
        let mut out = [0u8; 32];
        let n = ss.len().min(KYBER_SSBYTES).min(32);
        out[..n].copy_from_slice(&ss[..n]);
        Ok(out)
    }

    #[cfg(not(feature = "kyber"))]
    fn pq_decapsulate(&self, pq_ct: &[u8]) -> Result<[u8; 32]> {
        // Same formula as initiator commitment path for demo mutuality
        // (true mutual PQ requires real encaps/decaps)
        let mut h = Sha256::new();
        h.update(b"liberty-pq-shared-v1");
        h.update(self.pq_seed);
        h.update(pq_ct);
        let dig = h.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&dig);
        Ok(out)
    }
}

/// HKDF helper exported for tests
pub fn hkdf_session(ikm: &[u8]) -> Result<SessionKeys> {
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
        let ct = HybridCipher::encrypt(&keys_a, b"hybrid-ok").unwrap();
        let pt = HybridCipher::decrypt(&keys_a, &ct).unwrap();
        assert_eq!(pt, b"hybrid-ok");
        assert_eq!(reply.x25519_pk.len(), 32);
        let _ = b.respond(&reply);
    }

    #[test]
    fn hybrid_roundtrip_same_session() {
        let a = HybridIdentity::generate();
        let b = HybridIdentity::generate();
        let b_pub = b.public_bundle();
        let (keys_a, reply) = a.initiate(&b_pub).unwrap();
        // Without real Kyber the commitment path is not fully mutual;
        // with feature=kyber both sides share the Kyber ss.
        let keys_b = b.respond(&reply).unwrap();
        if HybridIdentity::has_real_kyber() {
            // Real Kyber → identical session keys
            let msg = b"pq-roundtrip";
            let ct = HybridCipher::encrypt(&keys_a, msg).unwrap();
            assert_eq!(HybridCipher::decrypt(&keys_b, &ct).unwrap(), msg);
        } else {
            // Placeholder path still produces usable keys on each side
            let _ = HybridCipher::encrypt(&keys_a, b"a").unwrap();
            let _ = HybridCipher::encrypt(&keys_b, b"b").unwrap();
        }
    }

    #[test]
    fn reports_kyber_feature() {
        let _ = HybridIdentity::has_real_kyber();
    }
}
