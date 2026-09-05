//! Wire protocol for Gossipsub topics
use serde::{Serialize, Deserialize};

pub mod topics {
    pub const CHAT: &str = "liberty/chat/1.0.0";
    pub const PRESENCE: &str = "liberty/presence/1.0.0";
    pub const PREKEY: &str = "liberty/prekey/1.0.0";
    pub const GROUP: &str = "liberty/group/1.0.0";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMessage {
    SessionInit {
        from_peer: String,
        to_peer: String,
        initiator_ephemeral: [u8; 32],
        initiator_identity_x25519: [u8; 32],
        encrypted_payload: Vec<u8>,
        header: crate::crypto::MessageHeader,
        timestamp: i64,
    },
    Chat {
        from_peer: String,
        to_peer: String,
        encrypted_payload: Vec<u8>,
        header: crate::crypto::MessageHeader,
        timestamp: i64,
    },
    PreKeyAnnounce {
        peer_id: String,
        bundle: crate::crypto::PreKeyBundle,
    },
    Presence {
        peer_id: String,
        status: PresenceStatus,
        timestamp: i64,
    },
    GroupChat {
        group_id: String,
        from_peer: String,
        n: u32,
        ciphertext: Vec<u8>,
        timestamp: i64,
    },
    SealedChat {
        sealed_envelope: crate::crypto::SealedEnvelope,
        timestamp: i64,
    },
    Typing {
        from_peer: String,
        to_peer: String,
        is_typing: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceStatus {
    Online,
    Away,
    Offline,
}
