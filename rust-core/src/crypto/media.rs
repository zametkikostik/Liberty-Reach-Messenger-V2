//! Media / file encryption
use crate::crypto::{encrypt, decrypt, AeadError, CipherSuite, encrypt_with};
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use serde::{Serialize, Deserialize};
use rand::RngCore;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MediaAttachment {
    pub id: String,
    pub mime: String,
    pub size: u64,
    pub filename: String,
    pub ciphertext: Vec<u8>,
    pub suite: u8,
}

pub struct MediaCrypto;

impl MediaCrypto {
    pub fn derive_key(chain_key: &[u8; 32], attachment_id: &str) -> [u8; 32] {
        let hk = Hkdf::<Sha256>::new(Some(attachment_id.as_bytes()), chain_key);
        let mut key = [0u8; 32];
        hk.expand(b"Liberty-Media-v1", &mut key).expect("HKDF");
        key
    }

    pub fn encrypt_file(
        chain_key: &[u8; 32],
        filename: &str,
        mime: &str,
        data: &[u8],
    ) -> Result<MediaAttachment, AeadError> {
        let mut id_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut id_bytes);
        let id = hex::encode(id_bytes);
        let key = Self::derive_key(chain_key, &id);
        let ct = encrypt_with(CipherSuite::Aes256Gcm, &key, data, id.as_bytes())?;
        Ok(MediaAttachment {
            id,
            mime: mime.to_string(),
            size: data.len() as u64,
            filename: filename.to_string(),
            ciphertext: ct,
            suite: CipherSuite::Aes256Gcm as u8,
        })
    }

    pub fn decrypt_file(
        chain_key: &[u8; 32],
        attachment: &MediaAttachment,
    ) -> Result<Vec<u8>, AeadError> {
        let key = Self::derive_key(chain_key, &attachment.id);
        decrypt(&key, &attachment.ciphertext, attachment.id.as_bytes())
    }
}
