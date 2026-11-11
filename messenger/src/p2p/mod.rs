//! P2P layer for Liberty Messenger V2
//!
//! ## Production path (default)
//! **Honest relay** via server `WsHub` — ciphertext-only fan-out to chat members.
//! No fake decentralisation: the server is a pure relay of opaque payloads.
//!
//! ## Optional path (`--features p2p`)
//! libp2p Gossipsub + Kademlia + mDNS when those crates are available on the host.
//!
//! ```text
//! cargo build -p liberty-messenger --features p2p
//! ```
//!
//! Topic layout: `liberty/chat/{chat_id}` for message fan-out.
//! Payload is always client-encrypted (`lr1:` AES-GCM or hybrid session).

use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("p2p disabled — build with --features p2p (or use server WsHub relay)")]
    Disabled,
    #[error("network: {0}")]
    Network(String),
    #[error("topic: {0}")]
    Topic(String),
    #[error("not subscribed: {0}")]
    NotSubscribed(String),
}

/// Lightweight node identity used by both relay clients and future Gossipsub peers.
#[derive(Clone, Debug)]
pub struct P2pNode {
    pub peer_id: String,
}

/// Incoming message from a topic (Gossipsub or local test harness).
#[derive(Clone, Debug)]
pub struct P2pMessage {
    pub topic: String,
    pub data: Vec<u8>,
    pub from: Option<String>,
}

impl P2pNode {
    pub fn new() -> Self {
        Self {
            peer_id: format!("peer-{}", uuid::Uuid::new_v4()),
        }
    }

    /// Publish to a topic.
    ///
    /// Without `p2p` feature this is a no-op success — actual delivery is the
    /// server WebSocket hub (`WsHub::publish`). Callers in the app should prefer
    /// the REST send + hub path; this method keeps the API stable for a future
    /// libp2p backend.
    pub async fn publish(&self, topic: &str, data: &[u8]) -> Result<(), P2pError> {
        if topic.is_empty() {
            return Err(P2pError::Topic("empty topic".into()));
        }
        #[cfg(feature = "p2p")]
        {
            return self.publish_gossipsub(topic, data).await;
        }
        #[cfg(not(feature = "p2p"))]
        {
            let _ = data;
            Ok(())
        }
    }

    /// Subscribe to a topic. Without `p2p` returns Disabled — use server WS join.
    pub async fn subscribe(&self, topic: &str) -> Result<(), P2pError> {
        if topic.is_empty() {
            return Err(P2pError::Topic("empty topic".into()));
        }
        #[cfg(feature = "p2p")]
        {
            return self.subscribe_gossipsub(topic).await;
        }
        #[cfg(not(feature = "p2p"))]
        {
            Err(P2pError::Disabled)
        }
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// True when this binary was built with real libp2p Gossipsub.
    pub fn has_gossipsub() -> bool {
        cfg!(feature = "p2p")
    }

    // ── Feature-gated Gossipsub hooks (fill when libp2p is on the host) ──

    #[cfg(feature = "p2p")]
    async fn publish_gossipsub(&self, topic: &str, data: &[u8]) -> Result<(), P2pError> {
        // Placeholder for:
        //   swarm.behaviour_mut().gossipsub.publish(IdentTopic::new(topic), data)
        // Wire Swarm + Gossipsub in a dedicated runtime task when deps land.
        let _ = (topic, data);
        Err(P2pError::Network(
            "libp2p Gossipsub not yet wired — enable crate deps and implement Swarm".into(),
        ))
    }

    #[cfg(feature = "p2p")]
    async fn subscribe_gossipsub(&self, topic: &str) -> Result<(), P2pError> {
        let _ = topic;
        Err(P2pError::Network(
            "libp2p Gossipsub not yet wired — enable crate deps and implement Swarm".into(),
        ))
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

/// Transport preference for a deployment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportMode {
    /// Server WsHub only (default, always available)
    Relay,
    /// Direct libp2p when feature `p2p` is compiled in
    Gossipsub,
    /// Prefer Gossipsub, fall back to relay
    Hybrid,
}

impl TransportMode {
    pub fn effective(self) -> TransportMode {
        match self {
            TransportMode::Gossipsub | TransportMode::Hybrid if P2pNode::has_gossipsub() => self,
            _ => TransportMode::Relay,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_empty_topic_fails() {
        let n = P2pNode::new();
        assert!(matches!(n.publish("", b"x").await, Err(P2pError::Topic(_))));
    }

    #[tokio::test]
    async fn publish_ok_without_p2p() {
        let n = P2pNode::new();
        if !P2pNode::has_gossipsub() {
            assert!(n.publish(&chat_topic("c1"), b"cipher").await.is_ok());
        }
    }

    #[test]
    fn transport_defaults_to_relay() {
        assert_eq!(TransportMode::Gossipsub.effective(), TransportMode::Relay);
        assert_eq!(TransportMode::Relay.effective(), TransportMode::Relay);
    }
}
