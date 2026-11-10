//! P2P: server-relay by default; libp2p Gossipsub behind feature flag later

use thiserror::Error;

#[derive(Error, Debug)]
pub enum P2pError {
    #[error("p2p/relay error: {0}")]
    Network(String),
}

pub struct P2pNode {
    pub peer_id: String,
}

impl P2pNode {
    pub fn relay() -> Self {
        Self {
            peer_id: format!("relay-{}", uuid::Uuid::new_v4()),
        }
    }

    pub async fn publish(&self, _topic: &str, _data: &[u8]) -> Result<(), P2pError> {
        Ok(())
    }
}

impl Default for P2pNode {
    fn default() -> Self {
        Self::relay()
    }
}
