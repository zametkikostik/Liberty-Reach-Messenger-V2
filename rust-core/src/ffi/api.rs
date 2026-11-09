//! C ABI for mobile (Dart FFI) and other embedders.
//!
//! Conventions:
//! - Strings cross the boundary as NUL-terminated UTF-8
//!   (`*const c_char` in, `*mut c_char` out).
//! - Every returned `*mut c_char` is owned by the caller and must be
//!   released with `liberty_free_string`.
//! - Handles are `u32`; `0` means invalid handle / failed operation.
//! - Composite payloads (prekey bundles, encrypted messages, federation
//!   config) travel as JSON strings — the exact formats `api` produces.

use std::ffi::{c_char, CStr, CString};

use crate::api;

fn to_c(s: String) -> *mut c_char {
    // Interior NULs cannot occur in the JSON / UTF-8 we produce;
    // fall back to an empty string rather than panicking across the FFI.
    CString::new(s).unwrap_or_default().into_raw()
}

unsafe fn from_c(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    CStr::from_ptr(p).to_string_lossy().into_owned()
}

/// Free a string returned by any `liberty_*` function.
///
/// # Safety
///
/// `s` must be a pointer returned by one of the `liberty_*` functions of
/// this library (or null). Passing any other pointer is undefined behavior.
/// Each pointer must be freed exactly once.
#[no_mangle]
pub unsafe extern "C" fn liberty_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Library version string. Caller frees with `liberty_free_string`.
#[no_mangle]
pub extern "C" fn liberty_core_version() -> *mut c_char {
    to_c(api::core_version())
}

/// Compiled feature list (JSON array). Caller frees.
#[no_mangle]
pub extern "C" fn liberty_build_features() -> *mut c_char {
    to_c(api::build_features())
}

/// Available cipher suites (JSON array). Caller frees.
#[no_mangle]
pub extern "C" fn liberty_cipher_suites() -> *mut c_char {
    to_c(api::cipher_suites())
}

// ---- Vault ----

/// Create a hidden vault. Returns a handle, or 0 on failure.
///
/// # Safety
///
/// `db_path`, `master`, `duress` must point to valid NUL-terminated UTF-8
/// (or be null, which is treated as an empty string).
#[no_mangle]
pub unsafe extern "C" fn liberty_vault_create(
    db_path: *const c_char,
    master: *const c_char,
    duress: *const c_char,
) -> u32 {
    api::vault_create(from_c(db_path), from_c(master), from_c(duress))
}

/// Try to unlock the vault.
///
/// Returns 1 = Real, 2 = Decoy, 3 = Panic (wiped), 0 = error.
///
/// # Safety
///
/// `password` must point to valid NUL-terminated UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_vault_unlock(handle: u32, password: *const c_char) -> i32 {
    api::vault_unlock(handle, from_c(password))
}

/// Close and drop the vault handle. Returns false for an unknown handle.
#[no_mangle]
pub extern "C" fn liberty_vault_close(handle: u32) -> bool {
    api::vault_close(handle)
}

/// Whether the vault is currently unlocked in Real mode.
#[no_mangle]
pub extern "C" fn liberty_vault_is_real(handle: u32) -> bool {
    api::vault_is_real(handle)
}

// ---- Identity / sessions ----

/// Load-or-generate the identity file; returns its peer id. Caller frees.
///
/// # Safety
///
/// `path` must point to valid NUL-terminated UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_identity_create(path: *const c_char) -> *mut c_char {
    to_c(api::identity_create(from_c(path)))
}

/// Create a session with a fresh (in-memory) identity.
#[no_mangle]
pub extern "C" fn liberty_session_create() -> u32 {
    api::session_create()
}

/// Create a session bound to a persisted identity file, so the prekey
/// bundle is stable across app restarts. Returns 0 on failure.
///
/// # Safety
///
/// `identity_path` must point to valid NUL-terminated UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_session_create_with_identity(identity_path: *const c_char) -> u32 {
    api::session_create_with_identity(from_c(identity_path))
}

/// Our prekey bundle as JSON (share via QR). Caller frees. Empty on error.
#[no_mangle]
pub extern "C" fn liberty_session_prekey_bundle(handle: u32) -> *mut c_char {
    to_c(api::session_prekey_bundle(handle))
}

/// Start a session as initiator toward `peer_id`. Returns our ephemeral
/// public key as hex (attach to outgoing messages), or empty on error.
///
/// # Safety
///
/// `peer_id` and `their_bundle_json` must point to valid NUL-terminated
/// UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_session_start_initiator(
    handle: u32,
    peer_id: *const c_char,
    their_bundle_json: *const c_char,
) -> *mut c_char {
    to_c(api::session_start_initiator(
        handle,
        from_c(peer_id),
        from_c(their_bundle_json),
    ))
}

/// Encrypt `plaintext` for `peer_id`. `eph_hex` is OUR initiator ephemeral
/// (hex) kept from `session_start_initiator`; pass "" when responding.
/// Returns the EncryptedMessage JSON, or empty on error. Caller frees.
///
/// # Safety
///
/// `peer_id`, `plaintext`, `eph_hex` must point to valid NUL-terminated
/// UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_session_encrypt(
    handle: u32,
    peer_id: *const c_char,
    plaintext: *const c_char,
    eph_hex: *const c_char,
) -> *mut c_char {
    to_c(api::session_encrypt(
        handle,
        from_c(peer_id),
        from_c(plaintext),
        from_c(eph_hex),
    ))
}

/// Decrypt an incoming EncryptedMessage JSON; bootstraps a responder
/// session on the first message. Returns the plaintext, or empty on
/// failure. Caller frees.
///
/// # Safety
///
/// `peer_id` and `encrypted_json` must point to valid NUL-terminated
/// UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_session_decrypt(
    handle: u32,
    peer_id: *const c_char,
    encrypted_json: *const c_char,
) -> *mut c_char {
    to_c(api::session_decrypt(
        handle,
        from_c(peer_id),
        from_c(encrypted_json),
    ))
}

/// Whether an E2EE session exists for `peer_id`.
///
/// # Safety
///
/// `peer_id` must point to valid NUL-terminated UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_session_has(handle: u32, peer_id: *const c_char) -> bool {
    api::session_has(handle, from_c(peer_id))
}

// ---- Federation ----

/// Current federation config as JSON. Caller frees.
#[no_mangle]
pub extern "C" fn liberty_federation_get() -> *mut c_char {
    to_c(api::federation_get())
}

/// Import federation config from JSON.
///
/// # Safety
///
/// `json` must point to valid NUL-terminated UTF-8 (or be null).
#[no_mangle]
pub unsafe extern "C" fn liberty_federation_set_json(json: *const c_char) -> bool {
    api::federation_set_json(from_c(json))
}

/// Bootstrap multiaddrs as JSON array. Caller frees.
#[no_mangle]
pub extern "C" fn liberty_federation_bootstrap_addrs() -> *mut c_char {
    to_c(api::federation_bootstrap_addrs())
}

/// Whether mesh mode is enabled.
#[no_mangle]
pub extern "C" fn liberty_federation_mesh_enabled() -> bool {
    api::federation_mesh_enabled()
}
