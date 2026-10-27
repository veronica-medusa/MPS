use serde::{Deserialize, Serialize};
use wps_core::color::Color;
use wps_core::geometry::Rect;
use crate::parser::{PdfPage, PdfTextElement};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub bounds: Rect,
    pub confidence: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrPageResult {
    pub page_index: usize,
    pub words: Vec<OcrWord>,
    pub mean_confidence: f32,
}

pub struct OcrEngine;

impl OcrEngine {
    /// Injects an invisible OCR text layer (PDF rendering mode 3 Tr) over scanned pages
    /// to make them fully selectable, searchable, and copy-pasteable
    pub fn inject_searchable_text_layer(page: &mut PdfPage, words: &[OcrWord]) {
        let mut invisible_stream = Vec::new();
        invisible_stream.extend_from_slice(b"\nq\n3 Tr\n"); // 3 Tr = invisible text mode

        for word in words {
            let elem = PdfTextElement {
                text: word.text.clone(),
                bounds: word.bounds,
                font_name: "Helvetica".to_string(),
                font_size: word.bounds.height,
                color: Color::rgba(0, 0, 0, 0), // Transparent
                baseline: word.bounds.y + word.bounds.height * 0.8,
            };

            page.text_elements.push(elem);

            let pdf_y = page.height - word.bounds.y - word.bounds.height;
            let safe_text = word.text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");

            let cmd = format!(
                "/F1 {:.2} Tf\n1 0 0 1 {:.2} {:.2} Tm\n({}) Tj\n",
                word.bounds.height, word.bounds.x, pdf_y, safe_text
            );
            invisible_stream.extend_from_slice(cmd.as_bytes());
        }

        invisible_stream.extend_from_slice(b"Q\n");
        page.raw_stream.extend_from_slice(&invisible_stream);
    }
}
