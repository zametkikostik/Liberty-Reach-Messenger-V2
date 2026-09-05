//! Libp2p transport placeholder (full Gossipsub when feature enabled)
use crate::network::{Transport, PeerInfo, NetworkMessage, NetworkError};
use async_trait::async_trait;

pub struct Libp2pTransport {
    peer_id: String,
}

impl Libp2pTransport {
    pub fn new(peer_id: &str) -> Self {
        Self { peer_id: peer_id.to_string() }
    }
}

#[async_trait]
impl Transport for Libp2pTransport {
    async fn connect(&self, _peer: &PeerInfo) -> Result<(), NetworkError> {
        Ok(())
    }

    async fn send(&self, _to: &str, _msg: &NetworkMessage) -> Result<(), NetworkError> {
        Err(NetworkError::NotImplemented)
    }

    async fn receive(&self) -> Result<Option<NetworkMessage>, NetworkError> {
        Ok(None)
    }

    fn local_peer_id(&self) -> &str {
        &self.peer_id
    }
}
