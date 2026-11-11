//! P2P layer for Liberty Messenger V2
//!
//! ## Production path (default) — **honest relay**
//! Server `WsHub` fans out **ciphertext only** to chat members over WebSocket.
//! No plaintext on the wire or in the database. Clients encrypt with AES-GCM
//! (`lr1:`) before POST.
//!
//! ## In-process mesh (always available)
//! [`LocalMesh`] is a process-local pub/sub that mirrors Gossipsub topic
//! semantics (`liberty/chat/{id}`). Used for unit tests and same-process
//! multi-peer demos without network deps.
//!
//! ## Optional libp2p (`--features p2p`)
//! Real Swarm + Gossipsub needs **Rust ≥ ~1.85** (transitive `edition2024`) and
//! a working crates mirror. On older toolchains the feature is reserved:
//!
//! ```text
//! # messenger/Cargo.toml — uncomment libp2p dep, then:
//! cargo build -p liberty-messenger --features p2p
//! ```
//!
//! Payload is always client-encrypted; P2P never sees plaintext.

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::{broadcast, RwLock};

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("p2p disabled — use server WsHub relay, LocalMesh, or build with --features p2p")]
    Disabled,
    #[error("network: {0}")]
    Network(String),
    #[error("topic: {0}")]
    Topic(String),
    #[error("not subscribed: {0}")]
    NotSubscribed(String),
}

/// Lightweight node identity (relay client or future Gossipsub peer).
#[derive(Clone, Debug)]
pub struct P2pNode {
    pub peer_id: String,
}

/// Incoming message from a topic.
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

    pub fn has_gossipsub() -> bool {
        cfg!(feature = "p2p")
    }

    #[cfg(feature = "p2p")]
    async fn publish_gossipsub(&self, topic: &str, data: &[u8]) -> Result<(), P2pError> {
        let _ = (topic, data);
        Err(P2pError::Network(
            "feature p2p set but Swarm not linked — uncomment libp2p in Cargo.toml on Rust ≥1.85"
                .into(),
        ))
    }

    #[cfg(feature = "p2p")]
    async fn subscribe_gossipsub(&self, topic: &str) -> Result<(), P2pError> {
        let _ = topic;
        Err(P2pError::Network(
            "feature p2p set but Swarm not linked — uncomment libp2p in Cargo.toml on Rust ≥1.85"
                .into(),
        ))
    }
}

impl Default for P2pNode {
    fn default() -> Self {
        Self::new()
    }
}

pub fn chat_topic(chat_id: &str) -> String {
    format!("liberty/chat/{}", chat_id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportMode {
    Relay,
    Gossipsub,
    Hybrid,
    Local,
}

impl TransportMode {
    pub fn effective(self) -> TransportMode {
        match self {
            TransportMode::Gossipsub | TransportMode::Hybrid if P2pNode::has_gossipsub() => self,
            TransportMode::Local => TransportMode::Local,
            _ => TransportMode::Relay,
        }
    }
}

struct MeshInner {
    topics: HashMap<String, broadcast::Sender<P2pMessage>>,
}

#[derive(Clone)]
pub struct LocalMesh {
    inner: Arc<RwLock<MeshInner>>,
}

impl LocalMesh {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MeshInner {
                topics: HashMap::new(),
            })),
        }
    }

    pub async fn subscribe(&self, topic: &str) -> Result<broadcast::Receiver<P2pMessage>, P2pError> {
        if topic.is_empty() {
            return Err(P2pError::Topic("empty topic".into()));
        }
        let mut g = self.inner.write().await;
        let tx = g
            .topics
            .entry(topic.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            })
            .clone();
        Ok(tx.subscribe())
    }

    pub async fn publish(
        &self,
        topic: &str,
        data: &[u8],
        from: Option<&str>,
    ) -> Result<usize, P2pError> {
        if topic.is_empty() {
            return Err(P2pError::Topic("empty topic".into()));
        }
        let g = self.inner.read().await;
        let Some(tx) = g.topics.get(topic) else {
            return Ok(0);
        };
        let msg = P2pMessage {
            topic: topic.to_string(),
            data: data.to_vec(),
            from: from.map(str::to_string),
        };
        let n = tx.receiver_count();
        let _ = tx.send(msg);
        Ok(n)
    }

    pub async fn ensure_topic(&self, topic: &str) -> Result<(), P2pError> {
        if topic.is_empty() {
            return Err(P2pError::Topic("empty topic".into()));
        }
        let mut g = self.inner.write().await;
        g.topics.entry(topic.to_string()).or_insert_with(|| {
            let (tx, _) = broadcast::channel(256);
            tx
        });
        Ok(())
    }
}

impl Default for LocalMesh {
    fn default() -> Self {
        Self::new()
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
        assert_eq!(TransportMode::Local.effective(), TransportMode::Local);
    }

    #[tokio::test]
    async fn local_mesh_fanout() {
        let mesh = LocalMesh::new();
        let topic = chat_topic("room-1");
        mesh.ensure_topic(&topic).await.unwrap();
        let mut a = mesh.subscribe(&topic).await.unwrap();
        let mut b = mesh.subscribe(&topic).await.unwrap();
        let n = mesh
            .publish(&topic, b"lr1:ciphertext", Some("peer-alice"))
            .await
            .unwrap();
        assert!(n >= 2);
        let ma = a.recv().await.unwrap();
        let mb = b.recv().await.unwrap();
        assert_eq!(ma.data, b"lr1:ciphertext");
        assert_eq!(mb.from.as_deref(), Some("peer-alice"));
        assert_eq!(ma.topic, topic);
    }

    #[tokio::test]
    async fn local_mesh_no_subs_ok() {
        let mesh = LocalMesh::new();
        let n = mesh
            .publish(&chat_topic("empty"), b"x", None)
            .await
            .unwrap();
        assert_eq!(n, 0);
    }
}
