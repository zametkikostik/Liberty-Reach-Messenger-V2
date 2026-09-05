//! Group chats foundation (sender-keys style; full MLS later)
use crate::crypto::{encrypt, decrypt, AeadError};
use serde::{Serialize, Deserialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::collections::HashMap;
use hkdf::Hkdf;
use sha2::Sha256;
use rand::RngCore;

pub type GroupId = [u8; 32];
pub type MemberId = String;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role { Admin, Member }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupMember {
    pub peer_id: MemberId,
    pub role: Role,
    pub identity_public: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupMeta {
    pub id: GroupId,
    pub name: String,
    pub created_at: i64,
    pub members: Vec<GroupMember>,
}

#[derive(Zeroize, ZeroizeOnDrop)]
struct SenderChain {
    chain_key: [u8; 32],
    n: u32,
}

pub struct GroupSession {
    pub meta: GroupMeta,
    send_chain: Option<SenderChain>,
    recv_chains: HashMap<MemberId, SenderChain>,
}

impl GroupSession {
    pub fn create(name: &str, admin_peer: &str, admin_identity: [u8; 32]) -> Self {
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);
        let meta = GroupMeta {
            id,
            name: name.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            members: vec![GroupMember {
                peer_id: admin_peer.to_string(),
                role: Role::Admin,
                identity_public: admin_identity,
            }],
        };
        let mut session = Self { meta, send_chain: None, recv_chains: HashMap::new() };
        session.init_send_chain();
        session
    }

    fn init_send_chain(&mut self) {
        let mut ck = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut ck);
        self.send_chain = Some(SenderChain { chain_key: ck, n: 0 });
    }

    fn kdf_ck(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
        let hk = Hkdf::<Sha256>::new(None, chain_key);
        let mut okm = [0u8; 64];
        hk.expand(b"LibertyGroupCK", &mut okm).expect("HKDF");
        let mut mk = [0u8; 32];
        let mut next = [0u8; 32];
        mk.copy_from_slice(&okm[0..32]);
        next.copy_from_slice(&okm[32..64]);
        (mk, next)
    }

    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<(u32, Vec<u8>), AeadError> {
        let chain = self.send_chain.as_mut().ok_or(AeadError::Encrypt)?;
        let (mk, next) = Self::kdf_ck(&chain.chain_key);
        chain.chain_key = next;
        let n = chain.n;
        chain.n += 1;
        let ct = encrypt(&mk, plaintext, &self.meta.id)?;
        Ok((n, ct))
    }

    pub fn decrypt(&mut self, from: &str, n: u32, ciphertext: &[u8]) -> Result<Vec<u8>, AeadError> {
        let chain = self.recv_chains.get_mut(from).ok_or(AeadError::Decrypt)?;
        while chain.n < n {
            let (_, next) = Self::kdf_ck(&chain.chain_key);
            chain.chain_key = next;
            chain.n += 1;
        }
        let (mk, next) = Self::kdf_ck(&chain.chain_key);
        chain.chain_key = next;
        chain.n += 1;
        decrypt(&mk, ciphertext, &self.meta.id)
    }
}

pub struct GroupManager {
    groups: HashMap<String, GroupSession>,
}

impl GroupManager {
    pub fn new() -> Self { Self { groups: HashMap::new() } }
    pub fn create(&mut self, name: &str, admin: &str, id_pub: [u8; 32]) -> String {
        let g = GroupSession::create(name, admin, id_pub);
        let key = hex::encode(g.meta.id);
        self.groups.insert(key.clone(), g);
        key
    }
    pub fn get_mut(&mut self, id_hex: &str) -> Option<&mut GroupSession> {
        self.groups.get_mut(id_hex)
    }
}

impl Default for GroupManager {
    fn default() -> Self { Self::new() }
}
