//! Cryptographic primitives for Liberty Messenger

mod aead;
mod disappearing;
mod keys;
mod media;
mod mls;
mod pq;
mod ratchet;
mod sealed_sender;
mod session;
mod x3dh;

pub use aead::{
    available_suites, decrypt, decrypt_auto, encrypt, encrypt_with, AeadError, CipherSuite,
};
pub use disappearing::{sweep_expired, DisappearMode, EphemeralMessage};
pub use keys::{DiffieHellman, EphemeralKeyPair, IdentityKeyPair, PublicIdentity, SessionKeys};
pub use media::{MediaAttachment, MediaCrypto};
pub use mls::{Commit, KeyPackage, MlsGroup, MlsMessage, Welcome};
pub use pq::{HybridKeyExchange, PqKeyPair, PqPublicKey, PqSharedSecret};
pub use ratchet::{MessageHeader, RatchetSession};
pub use sealed_sender::{
    SealedEnvelope, SealedSender, SealedSenderKeys, SealedSenderSecret, UnsealedSender,
};
pub use session::{EncryptedMessage, SessionManager};
pub use x3dh::{PreKeyBundle, X3DH};

use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(v: Vec<u8>) -> Self {
        Self::new(v)
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
