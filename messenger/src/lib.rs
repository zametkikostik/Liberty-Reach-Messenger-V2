//! Liberty Messenger V2 — core library

pub mod crypto;
pub mod chat;
pub mod p2p;
pub mod web3;
pub mod ai;
pub mod telegram;
pub mod webrtc;

pub use crypto::{HybridCipher, HybridIdentity, SignKeypair};
pub use p2p::{LocalMesh, P2pNode, TransportMode, chat_topic};

#[cfg(feature = "stego")]
pub use crypto::Steganography;
