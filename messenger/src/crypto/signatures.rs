use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

use super::{CryptoError, Result};

#[derive(ZeroizeOnDrop)]
pub struct SignKeypair {
    signing: SigningKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicSignKey(pub [u8; 32]);

impl SignKeypair {
    pub fn generate() -> Self {
        Self {
            signing: SigningKey::generate(&mut OsRng),
        }
    }

    pub fn public_key(&self) -> PublicSignKey {
        PublicSignKey(self.signing.verifying_key().to_bytes())
    }

    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing.sign(message).to_bytes().to_vec()
    }

    pub fn verify(public: &PublicSignKey, message: &[u8], signature: &[u8]) -> Result<()> {
        let vk = VerifyingKey::from_bytes(&public.0)
            .map_err(|e| CryptoError::Signature(e.to_string()))?;
        let sig =
            Signature::from_slice(signature).map_err(|e| CryptoError::Signature(e.to_string()))?;
        vk.verify(message, &sig)
            .map_err(|e| CryptoError::Signature(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify() {
        let kp = SignKeypair::generate();
        let msg = b"signed by liberty";
        let sig = kp.sign(msg);
        assert!(SignKeypair::verify(&kp.public_key(), msg, &sig).is_ok());
        assert!(SignKeypair::verify(&kp.public_key(), b"nope", &sig).is_err());
    }
}
