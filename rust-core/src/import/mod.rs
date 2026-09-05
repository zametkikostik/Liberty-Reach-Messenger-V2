//! Import from other messengers
mod telegram;
mod whatsapp;
pub use telegram::{TelegramImporter, TelegramChat, TelegramMessage, ImportResult};
pub use whatsapp::{WhatsAppImporter, WhatsAppChat, WhatsAppMessage, WhatsAppImportResult};
