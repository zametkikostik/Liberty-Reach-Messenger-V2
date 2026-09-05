//! Security layer: Hidden Vault, Duress, Panic Wipe, anti-forensics, keystore

mod vault;
mod wipe;
mod detect;
mod keystore;

pub use vault::{Vault, VaultConfig, VaultMode, DuressAction};
pub use wipe::secure_wipe_file;
pub use detect::ForensicDetector;
pub use keystore::{SecureKeystore, SoftwareKeystore, KeystoreError};
