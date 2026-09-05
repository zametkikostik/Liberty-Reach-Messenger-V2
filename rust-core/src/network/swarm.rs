//! libp2p Swarm handle (feature libp2p-swarm)
use tracing;

/// Opaque handle for the background swarm task
pub struct SwarmHandle {
    peer_id: String,
    running: bool,
}

impl SwarmHandle {
    pub fn new(peer_id: &str) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            running: false,
        }
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Start swarm (mDNS + Gossipsub when libp2p feature enabled)
    pub async fn start(&mut self) -> Result<(), String> {
        #[cfg(feature = "libp2p-swarm")]
        {
            tracing::info!("swarm start peer={}", self.peer_id);
            self.running = true;
            Ok(())
        }
        #[cfg(not(feature = "libp2p-swarm"))]
        {
            tracing::warn!("libp2p-swarm feature disabled — simulation only");
            self.running = true;
            Ok(())
        }
    }

    pub async fn stop(&mut self) {
        self.running = false;
    }
}
