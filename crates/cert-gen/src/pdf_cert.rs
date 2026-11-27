//! PDF certificate generation (A4, built-in fonts, vector QR code)

use crate::qr_code::encode_qr;
use crate::{CertError, Result};
use printpdf::{BuiltinFont, Mm, PdfDocument, Point, Polygon, PolygonMode, WindingOrder};

const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 20.0;
/// Payload lines printed before truncating; the signed JSON stays the source of truth.
const MAX_PAYLOAD_LINES: usize = 45;

/// Render a one-page certificate: title, `signature_hex` as text and as a QR code,
/// and the start of the pretty-printed payload.
pub fn generate_certificate_pdf(payload: &serde_json::Value, signature_hex: &str) -> Result<Vec<u8>> {
    let pdf_err = |e: printpdf::Error| CertError::PdfGen(e.to_string());

    let (doc, page, layer) = PdfDocument::new("HWCert Certificate", Mm(PAGE_W), Mm(PAGE_H), "Layer 1");
    let layer = doc.get_page(page).get_layer(layer);
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(pdf_err)?;
    let mono = doc.add_builtin_font(BuiltinFont::Courier).map_err(pdf_err)?;

    layer.use_text("HWCert Certificate", 22.0, Mm(MARGIN), Mm(PAGE_H - 30.0), &bold);

    // Signature, wrapped to fit beside the QR code.
    layer.use_text("Ed25519 signature:", 10.0, Mm(MARGIN), Mm(PAGE_H - 42.0), &bold);
    let sig_chars: Vec<char> = signature_hex.chars().collect();
    for (i, chunk) in sig_chars.chunks(32).enumerate() {
        let line: String = chunk.iter().collect();
        layer.use_text(line, 9.0, Mm(MARGIN), Mm(PAGE_H - 48.0 - i as f32 * 4.5), &mono);
    }

    // QR code of the signature, top right, one filled square per dark module.
    let qr = encode_qr(signature_hex)?;
    let qr_side = 50.0;
    let module = qr_side / qr.size() as f32;
    let (qr_x, qr_top) = (PAGE_W - MARGIN - qr_side, PAGE_H - 22.0);
    let mut rings = Vec::new();
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                let (lx, ty) = (qr_x + x as f32 * module, qr_top - y as f32 * module);
                rings.push(vec![
                    (Point::new(Mm(lx), Mm(ty - module)), false),
                    (Point::new(Mm(lx + module), Mm(ty - module)), false),
                    (Point::new(Mm(lx + module), Mm(ty)), false),
                    (Point::new(Mm(lx), Mm(ty)), false),
                ]);
            }
        }
    }
    layer.add_polygon(Polygon { rings, mode: PolygonMode::Fill, winding_order: WindingOrder::NonZero });

    // Payload.
    let mut y = PAGE_H - 85.0;
    layer.use_text("Signed payload:", 10.0, Mm(MARGIN), Mm(y), &bold);
    let pretty = serde_json::to_string_pretty(payload)?;
    let lines: Vec<&str> = pretty.lines().collect();
    for line in lines.iter().take(MAX_PAYLOAD_LINES) {
        y -= 4.0;
        let clipped: String = line.chars().take(95).collect();
        layer.use_text(clipped, 7.0, Mm(MARGIN), Mm(y), &mono);
    }
    if lines.len() > MAX_PAYLOAD_LINES {
        y -= 5.0;
        let note = format!("... {} more lines; see the JSON certificate", lines.len() - MAX_PAYLOAD_LINES);
        layer.use_text(note, 7.0, Mm(MARGIN), Mm(y), &mono);
    }

    doc.save_to_bytes().map_err(pdf_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_pdf() {
        let pdf = generate_certificate_pdf(&serde_json::json!({"cpu": "test"}), "ab".repeat(64).as_str())
            .unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.len() > 1000);
    }
}
