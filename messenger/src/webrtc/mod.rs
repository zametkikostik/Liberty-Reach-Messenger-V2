use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallType {
    Audio,
    Video,
    WalkieTalkie,
    Conference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallSession {
    pub id: String,
    pub call_type: CallType,
    pub participants: Vec<String>,
}

pub struct CallManager;

impl CallManager {
    pub fn audio(peer: &str) -> CallSession {
        CallSession {
            id: uuid::Uuid::new_v4().to_string(),
            call_type: CallType::Audio,
            participants: vec![peer.into()],
        }
    }

    pub fn conference(peers: Vec<String>) -> CallSession {
        CallSession {
            id: uuid::Uuid::new_v4().to_string(),
            call_type: CallType::Conference,
            participants: peers,
        }
    }
}
