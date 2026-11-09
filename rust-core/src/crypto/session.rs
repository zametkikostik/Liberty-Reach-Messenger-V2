//! Session management: X3DH + Double Ratchet

use crate::crypto::{
    AeadError, EphemeralKeyPair, IdentityKeyPair, MessageHeader, PreKeyBundle, RatchetSession,
    SessionKeys, X3DH,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncryptedMessage {
    pub header: MessageHeader,
    pub ciphertext: Vec<u8>,
    pub timestamp: i64,
    pub initiator_ephemeral: Option<[u8; 32]>,
    /// Sender's long-term X25519 identity public. Travels with the first
    /// messages so the responder can bootstrap its session via X3DH respond
    /// without a second QR exchange (public key only — no secret).
    #[serde(default)]
    pub sender_identity_x25519: Option<[u8; 32]>,
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
        let (keys, our_eph_pub, _our_eph) =
            X3DH::initiate(&self.our_identity, their_bundle).map_err(|_| AeadError::Encrypt)?;
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
        )
        .map_err(|_| AeadError::Decrypt)?;
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
        // AAD is deliberately empty at this level: each side keys its session
        // map by the OTHER side's id, so a peer_id AAD would differ between
        // peers and break authentication. Identity binding already comes from
        // X3DH; ratchet.encrypt still binds (dh_public, n) into the AEAD AAD.
        let (header, ciphertext) = session.encrypt(plaintext, b"")?;
        Ok(EncryptedMessage {
            header,
            ciphertext,
            timestamp: chrono::Utc::now().timestamp(),
            initiator_ephemeral,
            sender_identity_x25519: self.our_bundle.as_ref().map(|b| b.identity_x25519),
        })
    }

    pub fn decrypt_from(
        &mut self,
        peer_id: &str,
        msg: &EncryptedMessage,
    ) -> Result<Vec<u8>, AeadError> {
        let session = self.sessions.get_mut(peer_id).ok_or(AeadError::Decrypt)?;
        // See encrypt_for: AAD is empty; binding comes from X3DH + ratchet header.
        session.decrypt(&msg.header, &msg.ciphertext, b"")
    }

    pub fn has_session(&self, peer_id: &str) -> bool {
        self.sessions.contains_key(peer_id)
    }

    /// Decrypt, bootstrapping a responder session from the first received
    /// message (X3DH respond) when no session exists for that peer yet.
    /// Requires the message to carry `initiator_ephemeral` and
    /// `sender_identity_x25519` (both public values).
    pub fn decrypt_smart(
        &mut self,
        peer_id: &str,
        msg: &EncryptedMessage,
    ) -> Result<Vec<u8>, AeadError> {
        if !self.has_session(peer_id) {
            let eph = msg.initiator_ephemeral.as_ref().ok_or(AeadError::Decrypt)?;
            let sender_ik = msg
                .sender_identity_x25519
                .as_ref()
                .ok_or(AeadError::Decrypt)?;
            self.start_as_responder(peer_id, sender_ik, eph)?;
        }
        self.decrypt_from(peer_id, msg)
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
