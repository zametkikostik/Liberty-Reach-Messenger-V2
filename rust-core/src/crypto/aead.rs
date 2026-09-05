//! Authenticated encryption (AEAD)
//! Wire format (v2): [suite:u8][nonce:12][ciphertext||tag]

use aes_gcm::{aead::{Aead, KeyInit, Payload}, Aes256Gcm, Nonce};
use chacha20poly1305::{ChaCha20Poly1305, Nonce as ChaNonce};
use rand::RngCore;
use serde::{Serialize, Deserialize};
use thiserror::Error;

const NONCE_LEN: usize = 12;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AeadError {
    #[error("encryption failed")]
    Encrypt,
    #[error("decryption failed (tampered or wrong key)")]
    Decrypt,
    #[error("invalid ciphertext length")]
    InvalidLength,
    #[error("unsupported algorithm")]
    UnsupportedAlgorithm,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum CipherSuite {
    #[default]
    Aes256Gcm = 1,
    ChaCha20Poly1305 = 2,
}

impl CipherSuite {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Aes256Gcm),
            2 => Some(Self::ChaCha20Poly1305),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Aes256Gcm => "AES-256-GCM",
            Self::ChaCha20Poly1305 => "ChaCha20-Poly1305",
        }
    }
}

pub fn encrypt(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, AeadError> {
    encrypt_with(CipherSuite::Aes256Gcm, key, plaintext, aad)
}

pub fn decrypt(key: &[u8; 32], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, AeadError> {
    decrypt_auto(key, ciphertext, aad)
}

pub fn encrypt_with(
    suite: CipherSuite,
    key: &[u8; 32],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, AeadError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let ct = match suite {
        CipherSuite::Aes256Gcm => {
            let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| AeadError::Encrypt)?;
            cipher.encrypt(Nonce::from_slice(&nonce_bytes), Payload { msg: plaintext, aad }).map_err(|_| AeadError::Encrypt)?
        }
        CipherSuite::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|_| AeadError::Encrypt)?;
            cipher.encrypt(ChaNonce::from_slice(&nonce_bytes), Payload { msg: plaintext, aad }).map_err(|_| AeadError::Encrypt)?
        }
    };
    let mut out = Vec::with_capacity(1 + NONCE_LEN + ct.len());
    out.push(suite as u8);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

pub fn decrypt_auto(key: &[u8; 32], data: &[u8], aad: &[u8]) -> Result<Vec<u8>, AeadError> {
    if data.len() > 1 + NONCE_LEN {
        if let Some(suite) = CipherSuite::from_u8(data[0]) {
            return open(suite, key, &data[1..1 + NONCE_LEN], &data[1 + NONCE_LEN..], aad);
        }
    }
    if data.len() > NONCE_LEN {
        return open(CipherSuite::Aes256Gcm, key, &data[..NONCE_LEN], &data[NONCE_LEN..], aad);
    }
    Err(AeadError::InvalidLength)
}

fn open(suite: CipherSuite, key: &[u8; 32], nonce_bytes: &[u8], ct: &[u8], aad: &[u8]) -> Result<Vec<u8>, AeadError> {
    match suite {
        CipherSuite::Aes256Gcm => {
            let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| AeadError::Decrypt)?;
            cipher.decrypt(Nonce::from_slice(nonce_bytes), Payload { msg: ct, aad }).map_err(|_| AeadError::Decrypt)
        }
        CipherSuite::ChaCha20Poly1305 => {
            let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|_| AeadError::Decrypt)?;
            cipher.decrypt(ChaNonce::from_slice(nonce_bytes), Payload { msg: ct, aad }).map_err(|_| AeadError::Decrypt)
        }
    }
}

pub fn preferred_suite() -> CipherSuite { CipherSuite::Aes256Gcm }
pub fn available_suites() -> &'static [CipherSuite] {
    &[CipherSuite::Aes256Gcm, CipherSuite::ChaCha20Poly1305]
}
