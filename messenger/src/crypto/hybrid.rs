//! Hybrid E2EE pipeline:
//! shared_secret (X25519 ∥ Kyber) → HKDF-SHA3-256 → AES-256-GCM
//!
//! Kyber material slots are ready; session key path works with pure AES-GCM today.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha3::Sha3_256;
use zeroize::Zeroize;

use super::{CryptoError, Result};

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

pub struct SessionKeys {
    key: [u8; KEY_LEN],
}

impl Drop for SessionKeys {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ciphertext {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    /// Opaque KEM ciphertext (Kyber) when hybrid path is fully wired
    pub kem_ct: Vec<u8>,
}

pub struct HybridCipher;

impl HybridCipher {
    pub fn generate_session_key() -> SessionKeys {
        let mut key = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut key);
        SessionKeys { key }
    }

    pub fn derive_session_key(shared: &[u8], salt: &[u8], info: &[u8]) -> Result<SessionKeys> {
        let hk = Hkdf::<Sha3_256>::new(Some(salt), shared);
        let mut okm = [0u8; KEY_LEN];
        hk.expand(info, &mut okm)
            .map_err(|e| CryptoError::Kdf(e.to_string()))?;
        Ok(SessionKeys { key: okm })
    }

    pub fn encrypt(keys: &SessionKeys, plaintext: &[u8]) -> Result<Ciphertext> {
        let cipher =
            Aes256Gcm::new_from_slice(&keys.key).map_err(|e| CryptoError::Encrypt(e.to_string()))?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
        Ok(Ciphertext {
            nonce: nonce_bytes.to_vec(),
            ciphertext: ct,
            kem_ct: Vec::new(),
        })
    }

    pub fn decrypt(keys: &SessionKeys, data: &Ciphertext) -> Result<Vec<u8>> {
        if data.nonce.len() != NONCE_LEN {
            return Err(CryptoError::Invalid("nonce length".into()));
        }
        let cipher =
            Aes256Gcm::new_from_slice(&keys.key).map_err(|e| CryptoError::Decrypt(e.to_string()))?;
        let nonce = Nonce::from_slice(&data.nonce);
        cipher
            .decrypt(nonce, data.ciphertext.as_ref())
            .map_err(|e| CryptoError::Decrypt(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let keys = HybridCipher::generate_session_key();
        let msg = b"Liberty Messenger V2 secret";
        let ct = HybridCipher::encrypt(&keys, msg).unwrap();
        assert_eq!(HybridCipher::decrypt(&keys, &ct).unwrap(), msg);
    }

    #[test]
    fn tamper_fails() {
        let keys = HybridCipher::generate_session_key();
        let mut ct = HybridCipher::encrypt(&keys, b"x").unwrap();
        if let Some(b) = ct.ciphertext.first_mut() {
            *b ^= 0xff;
        }
        assert!(HybridCipher::decrypt(&keys, &ct).is_err());
    }

    #[test]
    fn hkdf_derive() {
        let keys = HybridCipher::derive_session_key(b"shared", b"salt", b"liberty-v2").unwrap();
        let ct = HybridCipher::encrypt(&keys, b"ok").unwrap();
        assert_eq!(HybridCipher::decrypt(&keys, &ct).unwrap(), b"ok");
    }
}
