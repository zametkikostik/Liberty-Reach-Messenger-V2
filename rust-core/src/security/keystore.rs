//! Platform keystore interface + software fallback
use zeroize::Zeroizing;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("not available on this platform")]
    NotAvailable,
    #[error("key not found")]
    NotFound,
    #[error("operation failed: {0}")]
    Failed(String),
}

pub trait SecureKeystore: Send + Sync {
    fn store(&self, alias: &str, key: &[u8]) -> Result<(), KeystoreError>;
    fn load(&self, alias: &str) -> Result<Zeroizing<Vec<u8>>, KeystoreError>;
    fn delete(&self, alias: &str) -> Result<(), KeystoreError>;
    fn is_hardware_backed(&self) -> bool;
}

pub struct SoftwareKeystore {
    map: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

impl SoftwareKeystore {
    pub fn new() -> Self {
        Self { map: std::sync::Mutex::new(std::collections::HashMap::new()) }
    }
}

impl Default for SoftwareKeystore {
    fn default() -> Self { Self::new() }
}

impl SecureKeystore for SoftwareKeystore {
    fn store(&self, alias: &str, key: &[u8]) -> Result<(), KeystoreError> {
        self.map.lock().unwrap().insert(alias.to_string(), key.to_vec());
        Ok(())
    }
    fn load(&self, alias: &str) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        self.map.lock().unwrap().get(alias).cloned().map(Zeroizing::new).ok_or(KeystoreError::NotFound)
    }
    fn delete(&self, alias: &str) -> Result<(), KeystoreError> {
        self.map.lock().unwrap().remove(alias);
        Ok(())
    }
    fn is_hardware_backed(&self) -> bool { false }
}
