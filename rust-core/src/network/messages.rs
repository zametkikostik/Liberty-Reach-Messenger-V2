//! Network message envelope
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    Chat,
    Ack,
    KeyExchange,
    Presence,
    PanicSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub from: String,
    pub to: String,
    pub msg_type: MessageType,
    pub payload: Vec<u8>,
    pub timestamp: i64,
}
