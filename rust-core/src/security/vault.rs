//! Hidden Vault: Master / Duress / Decoy passwords
//! Master → Real DB; other → Decoy; Duress → Panic Wipe

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
    Argon2,
};
use sled::Db;
use std::path::PathBuf;
use zeroize::Zeroize;
use thiserror::Error;
use crate::security::wipe::secure_wipe_file;

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("db: {0}")]
    Db(#[from] sled::Error),
    #[error("crypto / password error")]
    Crypto,
    #[error("vault is locked")]
    Locked,
    #[error("already unlocked")]
    AlreadyUnlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultMode {
    Real,
    Decoy,
    Panic,
}

#[derive(Debug, Clone, Copy)]
pub enum DuressAction {
    WipeAndExit,
    WipeAndDecoy,
}

pub struct VaultConfig {
    pub db_path: PathBuf,
    pub master_password: String,
    pub duress_password: String,
}

pub struct Vault {
    config: VaultConfig,
    real_db: Option<Db>,
    decoy_db: Option<Db>,
    mode: Option<VaultMode>,
    master_hash: String,
    duress_hash: String,
    salt: String,
}

impl Vault {
    pub fn new(config: VaultConfig) -> Result<Self, VaultError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let master_hash = hash_password(&argon2, &config.master_password, &salt)?;
        let duress_hash = hash_password(&argon2, &config.duress_password, &salt)?;
        Ok(Self {
            config,
            real_db: None,
            decoy_db: None,
            mode: None,
            master_hash,
            duress_hash,
            salt: salt.to_string(),
        })
    }

    pub fn unlock(&mut self, password: &str) -> Result<VaultMode, VaultError> {
        if self.mode.is_some() {
            return Err(VaultError::AlreadyUnlocked);
        }
        let argon2 = Argon2::default();
        if verify_password(&argon2, password, &self.duress_hash)? {
            self.panic_wipe()?;
            return Ok(VaultMode::Panic);
        }
        if verify_password(&argon2, password, &self.master_hash)? {
            let real_path = self.config.db_path.with_extension("real");
            self.real_db = Some(sled::open(&real_path)?);
            self.mode = Some(VaultMode::Real);
            return Ok(VaultMode::Real);
        }
        let decoy_path = self.config.db_path.with_extension("decoy");
        self.decoy_db = Some(sled::open(&decoy_path)?);
        self.mode = Some(VaultMode::Decoy);
        Ok(VaultMode::Decoy)
    }

    pub fn mode(&self) -> Option<VaultMode> { self.mode }
    pub fn is_real(&self) -> bool { self.mode == Some(VaultMode::Real) }

    pub fn panic_wipe(&mut self) -> Result<(), VaultError> {
        tracing::warn!("PANIC WIPE initiated");
        self.real_db.take();
        self.decoy_db.take();
        let real_path = self.config.db_path.with_extension("real");
        let decoy_path = self.config.db_path.with_extension("decoy");
        let identity_path = PathBuf::from("./data/identity.key");
        let _ = secure_wipe_file(&real_path);
        let _ = secure_wipe_file(&decoy_path);
        let _ = secure_wipe_file(&identity_path);
        self.config.master_password.zeroize();
        self.config.duress_password.zeroize();
        self.mode = Some(VaultMode::Panic);
        Ok(())
    }

    pub fn db(&self) -> Result<&Db, VaultError> {
        match self.mode {
            Some(VaultMode::Real) => self.real_db.as_ref().ok_or(VaultError::Locked),
            Some(VaultMode::Decoy) => self.decoy_db.as_ref().ok_or(VaultError::Locked),
            _ => Err(VaultError::Locked),
        }
    }
}

fn hash_password(argon2: &Argon2, password: &str, salt: &SaltString) -> Result<String, VaultError> {
    argon2.hash_password(password.as_bytes(), salt).map(|h| h.to_string()).map_err(|_| VaultError::Crypto)
}

fn verify_password(argon2: &Argon2, password: &str, hash_str: &str) -> Result<bool, VaultError> {
    let parsed = PasswordHash::new(hash_str).map_err(|_| VaultError::Crypto)?;
    Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}
