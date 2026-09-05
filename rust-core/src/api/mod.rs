//! Public API for flutter_rust_bridge — simple types only
mod messaging_api;
pub use messaging_api::*;

use crate::security::{Vault, VaultConfig, VaultMode};
use crate::crypto::{IdentityKeyPair, SessionManager, PreKeyBundle};
use crate::identity::Identity;
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

static VAULTS: Lazy<Mutex<HashMap<u32, Vault>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SESSIONS: Lazy<Mutex<HashMap<u32, SessionManager>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(1));

fn alloc_id() -> u32 {
    let mut n = NEXT.lock().unwrap();
    let id = *n;
    *n += 1;
    id
}

pub fn vault_create(db_path: String, master: String, duress: String) -> u32 {
    let config = VaultConfig {
        db_path: PathBuf::from(db_path),
        master_password: master,
        duress_password: duress,
    };
    match Vault::new(config) {
        Ok(v) => {
            let id = alloc_id();
            VAULTS.lock().unwrap().insert(id, v);
            id
        }
        Err(_) => 0,
    }
}

/// 1=Real, 2=Decoy, 3=Panic, 0=error
pub fn vault_unlock(handle: u32, password: String) -> i32 {
    let mut map = VAULTS.lock().unwrap();
    let vault = match map.get_mut(&handle) {
        Some(v) => v,
        None => return 0,
    };
    match vault.unlock(&password) {
        Ok(VaultMode::Real) => 1,
        Ok(VaultMode::Decoy) => 2,
        Ok(VaultMode::Panic) => 3,
        Err(_) => 0,
    }
}

pub fn vault_close(handle: u32) -> bool {
    VAULTS.lock().unwrap().remove(&handle).is_some()
}

pub fn vault_is_real(handle: u32) -> bool {
    VAULTS.lock().unwrap().get(&handle).map(|v| v.is_real()).unwrap_or(false)
}

pub fn identity_create(path: String) -> String {
    match Identity::load_or_generate(&PathBuf::from(path)) {
        Ok(id) => id.peer_id().to_string(),
        Err(_) => String::new(),
    }
}

pub fn session_create() -> u32 {
    let kp = IdentityKeyPair::generate();
    let sm = SessionManager::new(kp);
    let id = alloc_id();
    SESSIONS.lock().unwrap().insert(id, sm);
    id
}

pub fn session_prekey_bundle(handle: u32) -> String {
    let map = SESSIONS.lock().unwrap();
    match map.get(&handle).and_then(|s| s.our_prekey_bundle()) {
        Some(b) => serde_json::to_string(b).unwrap_or_default(),
        None => String::new(),
    }
}

pub fn session_start_initiator(handle: u32, peer_id: String, their_bundle_json: String) -> String {
    let mut map = SESSIONS.lock().unwrap();
    let sm = match map.get_mut(&handle) {
        Some(s) => s,
        None => return String::new(),
    };
    let bundle: PreKeyBundle = match serde_json::from_str(&their_bundle_json) {
        Ok(b) => b,
        Err(_) => return String::new(),
    };
    match sm.start_as_initiator(&peer_id, &bundle) {
        Ok(eph) => hex::encode(eph),
        Err(_) => String::new(),
    }
}

pub fn session_encrypt(handle: u32, peer_id: String, plaintext: String, eph_hex: String) -> String {
    let mut map = SESSIONS.lock().unwrap();
    let sm = match map.get_mut(&handle) {
        Some(s) => s,
        None => return String::new(),
    };
    let eph = if eph_hex.len() == 64 {
        let bytes = hex::decode(&eph_hex).ok();
        bytes.and_then(|b| {
            if b.len() == 32 {
                let mut a = [0u8; 32];
                a.copy_from_slice(&b);
                Some(a)
            } else {
                None
            }
        })
    } else {
        None
    };
    match sm.encrypt_for(&peer_id, plaintext.as_bytes(), eph) {
        Ok(msg) => serde_json::to_string(&msg).unwrap_or_default(),
        Err(_) => String::new(),
    }
}

pub fn federation_get() -> String {
    crate::network::federation_export()
}

pub fn federation_set_json(json: String) -> bool {
    crate::network::federation_import(&json).is_ok()
}

pub fn federation_bootstrap_addrs() -> String {
    serde_json::to_string(&crate::network::bootstrap_addrs()).unwrap_or_else(|_| "[]".into())
}

pub fn federation_mesh_enabled() -> bool {
    crate::network::mesh_enabled()
}

pub fn cipher_suites() -> String {
    let names: Vec<&str> = crate::crypto::available_suites().iter().map(|s| s.name()).collect();
    serde_json::to_string(&names).unwrap_or_default()
}

pub fn build_features() -> String {
    let mut f = vec!["e2ee", "vault", "federation"];
    #[cfg(feature = "libp2p-swarm")]
    f.push("libp2p");
    #[cfg(feature = "post-quantum")]
    f.push("post-quantum");
    serde_json::to_string(&f).unwrap_or_default()
}

pub fn core_version() -> String {
    crate::VERSION.to_string()
}
