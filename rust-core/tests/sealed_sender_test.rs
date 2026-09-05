use liberty_core::crypto::{IdentityKeyPair, SealedSender};

#[test]
fn seal_and_open() {
    let alice = IdentityKeyPair::generate();
    let bob = IdentityKeyPair::generate();
    let (bob_secret, bob_keys) = SealedSender::create_keys(&bob);
    let envelope = SealedSender::seal(
        "alice-peer",
        &alice,
        &bob_keys.public,
        b"ciphertext-payload".to_vec(),
        None,
    )
    .unwrap();
    let (sender, payload) = SealedSender::open(&envelope, &bob_secret.secret).unwrap();
    assert_eq!(sender.peer_id, "alice-peer");
    assert_eq!(payload, b"ciphertext-payload");
}
