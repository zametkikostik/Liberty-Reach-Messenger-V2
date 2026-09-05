//! Key management with zeroize
//! Ed25519 signing + independent X25519 DH (Signal-style)

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{StaticSecret, PublicKey as X25519Public};
use rand::rngs::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::crypto::SecretBytes;
use serde::{Serialize, Deserialize};

#[derive(ZeroizeOnDrop)]
pub struct IdentityKeyPair {
    signing: SigningKey,
    #[zeroize(skip)]
    verifying: VerifyingKey,
    dh_secret: StaticSecret,
    #[zeroize(skip)]
    dh_public: X25519Public,
}

impl IdentityKeyPair {
    pub fn generate() -> Self {
        let signing = SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        let dh_secret = StaticSecret::random_from_rng(OsRng);
        let dh_public = X25519Public::from(&dh_secret);
        Self { signing, verifying, dh_secret, dh_public }
    }

    pub fn from_bytes(ed_secret: [u8; 32], x25519_secret: [u8; 32]) -> Self {
        let signing = SigningKey::from_bytes(&ed_secret);
        let verifying = signing.verifying_key();
        let dh_secret = StaticSecret::from(x25519_secret);
        let dh_public = X25519Public::from(&dh_secret);
        Self { signing, verifying, dh_secret, dh_public }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] { self.verifying.to_bytes() }
    pub fn secret_bytes(&self) -> [u8; 32] { self.signing.to_bytes() }
    pub fn dh_public_bytes(&self) -> [u8; 32] { *self.dh_public.as_bytes() }
    pub fn dh_secret_bytes(&self) -> [u8; 32] { self.dh_secret.to_bytes() }

    pub fn diffie_hellman(&self, their_public: &[u8; 32]) -> SecretBytes {
        let their = X25519Public::from(*their_public);
        let shared = self.dh_secret.diffie_hellman(&their);
        SecretBytes::new(shared.as_bytes().to_vec())
    }

    pub fn sign(&self, msg: &[u8]) -> Signature { self.signing.sign(msg) }

    pub fn verify(public: &[u8; 32], msg: &[u8], sig: &Signature) -> bool {
        match VerifyingKey::from_bytes(public) {
            Ok(vk) => vk.verify(msg, sig).is_ok(),
            Err(_) => false,
        }
    }
}

#[derive(ZeroizeOnDrop)]
pub struct EphemeralKeyPair {
    secret: StaticSecret,
    #[zeroize(skip)]
    public: X25519Public,
}

impl EphemeralKeyPair {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519Public::from(&secret);
        Self { secret, public }
    }

    pub fn from_secret(bytes: [u8; 32]) -> Self {
        let secret = StaticSecret::from(bytes);
        let public = X25519Public::from(&secret);
        Self { secret, public }
    }

    pub fn public_bytes(&self) -> [u8; 32] { *self.public.as_bytes() }
    pub fn secret_bytes(&self) -> [u8; 32] { self.secret.to_bytes() }

    pub fn diffie_hellman(&self, their_public: &[u8; 32]) -> SecretBytes {
        let their = X25519Public::from(*their_public);
        let shared = self.secret.diffie_hellman(&their);
        SecretBytes::new(shared.as_bytes().to_vec())
    }
}

pub struct DiffieHellman;

impl DiffieHellman {
    pub fn pairwise_shared(alice: &EphemeralKeyPair, bob_public: &[u8; 32]) -> SecretBytes {
        alice.diffie_hellman(bob_public)
    }

    pub fn secrets_match(a: &SecretBytes, b: &SecretBytes) -> bool {
        use subtle::ConstantTimeEq;
        if a.len() != b.len() { return false; }
        a.as_slice().ct_eq(b.as_slice()).into()
    }
}

#[derive(Clone, Debug, Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    pub send_key: [u8; 32],
    pub recv_key: [u8; 32],
    pub chain_key: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PublicIdentity {
    pub ed25519: [u8; 32],
    pub x25519: [u8; 32],
}
