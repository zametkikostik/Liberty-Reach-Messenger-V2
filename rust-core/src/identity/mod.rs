//! Identity management
use crate::crypto::IdentityKeyPair;
use std::path::Path;
use std::fs;
use zeroize::Zeroizing;
use thiserror::Error;
use sha2::{Sha256, Digest};

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid key material")]
    InvalidKey,
}

pub struct Identity {
    app_keypair: IdentityKeyPair,
    peer_id: String,
}

impl Identity {
    pub fn generate() -> Self {
        let app_keypair = IdentityKeyPair::generate();
        let peer_id = derive_peer_id(&app_keypair.public_key_bytes());
        Self { app_keypair, peer_id }
    }

    pub fn load_or_generate(path: &Path) -> Result<Self, IdentityError> {
        if path.exists() { Self::load(path) } else {
            let id = Self::generate();
            id.save(path)?;
            Ok(id)
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), IdentityError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let secret = self.app_keypair.secret_bytes();
        let dh = self.app_keypair.dh_secret_bytes();
        let mut blob = Vec::with_capacity(64);
        blob.extend_from_slice(&secret);
        blob.extend_from_slice(&dh);
        let z = Zeroizing::new(blob);
        fs::write(path, z.as_ref())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, IdentityError> {
        let data = fs::read(path)?;
        if data.len() < 32 { return Err(IdentityError::InvalidKey); }
        let mut ed = [0u8; 32];
        ed.copy_from_slice(&data[..32]);
        let mut x = [0u8; 32];
        if data.len() >= 64 {
            x.copy_from_slice(&data[32..64]);
        } else {
            x = ed;
        }
        let app_keypair = IdentityKeyPair::from_bytes(ed, x);
        let peer_id = derive_peer_id(&app_keypair.public_key_bytes());
        Ok(Self { app_keypair, peer_id })
    }

    pub fn peer_id(&self) -> &str { &self.peer_id }
    pub fn keypair(&self) -> &IdentityKeyPair { &self.app_keypair }
    pub fn public_key_bytes(&self) -> [u8; 32] { self.app_keypair.public_key_bytes() }
}

fn derive_peer_id(public: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"liberty-peer-v1");
    hasher.update(public);
    let hash = hasher.finalize();
    format!("liberty:{}", hex::encode(&hash[..16]))
}
