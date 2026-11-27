//! QR code generation

use crate::{CertError, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use qrcodegen::{QrCode, QrCodeEcc};

/// Quiet-zone width in modules required around a QR symbol.
const QUIET_ZONE: i32 = 4;

/// Encode `data` as a QR symbol (medium error correction).
pub fn encode_qr(data: &str) -> Result<QrCode> {
    QrCode::encode_text(data, QrCodeEcc::Medium).map_err(|e| CertError::QrCode(e.to_string()))
}

/// QR code for `data` as a base64-encoded PNG (8 px per module).
pub fn generate_qr_code(data: &str) -> Result<String> {
    Ok(STANDARD.encode(generate_qr_code_png(data, 8)?))
}

/// QR code for `data` as raw PNG bytes, `scale` pixels per module, with quiet zone.
pub fn generate_qr_code_png(data: &str, scale: usize) -> Result<Vec<u8>> {
    let qr = encode_qr(data)?;
    let scale = scale.max(1);
    let modules = (qr.size() + 2 * QUIET_ZONE) as usize;
    let side = modules * scale;

    // 8-bit grayscale: 0 = dark module, 255 = light.
    let mut pixels = vec![255u8; side * side];
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if !qr.get_module(x, y) {
                continue;
            }
            let (px, py) = ((x + QUIET_ZONE) as usize * scale, (y + QUIET_ZONE) as usize * scale);
            for row in py..py + scale {
                pixels[row * side + px..row * side + px + scale].fill(0);
            }
        }
    }

    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, side as u32, side as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let png_err = |e: png::EncodingError| CertError::QrCode(e.to_string());
    let mut writer = encoder.write_header().map_err(png_err)?;
    writer.write_image_data(&pixels).map_err(png_err)?;
    writer.finish().map_err(png_err)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_has_signature_and_quiet_zone() {
        let png = generate_qr_code_png("deadbeef", 2).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let b64 = generate_qr_code("deadbeef").unwrap();
        assert!(STANDARD.decode(b64).unwrap().starts_with(b"\x89PNG"));
    }
}
