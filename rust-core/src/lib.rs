//! Liberty Core — Sovereign P2P Messenger Engine

pub mod api;
pub mod crypto;
pub mod ffi;
pub mod groups;
pub mod identity;
pub mod import;
pub mod messaging;
pub mod network;
pub mod security;
pub mod storage;

pub use crypto::{EncryptedMessage, MessageHeader, RatchetSession, SessionKeys, SessionManager};
pub use groups::{GroupManager, GroupMeta, GroupSession, Role};
pub use identity::Identity;
pub use import::{ImportResult, TelegramImporter, WhatsAppImportResult, WhatsAppImporter};
pub use messaging::MessagingService;
pub use security::{Vault, VaultConfig, VaultMode};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
