//! Telegram export JSON parser (result.json)
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub id: i64,
    pub date: Option<String>,
    pub from: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub name: String,
    pub messages: Vec<TelegramMessage>,
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub chats: Vec<TelegramChat>,
    pub total_messages: usize,
}

pub struct TelegramImporter;

impl TelegramImporter {
    pub fn from_json(json: &str) -> Result<ImportResult, String> {
        let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let mut chats = Vec::new();
        let mut total = 0usize;
        if let Some(arr) = v.get("chats").and_then(|c| c.get("list")).and_then(|l| l.as_array()) {
            for chat in arr {
                let name = chat.get("name").and_then(|n| n.as_str()).unwrap_or("Chat").to_string();
                let mut messages = Vec::new();
                if let Some(msgs) = chat.get("messages").and_then(|m| m.as_array()) {
                    for m in msgs {
                        if m.get("type").and_then(|t| t.as_str()) != Some("message") { continue; }
                        let text = match m.get("text") {
                            Some(serde_json::Value::String(s)) => s.clone(),
                            Some(serde_json::Value::Array(parts)) => parts.iter().filter_map(|p| p.as_str().map(|s| s.to_string()).or_else(|| p.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))).collect::<Vec<_>>().join(""),
                            _ => String::new(),
                        };
                        if text.is_empty() { continue; }
                        messages.push(TelegramMessage {
                            id: m.get("id").and_then(|i| i.as_i64()).unwrap_or(0),
                            date: m.get("date").and_then(|d| d.as_str()).map(|s| s.to_string()),
                            from: m.get("from").and_then(|f| f.as_str()).map(|s| s.to_string()),
                            text,
                        });
                    }
                }
                total += messages.len();
                chats.push(TelegramChat { name, messages });
            }
        }
        Ok(ImportResult { chats, total_messages: total })
    }
}
