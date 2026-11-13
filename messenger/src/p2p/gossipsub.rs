//! libp2p Gossipsub backend (feature `p2p`, Rust ≥ ~1.85).
//!
//! Topics: `liberty/chat/{chat_id}` — payload is always client ciphertext (`lr1:`).
//! ```text
//! cargo build -p liberty-messenger --features p2p
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use libp2p::gossipsub::{
    self, IdentTopic, MessageAuthenticity, ValidationMode,
};
use libp2p::identity::Keypair;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{noise, tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder};
use thiserror::Error;
use tokio::sync::mpsc;

use super::{P2pError, P2pMessage};

#[derive(NetworkBehaviour)]
struct LibertyBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: libp2p::identify::Behaviour,
    ping: libp2p::ping::Behaviour,
}

#[derive(Error, Debug)]
pub enum GossipError {
    #[error("swarm: {0}")]
    Swarm(String),
    #[error("publish: {0}")]
    Publish(String),
    #[error("subscribe: {0}")]
    Subscribe(String),
}

impl From<GossipError> for P2pError {
    fn from(e: GossipError) -> Self {
        P2pError::Network(e.to_string())
    }
}

pub struct GossipNode {
    pub peer_id: PeerId,
    swarm: Swarm<LibertyBehaviour>,
    event_tx: mpsc::UnboundedSender<P2pMessage>,
}

impl GossipNode {
    pub fn new(listen: &str) -> Result<(Self, mpsc::UnboundedReceiver<P2pMessage>), GossipError> {
        let id_keys = Keypair::generate_ed25519();
        let peer_id = id_keys.public().to_peer_id();

        let message_id_fn = |message: &gossipsub::Message| {
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            gossipsub::MessageId::from(hasher.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|e| GossipError::Swarm(e.to_string()))?;

        let gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub_config,
        )
        .map_err(|e| GossipError::Swarm(e.to_string()))?;

        let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
            "/liberty/1.0.0".into(),
            id_keys.public(),
        ));
        let ping = libp2p::ping::Behaviour::default();

        let behaviour = LibertyBehaviour {
            gossipsub,
            identify,
            ping,
        };

        let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| GossipError::Swarm(e.to_string()))?
            .with_behaviour(|_| behaviour)
            .map_err(|e| GossipError::Swarm(e.to_string()))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        let addr: Multiaddr = listen
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| GossipError::Swarm(e.to_string()))?;
        swarm
            .listen_on(addr)
            .map_err(|e| GossipError::Swarm(e.to_string()))?;

        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok((Self { peer_id, swarm, event_tx }, event_rx))
    }

    pub fn subscribe(&mut self, topic: &str) -> Result<(), GossipError> {
        let t = IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&t)
            .map_err(|e| GossipError::Subscribe(e.to_string()))?;
        Ok(())
    }

    pub fn publish(&mut self, topic: &str, data: &[u8]) -> Result<(), GossipError> {
        let t = IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(t, data.to_vec())
            .map_err(|e| GossipError::Publish(e.to_string()))?;
        Ok(())
    }

    pub fn dial(&mut self, addr: &str) -> Result<(), GossipError> {
        let a: Multiaddr = addr
            .parse()
            .map_err(|e: libp2p::multiaddr::Error| GossipError::Swarm(e.to_string()))?;
        self.swarm
            .dial(a)
            .map_err(|e| GossipError::Swarm(e.to_string()))?;
        Ok(())
    }

    pub async fn run(mut self) {
        use futures::StreamExt;
        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::Behaviour(LibertyBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source,
                        message,
                        ..
                    },
                )) => {
                    let _ = self.event_tx.send(P2pMessage {
                        topic: message.topic.to_string(),
                        data: message.data,
                        from: Some(propagation_source.to_string()),
                    });
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    tracing::info!(%address, peer = %self.peer_id, "gossipsub listening");
                }
                _ => {}
            }
        }
    }
}
