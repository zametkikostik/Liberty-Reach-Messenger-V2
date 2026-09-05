//! CLI demo for liberty-core
use liberty_core::crypto::{IdentityKeyPair, X3DH, SessionManager};
use liberty_core::VERSION;

fn main() {
    tracing_subscriber::fmt::init();
    println!("Liberty Core {VERSION}");
    let alice = IdentityKeyPair::generate();
    let bob = IdentityKeyPair::generate();
    let (bundle, _) = X3DH::create_bundle(&bob);
    let mut alice_sm = SessionManager::new(alice);
    let eph = alice_sm.start_as_initiator("bob", &bundle).expect("x3dh");
    let msg = alice_sm.encrypt_for("bob", b"hello from liberty", Some(eph)).expect("encrypt");
    println!("encrypted {} bytes, n={}", msg.ciphertext.len(), msg.header.n);
    println!("demo ok");
}
