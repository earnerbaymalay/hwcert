//! Ed25519 key pair management

use crate::{CertError, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct KeyPair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

// Custom serialization using bytes
impl Serialize for KeyPair {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("KeyPair", 2)?;
        state.serialize_field("signing_key", &hex::encode(self.signing_key.to_bytes()))?;
        state.serialize_field("verifying_key", &hex::encode(self.verifying_key.as_bytes()))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for KeyPair {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct KeyPairData {
            signing_key: String,
            verifying_key: String,
        }
        let data = KeyPairData::deserialize(deserializer)?;
        let signing_key_bytes = hex::decode(&data.signing_key)
            .map_err(|e| serde::de::Error::custom(format!("Invalid signing key hex: {}", e)))?;
        let verifying_key_bytes = hex::decode(&data.verifying_key)
            .map_err(|e| serde::de::Error::custom(format!("Invalid verifying key hex: {}", e)))?;

        let signing_key = SigningKey::from_bytes(
            signing_key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| serde::de::Error::custom("Invalid signing key length"))?,
        );
        let verifying_key = VerifyingKey::from_bytes(
            verifying_key_bytes
                .as_slice()
                .try_into()
                .map_err(|_| serde::de::Error::custom("Invalid verifying key length"))?,
        )
        .map_err(|e| serde::de::Error::custom(e.to_string()))?;

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }
}

impl KeyPair {
    /// Generate a fresh random key pair
    pub fn generate() -> Self {
        let mut secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
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

    /// Load from raw 32-byte secret key (PKCS#8 DER support deferred)
    pub fn from_pkcs8(pkcs8: &[u8]) -> Result<Self> {
        let key_bytes: [u8; 32] = pkcs8
            .get(..32)
            .ok_or_else(|| CertError::KeyGen("Key material too short (need 32 bytes)".to_string()))?
            .try_into()
            .map_err(|_| CertError::KeyGen("Invalid key length".to_string()))?;
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Export raw 32-byte secret key
    pub fn to_pkcs8(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
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
    let pk: [u8; 32] = public_key
        .try_into()
        .map_err(|_| CertError::Verification("Invalid public key length".to_string()))?;
    let verifying_key =
        VerifyingKey::from_bytes(&pk).map_err(|e| CertError::Verification(e.to_string()))?;

    let sig_bytes: [u8; 64] = signature
        .try_into()
        .map_err(|_| CertError::Verification("Invalid signature length".to_string()))?;
    let signature = Signature::from_bytes(&sig_bytes);

    verifying_key
        .verify(data, &signature)
        .map(|_| true)
        .map_err(|e| CertError::Verification(e.to_string()))
}