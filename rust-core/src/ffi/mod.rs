//! Public API for flutter_rust_bridge — simple types only
mod messaging_api;
pub use messaging_api::*;

use crate::crypto::{
    decrypt as aead_decrypt, encrypt as aead_encrypt, EncryptedMessage, IdentityKeyPair,
    PreKeyBundle, SessionManager, SessionManagerState,
};
use crate::identity::Identity;
use crate::security::{secure_wipe_file, Vault, VaultConfig, VaultMode};
use argon2::Argon2;
use once_cell::sync::Lazy;
use rand::RngCore;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static VAULTS: Lazy<Mutex<HashMap<u32, Vault>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SESSIONS: Lazy<Mutex<HashMap<u32, SessionManager>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static NEXT: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(1));
/// Auto-save target registered per session handle: (path, password).
/// After every state mutation the state is re-encrypted to disk so a kill
/// at any point loses at most the in-flight message.
static AUTOSAVE: Lazy<Mutex<HashMap<u32, (PathBuf, String)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
/// Extra paths wiped on panic (duress) unlock. `vault_create` registers the
/// identity file that lives next to the vault DB (the app keeps
/// `identity.key` beside `vault.db` in the app documents directory) — the
/// old hardcoded `./data/identity.key` wipe never matched real paths.
static PANIC_WIPE_EXTRA: Lazy<Mutex<Vec<PathBuf>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Session-state file format: b"LRS1" || salt[16] || aead(suite||nonce||ct)
/// where the AEAD key is Argon2(password, salt). Domain-separated AAD binds
/// the blob to this format so it can't be replayed as another artifact.
const STATE_MAGIC: &[u8; 4] = b"LRS1";
const STATE_AAD: &[u8] = b"liberty-session-state-v1";

fn derive_file_key(password: &str, salt: &[u8]) -> Result<[u8; 32], ()> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| ())?;
    Ok(key)
}

fn session_save_impl(handle: u32, path: &Path, password: &str) -> bool {
    let state = {
        let map = SESSIONS.lock().unwrap();
        match map.get(&handle).and_then(|s| s.export_state()) {
            Some(s) => s,
            None => return false,
        }
    };
    let json = match serde_json::to_vec(&state) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let key = match derive_file_key(password, &salt) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let blob = match aead_encrypt(&key, &json, STATE_AAD) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let mut file = Vec::with_capacity(STATE_MAGIC.len() + salt.len() + blob.len());
    file.extend_from_slice(STATE_MAGIC);
    file.extend_from_slice(&salt);
    file.extend_from_slice(&blob);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    std::fs::write(path, file).is_ok()
}

/// Best-effort auto-save after a state mutation; never fails the caller.
fn autosave(handle: u32) {
    let target = AUTOSAVE.lock().unwrap().get(&handle).cloned();
    if let Some((path, password)) = target {
        let _ = session_save_impl(handle, &path, &password);
    }
}

fn alloc_id() -> u32 {
    let mut n = NEXT.lock().unwrap();
    let id = *n;
    *n += 1;
    id
}

pub fn vault_create(db_path: String, master: String, duress: String) -> u32 {
    // The app keeps identity.key beside vault.db; on panic (duress) unlock
    // the identity file must be destroyed too (see PANIC_WIPE_EXTRA).
    let identity_sibling = Path::new(&db_path).with_file_name("identity.key");
    let config = VaultConfig {
        db_path: PathBuf::from(db_path),
        master_password: master,
        duress_password: duress,
    };
    match Vault::new(config) {
        Ok(v) => {
            let id = alloc_id();
            PANIC_WIPE_EXTRA.lock().unwrap().push(identity_sibling);
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
    let mode = vault.unlock(&password);
    match mode {
        Ok(VaultMode::Real) => 1,
        Ok(VaultMode::Decoy) => 2,
        // Panic wipe: also destroy the encrypted session-state file and the
        // identity file so key material doesn't outlive the vault.
        Ok(VaultMode::Panic) => {
            let mut targets: Vec<PathBuf> = AUTOSAVE
                .lock()
                .unwrap()
                .values()
                .map(|(p, _)| p.clone())
                .collect();
            targets.extend(PANIC_WIPE_EXTRA.lock().unwrap().iter().cloned());
            for p in targets {
                let _ = secure_wipe_file(&p);
            }
            AUTOSAVE.lock().unwrap().clear();
            3
        }
        Err(_) => 0,
    }
}

pub fn vault_close(handle: u32) -> bool {
    VAULTS.lock().unwrap().remove(&handle).is_some()
}

pub fn vault_is_real(handle: u32) -> bool {
    VAULTS
        .lock()
        .unwrap()
        .get(&handle)
        .map(|v| v.is_real())
        .unwrap_or(false)
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
        Ok(eph) => {
            drop(map);
            autosave(handle);
            hex::encode(eph)
        }
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
        Ok(msg) => {
            let out = serde_json::to_string(&msg).unwrap_or_default();
            drop(map);
            if !out.is_empty() {
                autosave(handle);
            }
            out
        }
        Err(_) => String::new(),
    }
}

/// Create a session bound to a persisted identity file (load-or-generate).
/// The prekey bundle is then stable across app restarts and tied to
/// `identity.key`. Returns 0 on error.
pub fn session_create_with_identity(identity_path: String) -> u32 {
    let id = match crate::identity::Identity::load_or_generate(&PathBuf::from(identity_path)) {
        Ok(id) => id,
        Err(_) => return 0,
    };
    let sm = SessionManager::new(id.keypair().clone());
    let h = alloc_id();
    SESSIONS.lock().unwrap().insert(h, sm);
    h
}

/// Decrypt an incoming EncryptedMessage (JSON). Bootstraps a responder
/// session via X3DH when none exists for the peer yet (requires
/// `initiator_ephemeral` + `sender_identity_x25519` in the message).
/// Returns the plaintext, or an empty string on any failure.
pub fn session_decrypt(handle: u32, peer_id: String, encrypted_json: String) -> String {
    let mut map = SESSIONS.lock().unwrap();
    let sm = match map.get_mut(&handle) {
        Some(s) => s,
        None => return String::new(),
    };
    let msg: EncryptedMessage = match serde_json::from_str(&encrypted_json) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    match sm.decrypt_smart(&peer_id, &msg) {
        Ok(pt) => {
            let out = String::from_utf8(pt).unwrap_or_default();
            drop(map);
            if !out.is_empty() {
                autosave(handle);
            }
            out
        }
        Err(_) => String::new(),
    }
}

/// Whether an E2EE session already exists for the given peer.
pub fn session_has(handle: u32, peer_id: String) -> bool {
    SESSIONS
        .lock()
        .unwrap()
        .get(&handle)
        .map(|s| s.has_session(&peer_id))
        .unwrap_or(false)
}

/// Persist the session state (ratchet sessions + prekey material) to `path`,
/// encrypted with a key derived from `password` (Argon2 → AES-256-GCM).
/// The plaintext never touches disk. Returns false on any failure.
pub fn session_save(handle: u32, path: String, password: String) -> bool {
    session_save_impl(handle, &PathBuf::from(path), &password)
}

/// Load a session state previously written by [`session_save`] into an
/// existing session handle. The handle must be bound to the SAME identity
/// (see `session_create_with_identity`); ratchet sessions and the saved
/// prekey bundle are restored. Returns false on wrong password, corrupt
/// file, or identity mismatch.
pub fn session_load(handle: u32, path: String, password: String) -> bool {
    let file = match std::fs::read(&path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    if file.len() < STATE_MAGIC.len() + 16 || file[..4] != *STATE_MAGIC {
        return false;
    }
    let salt = &file[4..20];
    let blob = &file[20..];
    let key = match derive_file_key(&password, salt) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let json = match aead_decrypt(&key, blob, STATE_AAD) {
        Ok(j) => j,
        Err(_) => return false,
    };
    let state: SessionManagerState = match serde_json::from_slice(&json) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut map = SESSIONS.lock().unwrap();
    match map.get_mut(&handle) {
        Some(sm) => sm.import_state(&state).is_ok(),
        None => false,
    }
}

/// Register an auto-save target for a session handle and write the state
/// immediately. After every mutation (initiator start, encrypt, decrypt)
/// the state is re-encrypted to disk, so a kill at any point loses at most
/// one in-flight message. Also wiped automatically on panic (duress) unlock.
pub fn session_set_autosave(handle: u32, path: String, password: String) -> bool {
    if !session_save_impl(handle, &PathBuf::from(&path), &password) {
        return false;
    }
    AUTOSAVE
        .lock()
        .unwrap()
        .insert(handle, (PathBuf::from(path), password));
    true
}

/// Securely overwrite and remove a session-state file (e.g. when the UI
/// observes a panic/duress unlock before any autosave was registered).
pub fn session_wipe_state(path: String) -> bool {
    secure_wipe_file(&PathBuf::from(path)).is_ok()
}

/// Remove the auto-save registration for a handle (call on lock). The
/// already-saved file stays on disk, encrypted; this stops a later decoy
/// session from writing into the real state file.
pub fn session_clear_autosave(handle: u32) -> bool {
    AUTOSAVE.lock().unwrap().remove(&handle).is_some()
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
    let names: Vec<&str> = crate::crypto::available_suites()
        .iter()
        .map(|s| s.name())
        .collect();
    serde_json::to_string(&names).unwrap_or_default()
}

pub fn build_features() -> String {
    #[allow(unused_mut)] // mut is only used when libp2p-swarm / post-quantum features are on
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
