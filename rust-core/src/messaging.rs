//! High-level messaging service combining sessions + transport
use crate::crypto::{SessionManager, EncryptedMessage, IdentityKeyPair};
use crate::network::{MessengerNode, LocalTransport, NetworkError};
use std::sync::Arc;

pub struct MessagingService {
    pub peer_id: String,
    sessions: SessionManager,
    node: Option<MessengerNode>,
}

impl MessagingService {
    pub fn new(identity: IdentityKeyPair, peer_id: String) -> Self {
        Self {
            peer_id: peer_id.clone(),
            sessions: SessionManager::new(identity),
            node: None,
        }
    }

    pub fn with_local_transport(mut self) -> Self {
        let transport = LocalTransport::new(&self.peer_id);
        self.node = Some(MessengerNode::new(self.peer_id.clone(), Box::new(transport)));
        self
    }

    pub fn sessions_mut(&mut self) -> &mut SessionManager {
        &mut self.sessions
    }

    pub async fn send(&mut self, to: &str, plaintext: &[u8]) -> Result<EncryptedMessage, String> {
        let msg = self.sessions.encrypt_for(to, plaintext, None).map_err(|e| e.to_string())?;
        if let Some(ref node) = self.node {
            node.send_encrypted(to, &msg).await.map_err(|e| e.to_string())?;
        }
        Ok(msg)
    }
}
