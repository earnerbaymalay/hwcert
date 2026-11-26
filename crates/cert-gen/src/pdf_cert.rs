//! PDF certificate generation

use crate::{CertError, Result};
use printpdf::{PdfDocument, PdfPage, PdfLayer, Font, Mm, Pt, Color, Rgb, Line, Point, Matrix};
use base64;

pub fn generate_certificate_pdf(payload: &serde_json::Value, qr_base64: &str) -> Result<Vec<u8>> {
    let (doc, page1, layer1) = PdfDocument::new("HWCert Certificate", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);

    // Load font
    let font = doc.add_builtin_font(Font::Helvetica).map_err(|e| CertError::PdfGen(e.to_string()))?;
    let font_bold = doc.add_builtin_font(Font::HelveticaBold).map_err(|e| CertError::PdfGen(e.to_string()))?;

    let mut y = Mm(270.0);

    // Title
    layer.use_text("HWCert - Hardware Attestation Certificate", Pt(24.0), Mm(20.0), y, &font_bold);
    y -= Mm(15.0);

    // Separator
    layer.set_outline_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.set_outline_thickness(Pt(0.5));
    let mut line = Line::new();
    line.move_to(Mm(20.0), y);
    line.line_to(Mm(190.0), y);
    layer.add_line(line);
    y -= Mm(10.0);

    // Certificate ID
    if let Some(id) = payload.get("id") {
        layer.use_text(&format!("Certificate ID: {}", id), Pt(10.0), Mm(20.0), y, &font);
        y -= Mm(8.0);
    }

    // Type
    if let Some(cert_type) = payload.get("type") {
        layer.use_text(&format!("Type: {}", cert_type), Pt(10.0), Mm(20.0), y, &font);
        y -= Mm(8.0);
    }

    // Subject
    if let Some(subject) = payload.get("subject") {
        layer.use_text(&format!("Subject: {}", subject), Pt(10.0), Mm(20.0), y, &font);
        y -= Mm(8.0);
    }

    // Issuer
    layer.use_text("Issuer: HWCert", Pt(10.0), Mm(20.0), y, &font);
    y -= Mm(8.0);

    // Issued at
    let now = chrono::Utc::now();
    layer.use_text(&format!("Issued: {}", now.format("%Y-%m-%d %H:%M:%S UTC")), Pt(10.0), Mm(20.0), y, &font);
    y -= Mm(8.0);

    // Payload data
    y -= Mm(5.0);
    layer.use_text("Payload:", Pt(12.0), Mm(20.0), y, &font_bold);
    y -= Mm(8.0);

    let payload_str = serde_json::to_string_pretty(payload).unwrap_or_default();
    for line in payload_str.lines().take(20) {
        layer.use_text(line, Pt(8.0), Mm(25.0), y, &font);
        y -= Mm(5.0);
    }

    // QR Code
    if let Ok(qr_bytes) = base64::decode(qr_base64) {
        // QR code would be embedded as image
        // For now, just add placeholder
        layer.use_text("[QR Code]", Pt(10.0), Mm(20.0), y, &font);
        y -= Mm(30.0);
    }

    // Footer
    y = Mm(20.0);
    layer.use_text("Verify at: hwcert.example.com/verify", Pt(8.0), Mm(20.0), y, &font);
    y -= Mm(5.0);
    layer.use_text("This certificate is digitally signed.", Pt(8.0), Mm(20.0), y, &font);

    // Save to bytes
    doc.save_to(&mut Vec::new())
        .map_err(|e| CertError::PdfGen(e.to_string()))
}