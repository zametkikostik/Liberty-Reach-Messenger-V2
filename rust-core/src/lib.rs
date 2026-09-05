//! Liberty Core — Sovereign P2P Messenger Engine

pub mod crypto;
pub mod identity;
pub mod network;
pub mod storage;
pub mod security;
pub mod messaging;
pub mod groups;
pub mod import;
pub mod ffi;
pub mod api;

pub use security::{Vault, VaultMode, VaultConfig};
pub use identity::Identity;
pub use crypto::{SessionKeys, RatchetSession, MessageHeader, SessionManager, EncryptedMessage};
pub use messaging::MessagingService;
pub use groups::{GroupManager, GroupSession, GroupMeta, Role};
pub use import::{TelegramImporter, ImportResult, WhatsAppImporter, WhatsAppImportResult};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
