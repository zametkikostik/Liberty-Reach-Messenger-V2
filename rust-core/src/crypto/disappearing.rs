//! Disappearing (ephemeral) messages
use serde::{Serialize, Deserialize};
use crate::crypto::EncryptedMessage;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DisappearMode {
    AfterSend,
    AfterRead,
    Never,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EphemeralMessage {
    pub encrypted: EncryptedMessage,
    pub mode: DisappearMode,
    pub ttl_secs: u32,
    pub timer_start: Option<i64>,
    pub read: bool,
}

impl EphemeralMessage {
    pub fn new(encrypted: EncryptedMessage, mode: DisappearMode, ttl_secs: u32) -> Self {
        let timer_start = if mode == DisappearMode::AfterSend && ttl_secs > 0 {
            Some(encrypted.timestamp)
        } else {
            None
        };
        Self { encrypted, mode, ttl_secs, timer_start, read: false }
    }

    pub fn mark_read(&mut self, now: i64) {
        if !self.read {
            self.read = true;
            if self.mode == DisappearMode::AfterRead && self.ttl_secs > 0 {
                self.timer_start = Some(now);
            }
        }
    }

    pub fn is_expired(&self, now: i64) -> bool {
        if self.mode == DisappearMode::Never || self.ttl_secs == 0 {
            return false;
        }
        match self.timer_start {
            Some(start) => now >= start + self.ttl_secs as i64,
            None => false,
        }
    }

    pub fn seconds_remaining(&self, now: i64) -> Option<i64> {
        if self.mode == DisappearMode::Never || self.ttl_secs == 0 {
            return None;
        }
        let start = self.timer_start?;
        Some((start + self.ttl_secs as i64 - now).max(0))
    }
}

pub fn sweep_expired(messages: Vec<EphemeralMessage>, now: i64) -> Vec<EphemeralMessage> {
    messages.into_iter().filter(|m| !m.is_expired(now)).collect()
}

pub type DisappearingPolicy = DisappearMode;
