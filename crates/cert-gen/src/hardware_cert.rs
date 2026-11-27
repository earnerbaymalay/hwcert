//! Hardware certificate generation

use crate::{
    generate_certificate_pdf, generate_qr_code, sign_data, Certificate, CertificateType, KeyPair,
    Result,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCertificateData {
    pub hardware_inventory: Value,
    pub stress_results: Option<Value>,
    pub firmware_report: Option<Value>,
    pub benchmark_results: Option<Value>,
    pub generated_at: DateTime<Utc>,
}

pub async fn generate_hardware_certificate(
    hardware_data: &Value,
    key_pair: &KeyPair,
) -> Result<Certificate> {
    let cert_data = HardwareCertificateData {
        hardware_inventory: hardware_data.clone(),
        stress_results: None,
        firmware_report: None,
        benchmark_results: None,
        generated_at: Utc::now(),
    };

    let payload = serde_json::to_value(&cert_data)?;
    let payload_bytes = serde_json::to_vec(&payload)?;

    let signature = sign_data(key_pair, &payload_bytes)?;
    let signature_hex = hex::encode(signature.to_bytes());
    let qr_code = generate_qr_code(&signature_hex)?;
    let pdf_data = generate_certificate_pdf(&payload, &qr_code)?;

    Ok(Certificate {
        id: Uuid::new_v4(),
        cert_type: CertificateType::HardwareAttestation,
        subject: "Hardware Attestation".to_string(),
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