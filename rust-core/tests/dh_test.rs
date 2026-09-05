use liberty_core::crypto::{EphemeralKeyPair, DiffieHellman, IdentityKeyPair, X3DH};

#[test]
fn classic_diffie_hellman_matches() {
    let alice = EphemeralKeyPair::generate();
    let bob = EphemeralKeyPair::generate();
    let ss_a = alice.diffie_hellman(&bob.public_bytes());
    let ss_b = bob.diffie_hellman(&alice.public_bytes());
    assert!(DiffieHellman::secrets_match(&ss_a, &ss_b));
    assert_ne!(ss_a.as_slice(), &[0u8; 32]);
}

#[test]
fn x3dh_triple_dh_both_sides() {
    let alice = IdentityKeyPair::generate();
    let bob = IdentityKeyPair::generate();
    let (bundle, bob_spk) = X3DH::create_bundle(&bob);
    let (alice_keys, alice_eph, _) = X3DH::initiate(&alice, &bundle).unwrap();
    let bob_keys = X3DH::respond(&bob, &bob_spk, &alice.dh_public_bytes(), &alice_eph).unwrap();
    assert_eq!(alice_keys.chain_key, bob_keys.chain_key);
    assert_eq!(alice_keys.send_key, bob_keys.recv_key);
    assert_eq!(alice_keys.recv_key, bob_keys.send_key);
}

#[test]
fn identity_has_independent_x25519() {
    let id = IdentityKeyPair::generate();
    assert_ne!(id.public_key_bytes(), id.dh_public_bytes());
}
