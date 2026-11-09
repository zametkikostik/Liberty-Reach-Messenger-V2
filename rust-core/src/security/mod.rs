//! Security layer: Hidden Vault, Duress, Panic Wipe, anti-forensics, keystore

mod detect;
mod keystore;
mod vault;
mod wipe;

pub use detect::ForensicDetector;
pub use keystore::{KeystoreError, SecureKeystore, SoftwareKeystore};
pub use vault::{DuressAction, Vault, VaultConfig, VaultMode};
pub use wipe::secure_wipe_file;
