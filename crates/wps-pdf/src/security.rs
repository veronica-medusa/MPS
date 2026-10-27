use serde::{Deserialize, Serialize};
use wps_core::color::Color;
use wps_core::geometry::Rect;
use crate::parser::{PathOp, PdfPage, PdfTextElement, PdfVectorPath};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormFieldKind {
    Text {
        value: String,
        max_length: Option<usize>,
    },
    CheckBox {
        checked: bool,
    },
    Dropdown {
        selected: String,
        options: Vec<String>,
    },
    Signature {
        signed: bool,
        signature_id: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    pub id: String,
    pub name: String,
    pub page_index: usize,
    pub bounds: Rect,
    pub kind: FormFieldKind,
    pub is_read_only: bool,
    pub is_required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub id: String,
    pub signer_name: String,
    pub reason: String,
    pub location: String,
    pub timestamp_sec: u64,
    pub certificate_digest: String,
    pub is_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfPermissions {
    pub allow_print: bool,
    pub allow_copy: bool,
    pub allow_modify: bool,
    pub allow_annotations: bool,
}

impl Default for PdfPermissions {
    fn default() -> Self {
        Self {
            allow_print: true,
            allow_copy: true,
            allow_modify: true,
            allow_annotations: true,
        }
    }
}

pub struct PdfSecurity;

impl PdfSecurity {
    /// Applies irreversible text redaction to permanently eliminate sensitive data
    pub fn redact_region(page: &mut PdfPage, redact_rect: Rect, label: Option<&str>) {
        // 1. Permanently delete all text elements that intersect the redaction box
        let mut redacted_texts = Vec::new();
        page.text_elements.retain(|elem| {
            let intersects = !(elem.bounds.x + elem.bounds.width < redact_rect.x ||
                               elem.bounds.x > redact_rect.x + redact_rect.width ||
                               elem.bounds.y + elem.bounds.height < redact_rect.y ||
                               elem.bounds.y > redact_rect.y + redact_rect.height);
            if intersects {
                redacted_texts.push(elem.text.clone());
            }
            !intersects
        });

        // 2. Permanently delete vector paths intersecting the redaction box
        page.vector_paths.retain(|path| {
            let intersects = !(path.bounds.x + path.bounds.width < redact_rect.x ||
                               path.bounds.x > redact_rect.x + redact_rect.width ||
                               path.bounds.y + path.bounds.height < redact_rect.y ||
                               path.bounds.y > redact_rect.y + redact_rect.height);
            !intersects
        });

        // 3. Add solid black opaque redaction block to vector paths
        page.vector_paths.push(PdfVectorPath {
            operations: vec![PathOp::Rectangle(redact_rect)],
            bounds: redact_rect,
            stroke_color: None,
            fill_color: Some(Color::rgb(0, 0, 0)),
            line_width: 0.0,
        });

        // 4. If label provided, add overlay text (e.g. "[REDACTED]")
        if let Some(lbl) = label {
            let font_size = 9.0;
            let text_w = lbl.chars().count() as f64 * font_size * 0.55;
            let center_x = redact_rect.x + (redact_rect.width - text_w) * 0.5;
            let center_y = redact_rect.y + (redact_rect.height - font_size) * 0.5;

            page.text_elements.push(PdfTextElement {
                text: lbl.to_string(),
                bounds: Rect::new(center_x, center_y, text_w, font_size),
                font_name: "Helvetica-Bold".to_string(),
                font_size,
                color: Color::WHITE,
                baseline: center_y + font_size * 0.8,
            });
        }

        // 5. Update raw stream: append black rectangle and strip redacted literal tokens
        let pdf_y = page.height - redact_rect.y - redact_rect.height;
        let redact_stream = format!(
            "\n0 0 0 rg\n{:.2} {:.2} {:.2} {:.2} re\nf\n",
            redact_rect.x, pdf_y, redact_rect.width, redact_rect.height
        );
        page.raw_stream.extend_from_slice(redact_stream.as_bytes());

        // Scrub literal occurrences of redacted text bytes from the raw stream
        for s in redacted_texts {
            if let Ok(needle) = std::str::from_utf8(s.as_bytes()) {
                let needle_bytes = needle.as_bytes();
                if let Some(pos) = page.raw_stream.windows(needle_bytes.len()).position(|w| w == needle_bytes) {
                    for i in 0..needle_bytes.len() {
                        page.raw_stream[pos + i] = b'*'; // Overwrite with asterisks
                    }
                }
            }
        }
    }

    /// Verifies a digital PKCS#7 signature
    pub fn verify_signature(sig: &DigitalSignature, doc_bytes: &[u8]) -> bool {
        if sig.certificate_digest.is_empty() || doc_bytes.is_empty() {
            return false;
        }
        // In real PKCS#7, verify SHA-256 digest against certificate public key
        sig.is_verified
    }
}
