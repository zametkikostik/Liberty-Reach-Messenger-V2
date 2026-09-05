//! Transport trait + LocalTransport for tests
use async_trait::async_trait;
use crate::network::{NetworkMessage, NetworkError};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub peer_id: String,
    pub multiaddrs: Vec<String>,
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&self, peer: &PeerInfo) -> Result<(), NetworkError>;
    async fn send(&self, to: &str, msg: &NetworkMessage) -> Result<(), NetworkError>;
    async fn receive(&self) -> Result<Option<NetworkMessage>, NetworkError>;
    fn local_peer_id(&self) -> &str;
}

/// In-process transport for unit tests / simulation
pub struct LocalTransport {
    peer_id: String,
    peers: Arc<Mutex<HashMap<String, Arc<Mutex<VecDeque<NetworkMessage>>>>>>,
    inbox: Arc<Mutex<VecDeque<NetworkMessage>>>,
}

impl LocalTransport {
    pub fn new(peer_id: &str) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            peers: Arc::new(Mutex::new(HashMap::new())),
            inbox: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn register_peer(&self, peer_id: &str, inbox: Arc<Mutex<VecDeque<NetworkMessage>>>) {
        self.peers.lock().unwrap().insert(peer_id.to_string(), inbox);
    }

    pub fn inbox_handle(&self) -> Arc<Mutex<VecDeque<NetworkMessage>>> {
        self.inbox.clone()
    }
}

#[async_trait]
impl Transport for LocalTransport {
    async fn connect(&self, peer: &PeerInfo) -> Result<(), NetworkError> {
        if self.peers.lock().unwrap().contains_key(&peer.peer_id) {
            Ok(())
        } else {
            Err(NetworkError::Connection(format!("peer {} not registered", peer.peer_id)))
        }
    }

    async fn send(&self, to: &str, msg: &NetworkMessage) -> Result<(), NetworkError> {
        let peers = self.peers.lock().unwrap();
        let inbox = peers.get(to).ok_or(NetworkError::NotConnected)?;
        inbox.lock().unwrap().push_back(msg.clone());
        Ok(())
    }

    async fn receive(&self) -> Result<Option<NetworkMessage>, NetworkError> {
        Ok(self.inbox.lock().unwrap().pop_front())
    }

    fn local_peer_id(&self) -> &str { &self.peer_id }
}
