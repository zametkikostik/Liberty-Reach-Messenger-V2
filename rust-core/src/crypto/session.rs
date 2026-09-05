//! Session management: X3DH + Double Ratchet

use crate::crypto::{
    RatchetSession, MessageHeader, SessionKeys, AeadError,
    IdentityKeyPair, EphemeralKeyPair, X3DH, PreKeyBundle,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncryptedMessage {
    pub header: MessageHeader,
    pub ciphertext: Vec<u8>,
    pub timestamp: i64,
    pub initiator_ephemeral: Option<[u8; 32]>,
}

pub struct SessionManager {
    sessions: HashMap<String, RatchetSession>,
    our_identity: IdentityKeyPair,
    our_bundle: Option<PreKeyBundle>,
    our_signed_prekey: Option<EphemeralKeyPair>,
}

impl SessionManager {
    pub fn new(our_identity: IdentityKeyPair) -> Self {
        let (bundle, signed_prekey) = X3DH::create_bundle(&our_identity);
        Self {
            sessions: HashMap::new(),
            our_identity,
            our_bundle: Some(bundle),
            our_signed_prekey: Some(signed_prekey),
        }
    }

    pub fn our_prekey_bundle(&self) -> Option<&PreKeyBundle> {
        self.our_bundle.as_ref()
    }

    pub fn start_as_initiator(
        &mut self,
        peer_id: &str,
        their_bundle: &PreKeyBundle,
    ) -> Result<[u8; 32], AeadError> {
        let (keys, our_eph_pub, _our_eph) = X3DH::initiate(&self.our_identity, their_bundle)
            .map_err(|_| AeadError::Encrypt)?;
        let session = RatchetSession::new(keys, true);
        self.sessions.insert(peer_id.to_string(), session);
        Ok(our_eph_pub)
    }

    pub fn start_as_responder(
        &mut self,
        peer_id: &str,
        their_identity_x25519: &[u8; 32],
        their_ephemeral: &[u8; 32],
    ) -> Result<(), AeadError> {
        let our_spk = self.our_signed_prekey.as_ref().ok_or(AeadError::Decrypt)?;
        let keys = X3DH::respond(
            &self.our_identity,
            our_spk,
            their_identity_x25519,
            their_ephemeral,
        ).map_err(|_| AeadError::Decrypt)?;
        let session = RatchetSession::new(keys, false);
        self.sessions.insert(peer_id.to_string(), session);
        Ok(())
    }

    pub fn start_session(&mut self, peer_id: &str, shared: SessionKeys, is_initiator: bool) {
        let session = RatchetSession::new(shared, is_initiator);
        self.sessions.insert(peer_id.to_string(), session);
    }

    pub fn encrypt_for(
        &mut self,
        peer_id: &str,
        plaintext: &[u8],
        initiator_ephemeral: Option<[u8; 32]>,
    ) -> Result<EncryptedMessage, AeadError> {
        let session = self.sessions.get_mut(peer_id).ok_or(AeadError::Encrypt)?;
        let aad = peer_id.as_bytes();
        let (header, ciphertext) = session.encrypt(plaintext, aad)?;
        Ok(EncryptedMessage {
            header,
            ciphertext,
            timestamp: chrono::Utc::now().timestamp(),
            initiator_ephemeral,
        })
    }

    pub fn decrypt_from(&mut self, peer_id: &str, msg: &EncryptedMessage) -> Result<Vec<u8>, AeadError> {
        let session = self.sessions.get_mut(peer_id).ok_or(AeadError::Decrypt)?;
        let aad = peer_id.as_bytes();
        session.decrypt(&msg.header, &msg.ciphertext, aad)
    }

    pub fn has_session(&self, peer_id: &str) -> bool {
        self.sessions.contains_key(peer_id)
    }

    pub fn save_peer_list(&self, path: &Path) -> std::io::Result<()> {
        let peers: Vec<String> = self.sessions.keys().cloned().collect();
        let data = serde_json::to_vec_pretty(&peers).unwrap_or_default();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)
    }
}
