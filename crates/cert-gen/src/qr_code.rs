//! QR code generation

use crate::{CertError, Result};
use base64;
use qrcodegen::{QrCode, QrCodeEcc};

pub fn generate_qr_code(data: &str) -> Result<String> {
    let code = QrCode::new(data.as_bytes(), QrCodeEcc::Medium)
        .map_err(|e| CertError::QrCode(e.to_string()))?;

    // Render as SVG
    let svg = render_qr_svg(&code);
    Ok(base64::encode(svg))
}

pub fn generate_qr_code_png(data: &str, scale: usize) -> Result<Vec<u8>> {
    let code = QrCode::new(data.as_bytes(), QrCodeEcc::Medium)
        .map_err(|e| CertError::QrCode(e.to_string()))?;

    let mut png = Vec::new();
    let mut encoder =
        png::Encoder::new((code.width() * scale) as u32, (code.width() * scale) as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder
        .write_header(&mut png)
        .map_err(|e| CertError::QrCode(e.to_string()))?;

    let mut image_data = Vec::new();
    for y in 0..code.width() {
        for _ in 0..scale {
            for x in 0..code.width() {
                let pixel = if code.get_module(x, y) { 0 } else { 255 };
                for _ in 0..scale {
                    image_data.push(pixel);
                }
            }
        }
    }

    writer
        .write_image_data(&image_data)
        .map_err(|e| CertError::QrCode(e.to_string()))?;
    writer
        .finish()
        .map_err(|e| CertError::QrCode(e.to_string()))?;

    Ok(png)
}

fn render_qr_svg(code: &QrCode) -> String {
    let mut svg = String::new();
    svg.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg" "#);
    svg.push_str(&format!(
        r#"width="{}mm" height="{}mm" "#,
        code.width(),
        code.width()
    ));
    svg.push_str(r#"viewBox="0 0 "#);
    svg.push_str(&format!("{} {}", code.width(), code.width()));
    svg.push_str(r#"">"#);
    svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);
    svg.push_str(r#"<g fill="black">"#);

    for y in 0..code.width() {
        for x in 0..code.width() {
            if code.get_module(x, y) {
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="1" height="1"/>"#,
                    x, y
                ));
            }
        }
    }

    svg.push_str(r#"</g></svg>"#);
    svg
}
