//! X3DH — Extended Triple Diffie–Hellman
//! DH1 = DH(IKa, SPKb); DH2 = DH(EKa, IKb); DH3 = DH(EKa, SPKb)
//! Shared key = HKDF(DH1 || DH2 || DH3)

use crate::crypto::{IdentityKeyPair, EphemeralKeyPair, SessionKeys};
use sha2::Sha256;
use hkdf::Hkdf;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PreKeyBundle {
    pub identity_ed25519: [u8; 32],
    pub identity_x25519: [u8; 32],
    pub signed_prekey: [u8; 32],
    pub signature: Vec<u8>,
}

pub struct X3DH;

impl X3DH {
    pub fn create_bundle(identity: &IdentityKeyPair) -> (PreKeyBundle, EphemeralKeyPair) {
        let signed_prekey = EphemeralKeyPair::generate();
        let sig = identity.sign(&signed_prekey.public_bytes());
        let bundle = PreKeyBundle {
            identity_ed25519: identity.public_key_bytes(),
            identity_x25519: identity.dh_public_bytes(),
            signed_prekey: signed_prekey.public_bytes(),
            signature: sig.to_bytes().to_vec(),
        };
        (bundle, signed_prekey)
    }

    pub fn initiate(
        our_identity: &IdentityKeyPair,
        their_bundle: &PreKeyBundle,
    ) -> Result<(SessionKeys, [u8; 32], EphemeralKeyPair), &'static str> {
        let our_eph = EphemeralKeyPair::generate();
        let dh1 = our_identity.diffie_hellman(&their_bundle.signed_prekey);
        let dh2 = our_eph.diffie_hellman(&their_bundle.identity_x25519);
        let dh3 = our_eph.diffie_hellman(&their_bundle.signed_prekey);
        let keys = combine(&[dh1.as_slice(), dh2.as_slice(), dh3.as_slice()], true);
        Ok((keys, our_eph.public_bytes(), our_eph))
    }

    pub fn respond(
        our_identity: &IdentityKeyPair,
        our_signed_prekey: &EphemeralKeyPair,
        their_identity_x25519: &[u8; 32],
        their_ephemeral: &[u8; 32],
    ) -> Result<SessionKeys, &'static str> {
        let dh1 = our_signed_prekey.diffie_hellman(their_identity_x25519);
        let dh2 = our_identity.diffie_hellman(their_ephemeral);
        let dh3 = our_signed_prekey.diffie_hellman(their_ephemeral);
        Ok(combine(&[dh1.as_slice(), dh2.as_slice(), dh3.as_slice()], false))
    }
}

fn combine(parts: &[&[u8]], initiator: bool) -> SessionKeys {
    let mut ikm = Vec::with_capacity(96);
    for p in parts {
        ikm.extend_from_slice(p);
    }
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut okm = [0u8; 96];
    hk.expand(b"Liberty-X3DH-v2", &mut okm).expect("HKDF");
    let mut k1 = [0u8; 32];
    let mut k2 = [0u8; 32];
    let mut root = [0u8; 32];
    k1.copy_from_slice(&okm[0..32]);
    k2.copy_from_slice(&okm[32..64]);
    root.copy_from_slice(&okm[64..96]);
    if initiator {
        SessionKeys { send_key: k1, recv_key: k2, chain_key: root }
    } else {
        SessionKeys { send_key: k2, recv_key: k1, chain_key: root }
    }
}
