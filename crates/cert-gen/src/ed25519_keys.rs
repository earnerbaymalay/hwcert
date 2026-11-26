//! Ed25519 key pair management

use crate::{CertError, KeyPair, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    #[serde(skip)]
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl KeyPair {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    pub fn from_pkcs8(pkcs8: &[u8]) -> Result<Self> {
        let signing_key =
            SigningKey::from_pkcs8(pkcs8).map_err(|e| CertError::KeyGen(e.to_string()))?;
        let verifying_key = signing_key.verifying_key();
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    pub fn to_pkcs8(&self) -> Vec<u8> {
        self.signing_key.to_pkcs8_der().unwrap().as_bytes().to_vec()
    }

    pub fn verifying_key_bytes(&self) -> String {
        hex::encode(self.verifying_key.as_bytes())
    }

    pub fn signing_key_bytes(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }

    pub fn verify(&self, data: &[u8], signature: &Signature) -> Result<()> {
        self.verifying_key
            .verify(data, signature)
            .map_err(|e| CertError::Verification(e.to_string()))
    }
}

pub fn sign_data(key_pair: &KeyPair, data: &[u8]) -> Result<Signature> {
    Ok(key_pair.sign(data))
}

pub fn verify_signature(public_key: &[u8], data: &[u8], signature: &[u8]) -> Result<bool> {
    let verifying_key = VerifyingKey::from_bytes(
        public_key
            .try_into()
            .map_err(|_| CertError::Verification("Invalid public key length".to_string()))?,
    )
    .map_err(|e| CertError::Verification(e.to_string()))?;

    let signature = Signature::from_bytes(
        signature
            .try_into()
            .map_err(|_| CertError::Verification("Invalid signature length".to_string()))?,
    );

    verifying_key
        .verify(data, &signature)
        .map(|_| true)
        .map_err(|e| CertError::Verification(e.to_string()))
}
