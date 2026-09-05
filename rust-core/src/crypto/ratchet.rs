//! Double Ratchet (Signal-style)
use crate::crypto::{SessionKeys, encrypt, decrypt, AeadError, EphemeralKeyPair};
use zeroize::{Zeroize, ZeroizeOnDrop};
use sha2::Sha256;
use hkdf::Hkdf;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

const MAX_SKIP: u32 = 100;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct RatchetSession {
    root_key: [u8; 32],
    send_chain_key: Option<[u8; 32]>,
    send_n: u32,
    recv_chain_key: Option<[u8; 32]>,
    recv_n: u32,
    prev_recv_n: u32,
    dh_send: Option<EphemeralKeyPair>,
    dh_recv_public: Option<[u8; 32]>,
    #[zeroize(skip)]
    skipped_keys: HashMap<(Vec<u8>, u32), [u8; 32]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageHeader {
    pub dh_public: [u8; 32],
    pub n: u32,
    pub pn: u32,
}

impl RatchetSession {
    pub fn new(initial: SessionKeys, is_initiator: bool) -> Self {
        let mut session = Self {
            root_key: initial.chain_key,
            send_chain_key: None,
            send_n: 0,
            recv_chain_key: None,
            recv_n: 0,
            prev_recv_n: 0,
            dh_send: None,
            dh_recv_public: None,
            skipped_keys: HashMap::new(),
        };
        if is_initiator {
            session.dh_send = Some(EphemeralKeyPair::generate());
            session.send_chain_key = Some(initial.send_key);
        } else {
            session.recv_chain_key = Some(initial.recv_key);
        }
        session
    }

    fn kdf_ck(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
        let hk = Hkdf::<Sha256>::new(None, chain_key);
        let mut okm = [0u8; 64];
        hk.expand(b"LibertyCK", &mut okm).expect("HKDF");
        let mut mk = [0u8; 32];
        let mut next = [0u8; 32];
        mk.copy_from_slice(&okm[0..32]);
        next.copy_from_slice(&okm[32..64]);
        (mk, next)
    }

    fn kdf_rk(root: &[u8; 32], dh_out: &[u8]) -> ([u8; 32], [u8; 32]) {
        let hk = Hkdf::<Sha256>::new(Some(root), dh_out);
        let mut okm = [0u8; 64];
        hk.expand(b"LibertyRK", &mut okm).expect("HKDF");
        let mut new_root = [0u8; 32];
        let mut chain = [0u8; 32];
        new_root.copy_from_slice(&okm[0..32]);
        chain.copy_from_slice(&okm[32..64]);
        (new_root, chain)
    }

    pub fn encrypt(&mut self, plaintext: &[u8], aad: &[u8]) -> Result<(MessageHeader, Vec<u8>), AeadError> {
        let chain = self.send_chain_key.as_mut().ok_or(AeadError::Encrypt)?;
        let (msg_key, next_ck) = Self::kdf_ck(chain);
        *chain = next_ck;
        let dh_public = self.dh_send.as_ref().map(|d| d.public_bytes()).unwrap_or([0u8; 32]);
        let header = MessageHeader { dh_public, n: self.send_n, pn: self.prev_recv_n };
        self.send_n += 1;
        let mut full_aad = aad.to_vec();
        full_aad.extend_from_slice(&header.dh_public);
        full_aad.extend_from_slice(&header.n.to_be_bytes());
        let ct = encrypt(&msg_key, plaintext, &full_aad)?;
        Ok((header, ct))
    }

    pub fn decrypt(&mut self, header: &MessageHeader, ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, AeadError> {
        let skip_key = (header.dh_public.to_vec(), header.n);
        if let Some(msg_key) = self.skipped_keys.remove(&skip_key) {
            let mut full_aad = aad.to_vec();
            full_aad.extend_from_slice(&header.dh_public);
            full_aad.extend_from_slice(&header.n.to_be_bytes());
            return decrypt(&msg_key, ciphertext, &full_aad);
        }
        if self.dh_recv_public.as_ref() != Some(&header.dh_public) {
            self.dh_ratchet(header)?;
        }
        self.skip_message_keys(header.n)?;
        let chain = self.recv_chain_key.as_mut().ok_or(AeadError::Decrypt)?;
        let (msg_key, next_ck) = Self::kdf_ck(chain);
        *chain = next_ck;
        self.recv_n += 1;
        let mut full_aad = aad.to_vec();
        full_aad.extend_from_slice(&header.dh_public);
        full_aad.extend_from_slice(&header.n.to_be_bytes());
        decrypt(&msg_key, ciphertext, &full_aad)
    }

    fn dh_ratchet(&mut self, header: &MessageHeader) -> Result<(), AeadError> {
        self.prev_recv_n = self.recv_n;
        self.recv_n = 0;
        self.send_n = 0;
        self.dh_recv_public = Some(header.dh_public);
        if let Some(ref dh_send) = self.dh_send {
            let shared = dh_send.diffie_hellman(&header.dh_public);
            let (new_root, recv_chain) = Self::kdf_rk(&self.root_key, shared.as_slice());
            self.root_key = new_root;
            self.recv_chain_key = Some(recv_chain);
        }
        let new_dh = EphemeralKeyPair::generate();
        if let Some(ref their_pub) = self.dh_recv_public {
            let shared = new_dh.diffie_hellman(their_pub);
            let (new_root, send_chain) = Self::kdf_rk(&self.root_key, shared.as_slice());
            self.root_key = new_root;
            self.send_chain_key = Some(send_chain);
        }
        self.dh_send = Some(new_dh);
        Ok(())
    }

    fn skip_message_keys(&mut self, until: u32) -> Result<(), AeadError> {
        if self.recv_n + MAX_SKIP < until {
            return Err(AeadError::Decrypt);
        }
        while self.recv_n < until {
            if let Some(ref mut chain) = self.recv_chain_key {
                let (mk, next) = Self::kdf_ck(chain);
                *chain = next;
                if let Some(dh_pub) = self.dh_recv_public {
                    self.skipped_keys.insert((dh_pub.to_vec(), self.recv_n), mk);
                }
                self.recv_n += 1;
            } else {
                break;
            }
        }
        Ok(())
    }
}
