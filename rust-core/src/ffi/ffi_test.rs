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

/// App-restart scenario: B encrypts its session state to disk (autosave),
/// the app is killed, a NEW session handle is created for the same identity,
/// the state is loaded back — and the conversation continues without a new
/// QR exchange. Also covers: wrong password, corrupted file, foreign identity.
#[test]
fn ffi_session_persistence_roundtrip() {
    unsafe {
        let a_key = c(&tmp("ps-a"));
        let b_key = c(&tmp("ps-b"));
        let peer_a = take_string(ffi::liberty_identity_create(a_key.as_ptr()));
        let peer_b = take_string(ffi::liberty_identity_create(b_key.as_ptr()));
        let pa = c(&peer_a);
        let pb = c(&peer_b);

        // ---- App run 1 (B): bind session to identity, enable autosave ----
        let hb1 = ffi::liberty_session_create_with_identity(b_key.as_ptr());
        assert_ne!(hb1, 0);
        let state_path = c(&tmp("ps-state"));
        let pw = c("correct horse battery staple");
        assert!(ffi::liberty_session_set_autosave(
            hb1,
            state_path.as_ptr(),
            pw.as_ptr()
        ));

        // A initiates toward B's bundle and sends message 1
        let ha = ffi::liberty_session_create_with_identity(a_key.as_ptr());
        assert_ne!(ha, 0);
        let bundle_b = take_string(ffi::liberty_session_prekey_bundle(hb1));
        assert!(bundle_b.contains("signed_prekey"));
        let bundle_c = c(&bundle_b);
        let eph_a = take_string(ffi::liberty_session_start_initiator(
            ha,
            pb.as_ptr(),
            bundle_c.as_ptr(),
        ));
        let m1 = encrypt_via_ffi(ha, &peer_b, "до рестарта", &eph_a);
        assert_eq!(decrypt_via_ffi(hb1, &peer_a, &m1), "до рестарта");
        // (B bootstraps as responder + ratchet advances on both sides)
        let m2 = encrypt_via_ffi(hb1, &peer_a, "ответ B", "");
        let p2 = take_string_or_empty(ffi::liberty_session_decrypt(
            ha,
            pb.as_ptr(),
            c(&m2).as_ptr(),
        ));
        assert_eq!(p2, "ответ B");

        // ---- App restart (B): fresh handle, session state is GONE ----
        let hb2 = ffi::liberty_session_create_with_identity(b_key.as_ptr());
        assert_ne!(hb2, 0);
        assert!(
            !ffi::liberty_session_has(hb2, pa.as_ptr()),
            "fresh handle must not have sessions"
        );

        // Wrong password must fail without restoring anything
        let bad_pw = c("wrong password");
        assert!(!ffi::liberty_session_load(
            hb2,
            state_path.as_ptr(),
            bad_pw.as_ptr()
        ));
        assert!(!ffi::liberty_session_has(hb2, pa.as_ptr()));

        // Correct password restores the ratchet sessions + prekey bundle
        assert!(ffi::liberty_session_load(
            hb2,
            state_path.as_ptr(),
            pw.as_ptr()
        ));
        assert!(ffi::liberty_session_has(hb2, pa.as_ptr()));

        // ---- Conversation continues across the restart ----
        let m3 = encrypt_via_ffi(ha, &peer_b, "после рестарта", &eph_a);
        let m3_c = c(&m3);
        let p3 = take_string_or_empty(ffi::liberty_session_decrypt(
            hb2,
            pa.as_ptr(),
            m3_c.as_ptr(),
        ));
        assert_eq!(
            p3, "после рестарта",
            "B must decrypt A's post-restart message"
        );

        let m4 = encrypt_via_ffi(hb2, &peer_a, "ответ после рестарта", "");
        let m4_c = c(&m4);
        let p4 = take_string_or_empty(ffi::liberty_session_decrypt(ha, pb.as_ptr(), m4_c.as_ptr()));
        assert_eq!(
            p4, "ответ после рестарта",
            "A must decrypt B's post-restart reply"
        );

        // ---- File at rest is really encrypted: no plaintext key material ----
        let raw = std::fs::read(tmp("ps-state")).expect("state file exists");
        assert!(raw.starts_with(b"LRS1"), "magic header");
        assert!(
            raw.len() > 64,
            "state file looks non-trivial: {}",
            raw.len()
        );
        let as_text = String::from_utf8_lossy(&raw);
        assert!(
            !as_text.contains("root_key"),
            "JSON state must not be readable in plaintext"
        );
        assert!(!as_text.contains("send_chain_key"));
        assert!(
            !as_text.contains(peer_a.as_str()),
            "peer id must not leak in plaintext"
        );

        // ---- Corrupted file must fail closed ----
        let mut corrupt = raw.clone();
        let mid = corrupt.len() / 2;
        corrupt[mid] ^= 0xFF;
        let corrupt_path = c(&tmp("ps-corrupt"));
        std::fs::write(corrupt_path.to_string_lossy().into_owned(), &corrupt).unwrap();
        let hb3 = ffi::liberty_session_create_with_identity(b_key.as_ptr());
        assert!(!ffi::liberty_session_load(
            hb3,
            corrupt_path.as_ptr(),
            pw.as_ptr()
        ));

        // ---- State bound to a different identity must be rejected ----
        let x_key = c(&tmp("ps-x"));
        take_string(ffi::liberty_identity_create(x_key.as_ptr()));
        let hx = ffi::liberty_session_create_with_identity(x_key.as_ptr());
        assert_ne!(hx, 0);
        assert!(
            !ffi::liberty_session_load(hx, state_path.as_ptr(), pw.as_ptr()),
            "state for B's identity must not load into X's session"
        );

        // ---- Explicit wipe: file gone, load fails ----
        let copy_path = c(&tmp("ps-copy"));
        std::fs::copy(tmp("ps-state"), copy_path.to_string_lossy().into_owned()).unwrap();
        assert!(ffi::liberty_session_wipe_state(copy_path.as_ptr()));
        assert!(!ffi::liberty_session_load(
            hb3,
            copy_path.as_ptr(),
            pw.as_ptr()
        ));
        assert!(std::fs::read(copy_path.to_string_lossy().into_owned()).is_err());

        // ---- clear autosave: further mutations must not touch the file ----
        let hb4 = ffi::liberty_session_create_with_identity(b_key.as_ptr());
        assert_ne!(hb4, 0);
        assert!(ffi::liberty_session_load(
            hb4,
            state_path.as_ptr(),
            pw.as_ptr()
        ));
        let path2 = c(&tmp("ps-state2"));
        assert!(ffi::liberty_session_set_autosave(
            hb4,
            path2.as_ptr(),
            pw.as_ptr()
        ));
        let snap1 = std::fs::read(tmp("ps-state2")).expect("autosave wrote the file");
        assert!(ffi::liberty_session_clear_autosave(hb4));
        let _ = encrypt_via_ffi(hb4, &peer_a, "после clear", "");
        let snap2 = std::fs::read(tmp("ps-state2")).expect("file still there");
        assert_eq!(
            snap1, snap2,
            "after clear_autosave the state file must not be rewritten"
        );
    }
}

/// Like `decrypt_via_ffi` but tolerant of empty results (assertion-free
/// wrapper for negative cases), returning "" on failure.
unsafe fn take_string_or_empty(p: *mut c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    let s = CStr::from_ptr(p).to_string_lossy().into_owned();
    ffi::liberty_free_string(p);
    s
}
