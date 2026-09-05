//! WhatsApp chat export (.txt) parser
use serde::{Serialize, Deserialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppMessage {
    pub date: String,
    pub sender: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppChat {
    pub name: String,
    pub messages: Vec<WhatsAppMessage>,
}

#[derive(Debug, Clone)]
pub struct WhatsAppImportResult {
    pub chat: WhatsAppChat,
    pub total_messages: usize,
}

pub struct WhatsAppImporter;

impl WhatsAppImporter {
    /// Parse WhatsApp exported chat text
    pub fn from_text(name: &str, text: &str) -> Result<WhatsAppImportResult, String> {
        let re = Regex::new(r"(?m)^(\d{1,4}[/.\-]\d{1,2}[/.\-]\d{1,4},?\s+\d{1,2}:\d{2}(?::\d{2})?(?:\s*[APMapm]{2})?)\s+-\s+([^:]+):\s+(.*)$").map_err(|e| e.to_string())?;
        let mut messages = Vec::new();
        for cap in re.captures_iter(text) {
            messages.push(WhatsAppMessage {
                date: cap[1].to_string(),
                sender: cap[2].trim().to_string(),
                text: cap[3].to_string(),
            });
        }
        let total = messages.len();
        Ok(WhatsAppImportResult {
            chat: WhatsAppChat { name: name.to_string(), messages },
            total_messages: total,
        })
    }
}
