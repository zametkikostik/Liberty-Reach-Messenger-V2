//! Network layer: transport, protocol, swarm, federation

mod transport;
mod messages;
mod libp2p_transport;
mod protocol;
mod swarm;
mod federation;

pub use transport::{Transport, LocalTransport, PeerInfo};
pub use messages::{NetworkMessage, MessageType};
pub use libp2p_transport::Libp2pTransport;
pub use protocol::{WireMessage, PresenceStatus, topics};
pub use swarm::SwarmHandle;
pub use federation::{FederationConfig, FederationMode, FederationPeer, get_config as federation_config, set_config as set_federation_config, add_peer as federation_add_peer, bootstrap_addrs, mesh_enabled, export_json as federation_export, import_json as federation_import};

use thiserror::Error;
use crate::crypto::EncryptedMessage;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("send failed: {0}")]
    Send(String),
    #[error("not connected to peer")]
    NotConnected,
    #[error("serialization error")]
    Serialize,
    #[error("not implemented yet")]
    NotImplemented,
}

pub struct MessengerNode {
    peer_id: String,
    transport: Box<dyn Transport + Send + Sync>,
}

impl MessengerNode {
    pub fn new(peer_id: String, transport: Box<dyn Transport + Send + Sync>) -> Self {
        Self { peer_id, transport }
    }
    pub fn peer_id(&self) -> &str { &self.peer_id }

    pub async fn send_encrypted(&self, to: &str, msg: &EncryptedMessage) -> Result<(), NetworkError> {
        let net_msg = NetworkMessage {
            from: self.peer_id.clone(),
            to: to.to_string(),
            msg_type: MessageType::Chat,
            payload: serde_json::to_vec(msg).map_err(|_| NetworkError::Serialize)?,
            timestamp: msg.timestamp,
        };
        self.transport.send(to, &net_msg).await
    }

    pub async fn receive(&self) -> Result<Option<NetworkMessage>, NetworkError> {
        self.transport.receive().await
    }

    pub async fn connect(&self, peer: &PeerInfo) -> Result<(), NetworkError> {
        self.transport.connect(peer).await
    }
}
