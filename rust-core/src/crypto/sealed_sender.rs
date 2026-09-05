//! Sealed Sender — hide sender identity from transport observers

use crate::crypto::{IdentityKeyPair, EphemeralKeyPair, encrypt, decrypt, AeadError};
use serde::{Serialize, Deserialize};
use zeroize::Zeroize;
use sha2::Sha256;
use hkdf::Hkdf;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SealedSenderKeys {
    pub public: [u8; 32],
    pub certificate: Vec<u8>,
    pub identity_ed25519: [u8; 32],
}

#[derive(Zeroize)]
pub struct SealedSenderSecret {
    pub secret: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SealedEnvelope {
    pub sealed_sender: Vec<u8>,
    pub ephemeral_public: [u8; 32],
    pub payload: Vec<u8>,
    pub recipient_hint: Option<[u8; 8]>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UnsealedSender {
    pub peer_id: String,
    pub identity_ed25519: [u8; 32],
}

pub struct SealedSender;

impl SealedSender {
    pub fn create_keys(identity: &IdentityKeyPair) -> (SealedSenderSecret, SealedSenderKeys) {
        let eph = EphemeralKeyPair::generate();
        let secret = SealedSenderSecret { secret: eph.secret_bytes() };
        let public = eph.public_bytes();
        let cert = identity.sign(&public).to_bytes().to_vec();
        let keys = SealedSenderKeys {
            public,
            certificate: cert,
            identity_ed25519: identity.public_key_bytes(),
        };
        (secret, keys)
    }

    pub fn seal(
        sender_peer_id: &str,
        sender_identity: &IdentityKeyPair,
        recipient_sealed_pk: &[u8; 32],
        payload_ciphertext: Vec<u8>,
        recipient_hint: Option<[u8; 8]>,
    ) -> Result<SealedEnvelope, AeadError> {
        let eph = EphemeralKeyPair::generate();
        let shared = eph.diffie_hellman(recipient_sealed_pk);
        let key = derive_sealed_key(shared.as_slice());
        let inner = serde_json::json!({
            "peer_id": sender_peer_id,
            "identity_ed25519": hex::encode(sender_identity.public_key_bytes()),
        });
        let plaintext = serde_json::to_vec(&inner).map_err(|_| AeadError::Encrypt)?;
        let sealed = encrypt(&key, &plaintext, b"sealed-sender-v1")?;
        Ok(SealedEnvelope {
            sealed_sender: sealed,
            ephemeral_public: eph.public_bytes(),
            payload: payload_ciphertext,
            recipient_hint,
        })
    }

    pub fn open(
        envelope: &SealedEnvelope,
        recipient_secret: &[u8; 32],
    ) -> Result<(UnsealedSender, &[u8]), AeadError> {
        let eph = EphemeralKeyPair::from_secret(*recipient_secret);
        let shared = eph.diffie_hellman(&envelope.ephemeral_public);
        let key = derive_sealed_key(shared.as_slice());
        let plain = decrypt(&key, &envelope.sealed_sender, b"sealed-sender-v1")?;
        let v: serde_json::Value = serde_json::from_slice(&plain).map_err(|_| AeadError::Decrypt)?;
        let peer_id = v["peer_id"].as_str().unwrap_or("").to_string();
        let id_hex = v["identity_ed25519"].as_str().unwrap_or("");
        let mut identity_ed25519 = [0u8; 32];
        if let Ok(bytes) = hex::decode(id_hex) {
            if bytes.len() == 32 { identity_ed25519.copy_from_slice(&bytes); }
        }
        Ok((UnsealedSender { peer_id, identity_ed25519 }, &envelope.payload))
    }
}

fn derive_sealed_key(shared: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, shared);
    let mut key = [0u8; 32];
    hk.expand(b"Liberty-SealedSender-v1", &mut key).expect("HKDF");
    key
}
