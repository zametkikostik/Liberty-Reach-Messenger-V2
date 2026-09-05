//! Post-quantum hybrid: X25519 + Kyber768 (feature post-quantum)
//! Without feature: deterministic placeholder (API compiles, not PQ-safe)

use crate::crypto::{EphemeralKeyPair, SessionKeys};
use sha2::Sha256;
use hkdf::Hkdf;
use serde::{Serialize, Deserialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use rand::RngCore;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PqPublicKey {
    pub classical: [u8; 32],
    pub pq: Vec<u8>,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct PqSharedSecret {
    pub bytes: [u8; 32],
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct PqKeyPair {
    #[zeroize(skip)]
    pub public: Vec<u8>,
    pub secret: Vec<u8>,
}

pub struct HybridKeyExchange;

impl HybridKeyExchange {
    pub fn generate_pq_keypair() -> PqKeyPair { pq_keypair() }

    pub fn generate_hybrid_public() -> (EphemeralKeyPair, PqKeyPair, PqPublicKey) {
        let classical = EphemeralKeyPair::generate();
        let pq = pq_keypair();
        let pk = PqPublicKey { classical: classical.public_bytes(), pq: pq.public.clone() };
        (classical, pq, pk)
    }

    pub fn encapsulate(their_pk: &PqPublicKey) -> (PqSharedSecret, PqPublicKey, Vec<u8>) {
        let our_classical = EphemeralKeyPair::generate();
        let classical_ss = our_classical.diffie_hellman(&their_pk.classical);
        let (pq_ss, ct) = pq_encapsulate(&their_pk.pq);
        let our_pk = PqPublicKey { classical: our_classical.public_bytes(), pq: vec![] };
        (combine(classical_ss.as_slice(), &pq_ss), our_pk, ct)
    }

    pub fn decapsulate(
        our_classical: &EphemeralKeyPair,
        our_pq: &PqKeyPair,
        their_classical_pk: &[u8; 32],
        pq_ciphertext: &[u8],
    ) -> PqSharedSecret {
        let classical_ss = our_classical.diffie_hellman(their_classical_pk);
        let pq_ss = pq_decapsulate(&our_pq.secret, pq_ciphertext);
        combine(classical_ss.as_slice(), &pq_ss)
    }

    pub fn to_session_keys(ss: &PqSharedSecret, initiator: bool) -> SessionKeys {
        let hk = Hkdf::<Sha256>::new(None, &ss.bytes);
        let mut okm = [0u8; 96];
        hk.expand(b"Liberty-PQ-Session", &mut okm).expect("HKDF");
        let mut k1 = [0u8; 32];
        let mut k2 = [0u8; 32];
        let mut root = [0u8; 32];
        k1.copy_from_slice(&okm[0..32]);
        k2.copy_from_slice(&okm[32..64]);
        root.copy_from_slice(&okm[64..96]);
        if initiator {
            SessionKeys { send_key: k1, recv_key: k2, chain_key: root }
        } else {
            SessionKeys { send_key: k2, recv_key: k1, chain_key: root }
        }
    }
}

fn combine(classical: &[u8], pq: &[u8]) -> PqSharedSecret {
    let mut ikm = Vec::with_capacity(classical.len() + pq.len());
    ikm.extend_from_slice(classical);
    ikm.extend_from_slice(pq);
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut bytes = [0u8; 32];
    hk.expand(b"Liberty-Hybrid-v1", &mut bytes).expect("HKDF");
    PqSharedSecret { bytes }
}

#[cfg(feature = "post-quantum")]
fn pq_keypair() -> PqKeyPair {
    use pqcrypto_kyber::kyber768;
    use pqcrypto_traits::kem::{PublicKey, SecretKey};
    let (pk, sk) = kyber768::keypair();
    PqKeyPair { public: pk.as_bytes().to_vec(), secret: sk.as_bytes().to_vec() }
}

#[cfg(not(feature = "post-quantum"))]
fn pq_keypair() -> PqKeyPair {
    let mut public = vec![0u8; 32];
    let mut secret = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut public);
    rand::thread_rng().fill_bytes(&mut secret);
    PqKeyPair { public, secret }
}

#[cfg(feature = "post-quantum")]
fn pq_encapsulate(pk_bytes: &[u8]) -> (Vec<u8>, Vec<u8>) {
    use pqcrypto_kyber::kyber768;
    use pqcrypto_traits::kem::{PublicKey, SharedSecret, Ciphertext};
    let pk = kyber768::PublicKey::from_bytes(pk_bytes).expect("pk");
    let (ss, ct) = kyber768::encapsulate(&pk);
    (ss.as_bytes().to_vec(), ct.as_bytes().to_vec())
}

#[cfg(not(feature = "post-quantum"))]
fn pq_encapsulate(pk_bytes: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut ss = vec![0u8; 32];
    let mut ct = pk_bytes.to_vec();
    rand::thread_rng().fill_bytes(&mut ss);
    (ss, ct)
}

#[cfg(feature = "post-quantum")]
fn pq_decapsulate(sk_bytes: &[u8], ct_bytes: &[u8]) -> Vec<u8> {
    use pqcrypto_kyber::kyber768;
    use pqcrypto_traits::kem::{SecretKey, Ciphertext, SharedSecret};
    let sk = kyber768::SecretKey::from_bytes(sk_bytes).expect("sk");
    let ct = kyber768::Ciphertext::from_bytes(ct_bytes).expect("ct");
    kyber768::decapsulate(&ct, &sk).as_bytes().to_vec()
}

#[cfg(not(feature = "post-quantum"))]
fn pq_decapsulate(_sk: &[u8], ct: &[u8]) -> Vec<u8> {
    let mut ss = vec![0u8; 32];
    for (i, b) in ct.iter().take(32).enumerate() { ss[i] = *b; }
    ss
}
