//! End-to-end tests through the C ABI — the exact functions Dart calls.
//! Covers: version/features, vault cycle, and a full two-party encrypted
//! conversation (initiator QR → first message bootstrap → replies in both
//! directions → tamper rejection).

use liberty_core::ffi;
use std::ffi::{c_char, CStr, CString};

fn c(s: &str) -> CString {
    CString::new(s).expect("no interior NUL in test input")
}

unsafe fn take_string(p: *mut c_char) -> String {
    assert!(!p.is_null(), "FFI returned NULL");
    let s = CStr::from_ptr(p).to_string_lossy().into_owned();
    ffi::liberty_free_string(p);
    s
}

fn tmp(name: &str) -> String {
    std::env::temp_dir()
        .join(format!("liberty-ffi-{name}-{}", std::process::id()))
        .to_string_lossy()
        .into_owned()
}

unsafe fn make_vault(name: &str) -> u32 {
    let db = c(&tmp(name));
    let m = c("master");
    let d = c("duress");
    let h = ffi::liberty_vault_create(db.as_ptr(), m.as_ptr(), d.as_ptr());
    assert_ne!(h, 0, "vault_create failed");
    h
}

unsafe fn encrypt_via_ffi(handle: u32, peer: &str, text: &str, my_eph_hex: &str) -> String {
    let p = c(peer);
    let t = c(text);
    let e = c(my_eph_hex);
    let out = take_string(ffi::liberty_session_encrypt(
        handle,
        p.as_ptr(),
        t.as_ptr(),
        e.as_ptr(),
    ));
    assert!(!out.is_empty(), "encrypt failed for {peer}");
    out
}

unsafe fn decrypt_via_ffi(handle: u32, peer: &str, enc_json: &str) -> String {
    let p = c(peer);
    let j = c(enc_json);
    take_string(ffi::liberty_session_decrypt(handle, p.as_ptr(), j.as_ptr()))
}

fn tamper_first_ciphertext_byte(enc_json: &mut String) {
    let marker = "\"ciphertext\":[";
    let at = enc_json.find(marker).expect("ciphertext field") + marker.len();
    let ch = enc_json[at..].chars().next().expect("first byte char");
    let flipped = if ch == '0' { '1' } else { '0' };
    enc_json.replace_range(at..at + 1, &flipped.to_string());
}

#[test]
fn ffi_version_and_features() {
    unsafe {
        let v = take_string(ffi::liberty_core_version());
        assert!(!v.is_empty());
        let f = take_string(ffi::liberty_build_features());
        assert!(f.contains("e2ee"), "features: {f}");
        assert!(f.contains("vault"), "features: {f}");
        let suites = take_string(ffi::liberty_cipher_suites());
        assert!(!suites.is_empty());
    }
}

#[test]
fn ffi_vault_cycle() {
    unsafe {
        // master → Real (1)
        let h1 = make_vault("v1");
        let pw = c("master");
        assert_eq!(ffi::liberty_vault_unlock(h1, pw.as_ptr()), 1);
        assert!(ffi::liberty_vault_is_real(h1));
        assert!(ffi::liberty_vault_close(h1));

        // unknown password → Decoy (2)
        let h2 = make_vault("v2");
        let pw2 = c("not-the-password");
        assert_eq!(ffi::liberty_vault_unlock(h2, pw2.as_ptr()), 2);
        assert!(!ffi::liberty_vault_is_real(h2));
        assert!(ffi::liberty_vault_close(h2));

        // duress → Panic wipe (3)
        let h3 = make_vault("v3");
        let pw3 = c("duress");
        assert_eq!(ffi::liberty_vault_unlock(h3, pw3.as_ptr()), 3);
        assert!(ffi::liberty_vault_close(h3));
    }
}

#[test]
fn ffi_two_party_conversation() {
    unsafe {
        let a_key = c(&tmp("ida"));
        let b_key = c(&tmp("idb"));

        // Both devices: persisted identity + session bound to it
        let peer_a = take_string(ffi::liberty_identity_create(a_key.as_ptr()));
        let peer_b = take_string(ffi::liberty_identity_create(b_key.as_ptr()));
        assert!(peer_a.starts_with("liberty:"), "peer id: {peer_a}");
        assert_ne!(peer_a, peer_b);

        let ha = ffi::liberty_session_create_with_identity(a_key.as_ptr());
        let hb = ffi::liberty_session_create_with_identity(b_key.as_ptr());
        assert_ne!(ha, 0);
        assert_ne!(hb, 0);

        // Identity reload must be stable: second session on the same file
        // produces the same peer id (bundle may differ — fresh SPK per run).
        let peer_a2 = take_string(ffi::liberty_identity_create(a_key.as_ptr()));
        assert_eq!(peer_a, peer_a2);

        // B scans A's QR payload and initiates
        let bundle_a = take_string(ffi::liberty_session_prekey_bundle(ha));
        assert!(bundle_a.contains("signed_prekey"), "bundle: {bundle_a}");

        let peer_a_c = c(&peer_a);
        let bundle_a_c = c(&bundle_a);
        let eph_b = take_string(ffi::liberty_session_start_initiator(
            hb,
            peer_a_c.as_ptr(),
            bundle_a_c.as_ptr(),
        ));
        assert_eq!(eph_b.len(), 64, "eph hex must be 32 bytes");
        assert!(eph_b.chars().all(|ch| ch.is_ascii_hexdigit()));

        // B → A (message 1): A has no session yet → auto-bootstrap as responder
        let m1 = encrypt_via_ffi(hb, &peer_a, "привет от B", &eph_b);
        assert!(m1.contains("sender_identity_x25519"));
        assert!(m1.contains("initiator_ephemeral"));
        let p1 = decrypt_via_ffi(ha, &peer_b, &m1);
        assert_eq!(p1, "привет от B");

        // Session existence is now observable on both sides
        let pa = c(&peer_a);
        let pb = c(&peer_b);
        assert!(ffi::liberty_session_has(hb, pa.as_ptr()));
        assert!(ffi::liberty_session_has(ha, pb.as_ptr()));

        // A → B (message 2): A replies through the bootstrapped session
        let m2 = encrypt_via_ffi(ha, &peer_b, "ответ от A", "");
        let p2 = decrypt_via_ffi(hb, &peer_a, &m2);
        assert_eq!(p2, "ответ от A");

        // B → A (message 3): ratchet advances on both sides
        let m3 = encrypt_via_ffi(hb, &peer_a, "ещё раз от B", &eph_b);
        let p3 = decrypt_via_ffi(ha, &peer_b, &m3);
        assert_eq!(p3, "ещё раз от B");

        // A → B (message 4)
        let m4 = encrypt_via_ffi(ha, &peer_b, "финал от A", "");
        let p4 = decrypt_via_ffi(hb, &peer_a, &m4);
        assert_eq!(p4, "финал от A");

        // Ciphertext never leaks the plaintext
        assert!(!m1.contains("привет"));

        // Tampered ciphertext must fail (empty string)
        let mut bad = m1.clone();
        tamper_first_ciphertext_byte(&mut bad);
        assert_eq!(decrypt_via_ffi(ha, &peer_b, &bad), "");

        // A third device without a session fails cleanly
        let hx = ffi::liberty_session_create();
        assert_ne!(hx, 0);
        assert_eq!(decrypt_via_ffi(hx, "liberty:xyz", &m1), "");
    }
}
