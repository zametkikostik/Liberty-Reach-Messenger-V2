//! Import from other messengers
mod telegram;
mod whatsapp;
pub use telegram::{ImportResult, TelegramChat, TelegramImporter, TelegramMessage};
pub use whatsapp::{WhatsAppChat, WhatsAppImportResult, WhatsAppImporter, WhatsAppMessage};
