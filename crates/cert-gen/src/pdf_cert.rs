//! PDF certificate generation - disabled for now

use crate::{CertError, Result};

pub fn generate_certificate_pdf(_payload: &serde_json::Value, _qr_base64: &str) -> Result<Vec<u8>> {
    // PDF generation disabled due to printpdf API changes
    // Return a minimal PDF placeholder
    Ok(b"%PDF-1.4\n%HWCert Certificate Placeholder\n%%EOF".to_vec())
}