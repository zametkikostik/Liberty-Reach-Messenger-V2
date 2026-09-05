//! Cryptographic primitives for Liberty Messenger

mod keys;
mod aead;
mod ratchet;
mod session;
mod x3dh;
mod disappearing;
mod mls;
mod pq;
mod sealed_sender;
mod media;

pub use keys::{IdentityKeyPair, SessionKeys, EphemeralKeyPair, DiffieHellman, PublicIdentity};
pub use aead::{encrypt, decrypt, encrypt_with, decrypt_auto, AeadError, CipherSuite, available_suites};
pub use ratchet::{RatchetSession, MessageHeader};
pub use session::{SessionManager, EncryptedMessage};
pub use x3dh::{X3DH, PreKeyBundle};
pub use disappearing::{EphemeralMessage, DisappearMode, sweep_expired};
pub use mls::{MlsGroup, MlsMessage, KeyPackage, Commit, Welcome};
pub use pq::{HybridKeyExchange, PqPublicKey, PqSharedSecret, PqKeyPair};
pub use sealed_sender::{SealedSender, SealedEnvelope, SealedSenderKeys, SealedSenderSecret, UnsealedSender};
pub use media::{MediaCrypto, MediaAttachment};

use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(data: Vec<u8>) -> Self { Self(data) }
    pub fn as_slice(&self) -> &[u8] { &self.0 }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}

impl From<Vec<u8>> for SecretBytes {
    fn from(v: Vec<u8>) -> Self { Self::new(v) }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] { &self.0 }
}
