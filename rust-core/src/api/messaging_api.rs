//! Higher-level messaging helpers for FRB
use crate::import::{TelegramImporter, WhatsAppImporter};

pub fn telegram_import_json(json: String) -> String {
    match TelegramImporter::from_json(&json) {
        Ok(r) => serde_json::json!({
            "ok": true,
            "chats": r.chats.len(),
            "messages": r.total_messages,
        }).to_string(),
        Err(e) => serde_json::json!({"ok": false, "error": e}).to_string(),
    }
}

pub fn whatsapp_import_text(name: String, text: String) -> String {
    match WhatsAppImporter::from_text(&name, &text) {
        Ok(r) => serde_json::json!({
            "ok": true,
            "name": r.chat.name,
            "messages": r.total_messages,
        }).to_string(),
        Err(e) => serde_json::json!({"ok": false, "error": e}).to_string(),
    }
}
