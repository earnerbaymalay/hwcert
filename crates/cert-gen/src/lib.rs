//! Certificate generation crate
//! Provides Ed25519 signing, QR codes, and PDF certificates

pub mod ed25519_keys;
pub mod hardware_cert;
pub mod pdf_cert;
pub mod qr_code;

// Re-export key types
pub use ed25519_keys::{sign_data, verify_signature, KeyPair};
pub use hardware_cert::generate_hardware_certificate;
pub use pdf_cert::generate_certificate_pdf;
pub use qr_code::generate_qr_code;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Certificate types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateType {
    HardwareAttestation,
    FirmwareVerification,
    StressTestResult,
    BenchmarkResult,
    Custom(String),
}

/// Certificate metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: Uuid,
    pub cert_type: CertificateType,
    pub subject: String,
    pub issuer: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub public_key: String,
    pub signature: String,
    pub payload: serde_json::Value,
    pub qr_code: Option<String>,
    pub pdf_data: Option<Vec<u8>>,
}

/// Errors for certificate generation
#[derive(Error, Debug)]
pub enum CertError {
    #[error("Key generation error: {0}")]
    KeyGen(String),
    #[error("Signing error: {0}")]
    Signing(String),
    #[error("Verification error: {0}")]
    Verification(String),
    #[error("QR code error: {0}")]
    QrCode(String),
    #[error("PDF generation error: {0}")]
    PdfGen(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CertError>;

/// Generate a hardware attestation certificate
pub async fn generate_hardware_attestation(
    hardware_data: &serde_json::Value,
    key_pair: &KeyPair,
) -> Result<Certificate> {
    generate_hardware_certificate(hardware_data, key_pair).await
}

/// Generate a firmware verification certificate
pub async fn generate_firmware_verification(
    firmware_report: &serde_json::Value,
    key_pair: &KeyPair,
) -> Result<Certificate> {
    let payload = serde_json::json!({
        "type": "firmware_verification",
        "data": firmware_report,
        "timestamp": chrono::Utc::now(),
    });

    let signature = sign_data(key_pair, &serde_json::to_vec(&payload)?)?;
    let signature_hex = hex::encode(signature.to_bytes());
    let qr_code = generate_qr_code(&signature_hex)?;

    Ok(Certificate {
        id: Uuid::new_v4(),
        cert_type: CertificateType::FirmwareVerification,
        subject: "Firmware Verification".to_string(),
        issuer: "HWCert".to_string(),
        issued_at: chrono::Utc::now(),
        expires_at: None,
        public_key: key_pair.verifying_key_bytes(),
        signature: signature_hex,
        payload,
        qr_code: Some(qr_code),
        pdf_data: None,
    })
}

/// Generate a stress test result certificate
pub async fn generate_stress_test_certificate(
    stress_result: &serde_json::Value,
    key_pair: &KeyPair,
) -> Result<Certificate> {
    let payload = serde_json::json!({
        "type": "stress_test_result",
        "data": stress_result,
        "timestamp": chrono::Utc::now(),
    });

    let signature = sign_data(key_pair, &serde_json::to_vec(&payload)?)?;
    let signature_hex = hex::encode(signature.to_bytes());
    let qr_code = generate_qr_code(&signature_hex)?;
    let pdf_data = generate_certificate_pdf(&payload, &signature_hex)?;

    Ok(Certificate {
        id: Uuid::new_v4(),
        cert_type: CertificateType::StressTestResult,
        subject: "Stress Test Result".to_string(),
        issuer: "HWCert".to_string(),
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::days(365)),
        public_key: key_pair.verifying_key_bytes(),
        signature: signature_hex,
        payload,
        qr_code: Some(qr_code),
        pdf_data: Some(pdf_data),
    })
}

/// Verify a certificate
pub fn verify_certificate(cert: &Certificate, public_key: &str) -> Result<bool> {
    let payload_bytes = serde_json::to_vec(&cert.payload)?;
    let signature_bytes = hex::decode(&cert.signature)?;
    let pubkey_bytes = hex::decode(public_key)?;

    verify_signature(&pubkey_bytes, &payload_bytes, &signature_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_certificate_generation() {
        let key_pair = KeyPair::generate();
        let payload = serde_json::json!({"test": "data"});
        let result = generate_hardware_attestation(&payload, &key_pair).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sign_verify_roundtrip() {
        let key_pair = KeyPair::generate();
        let data = b"hello hardware";
        let sig = key_pair.sign(data);
        assert!(key_pair.verify(data, &sig).is_ok());
    }
}