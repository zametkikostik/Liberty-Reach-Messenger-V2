//! MLS-lite group scaffolding (not full RFC 9420 yet)
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyPackage {
    pub peer_id: String,
    pub identity_public: [u8; 32],
    pub init_key: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Welcome {
    pub group_id: [u8; 32],
    pub epoch: u64,
    pub encrypted_group_secrets: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    pub epoch: u64,
    pub proposals: Vec<u8>,
    pub path_secret: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MlsMessage {
    pub group_id: [u8; 32],
    pub epoch: u64,
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct MlsGroup {
    pub group_id: [u8; 32],
    pub epoch: u64,
    pub members: Vec<String>,
}

impl MlsGroup {
    pub fn new(group_id: [u8; 32], creator: &str) -> Self {
        Self {
            group_id,
            epoch: 0,
            members: vec![creator.to_string()],
        }
    }

    pub fn add_member(&mut self, peer_id: &str) {
        if !self.members.contains(&peer_id.to_string()) {
            self.members.push(peer_id.to_string());
            self.epoch += 1;
        }
    }
}
