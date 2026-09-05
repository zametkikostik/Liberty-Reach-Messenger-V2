use liberty_core::crypto::{SessionKeys, RatchetSession, IdentityKeyPair, X3DH, SessionManager};

#[test]
fn ratchet_encrypt_produces_output() {
    let keys = SessionKeys {
        send_key: [1u8; 32],
        recv_key: [2u8; 32],
        chain_key: [3u8; 32],
    };
    let mut alice = RatchetSession::new(keys, true);
    let (header, ct) = alice.encrypt(b"hello liberty", b"aad").unwrap();
    assert!(!ct.is_empty());
    assert_eq!(header.n, 0);
}

#[test]
fn x3dh_bundle_and_initiate() {
    let alice = IdentityKeyPair::generate();
    let bob = IdentityKeyPair::generate();
    let (bob_bundle, _bob_spk) = X3DH::create_bundle(&bob);
    let (alice_keys, alice_eph, _eph) = X3DH::initiate(&alice, &bob_bundle).unwrap();
    assert_ne!(alice_keys.send_key, [0u8; 32]);
    assert_ne!(alice_eph, [0u8; 32]);
}

#[test]
fn session_manager_initiator() {
    let alice = IdentityKeyPair::generate();
    let bob = IdentityKeyPair::generate();
    let mut alice_sm = SessionManager::new(alice);
    let bob_sm = SessionManager::new(bob);
    let bob_bundle = bob_sm.our_prekey_bundle().unwrap().clone();
    let eph = alice_sm.start_as_initiator("bob", &bob_bundle).unwrap();
    assert!(alice_sm.has_session("bob"));
    assert_ne!(eph, [0u8; 32]);
}
