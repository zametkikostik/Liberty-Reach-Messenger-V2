//! Post-quantum ready hybrid crypto + steganography + signatures

mod hybrid;
mod kyber_hybrid;
mod signatures;
mod steganography;

pub use hybrid::{Ciphertext, HybridCipher, SessionKeys};
pub use kyber_hybrid::{HybridHandshakeReply, HybridIdentity, HybridPublicBundle};
pub use signatures::{PublicSignKey, SignKeypair};
pub use steganography::Steganography;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("encrypt: {0}")]
    Encrypt(String),
    #[error("decrypt: {0}")]
    Decrypt(String),
    #[error("kdf: {0}")]
    Kdf(String),
    #[error("signature: {0}")]
    Signature(String),
    #[error("stego: {0}")]
    Stego(String),
    #[error("invalid: {0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;
