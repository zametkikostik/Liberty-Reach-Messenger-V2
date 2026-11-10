//! P2P layer for Liberty Messenger V2
//!
//! Current production path: **honest relay** via server `WsHub`
//! (ciphertext-only fan-out to chat members). No fake decentralisation.
//!
//! Optional path (feature `p2p`): libp2p Gossipsub + Kademlia + mDNS.
//! Enable with: `cargo build -p liberty-messenger --features p2p`
//! when libp2p dependencies are available on the host.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("p2p disabled — build with --features p2p (or use server WsHub relay)")]
    Disabled,
    #[error("network: {0}")]
    Network(String),
    #[error("topic: {0}")]
    Topic(String),
}

/// Lightweight node identity used by both relay clients and future Gossipsub peers.
#[derive(Clone, Debug)]
pub struct P2pNode {
    pub peer_id: String,
}

impl P2pNode {
    pub fn new() -> Self {
        Self {
            peer_id: format!("peer-{}", uuid::Uuid::new_v4()),
        }
    }

    /// Publish to a topic. Without `p2p` feature this is a no-op success
    /// (actual delivery is the server WebSocket hub).
    pub async fn publish(&self, topic: &str, data: &[u8]) -> Result<(), P2pError> {
        if topic.is_empty() {
            return Err(P2pError::Topic("empty topic".into()));
        }
        // Honest relay path: callers should prefer server WsHub.publish.
        // This stub keeps the API stable for a future libp2p backend.
        let _ = data;
        Ok(())
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }
}

impl Default for P2pNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Topic naming helper: chat rooms map to deterministic topics.
pub fn chat_topic(chat_id: &str) -> String {
    format!("liberty/chat/{}", chat_id)
}
