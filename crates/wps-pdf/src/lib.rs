pub mod annotations;
pub mod converter;
pub mod editor;
pub mod forms_security;
pub mod layout_reconstructor;
pub mod ocr;
pub mod organizer;
pub mod parser;
pub mod security;
pub mod spatial;

use std::collections::HashMap;
use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use wps_render::{DisplayList, DrawCommand};

pub use annotations::{Annotation, AnnotationKind, ShapeType, StampType};
pub use converter::{PdfConverter, PdfToExcelMode};
pub use editor::{PdfEditSession, reflow_page_to_stream};
pub use forms_security::{
    AcroFieldType, AcroFormAndSignatureManager, AcroFormField, DigitalSignatureRecord,
    EncryptedPdfPackage, EncryptionHeader, PdfAes256Security, PdfAnnotationRecord,
    PdfRasterImage, PdfSecurityDocument, RedactionAuditReport, RedactionEngine,
    RedactionPattern, RedactionProposal, SignatureVerificationStatus, SignatureVisualAppearance,
    UnlockedRole,
};
pub use layout_reconstructor::{
    EditableParagraphBlock, LayoutColumnRegion, LayoutElement, LayoutReconstructorConfig,
    PageScene, PdfRect, PositionedGlyphRun, ReconstructedPageLayout, ReconstructedTableCell,
    ReconstructedTableGrid, ReconstructedTextLine, ReconstructedWord, RgbaColor, SemanticRole,
    SpatialLayoutReconstructor, TextAlignment, VectorFilledRect, VectorStrokeSegment,
};
pub use ocr::{OcrEngine, OcrMode, OcrOptions, OcrPageResult, OcrWord};
pub use organizer::{BatesPosition, CompressionLevel, PageOrganizer, SplitMode};
pub use parser::{PdfError, PdfMetadata, PdfObject, PdfPage, PdfParser, PdfTextElement, PdfVectorPath, PathOp};
pub use security::{DigitalSignature, FormField, FormFieldKind, PdfPermissions, PdfSecurity};
pub use spatial::{BlockKind, ReconstructedBlock, ReconstructedCell, ReconstructedTable, SpatialPageLayout, SpatialReconstructor};

#[derive(Debug, Clone)]
pub struct PdfDocument {
    pub title: String,
    pub pages: Vec<PdfPage>,
    pub annotations: HashMap<usize, Vec<Annotation>>,
    pub metadata: PdfMetadata,
    pub permissions: PdfPermissions,
    pub signatures: Vec<DigitalSignature>,
}

impl PdfDocument {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            pages: Vec::new(),
            annotations: HashMap::new(),
            metadata: PdfMetadata::default(),
            permissions: PdfPermissions::default(),
            signatures: Vec::new(),
        }
    }

    pub fn load_from_bytes(data: &[u8]) -> Result<Self, PdfError> {
        Self::load_with_password(data, "")
    }

    pub fn load_with_password(data: &[u8], password: &str) -> Result<Self, PdfError> {
        let decrypted_bytes = if data.windows(8).any(|w| w == b"/Encrypt") {
            if password.is_empty() {
                return Err(PdfError::Unsupported("Password required to decrypt PDF".to_string()));
            }
            PdfSecurity::decrypt_aes256(data, password)?
        } else {
            data.to_vec()
        };

        let mut parser = PdfParser::new(&decrypted_bytes);
        let pages = parser.parse()?;

        Ok(Self {
            title: "Document.pdf".to_string(),
            pages,
            annotations: HashMap::new(),
            metadata: PdfMetadata::default(),
            permissions: PdfPermissions::default(),
            signatures: Vec::new(),
        })
    }

    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }

    pub fn add_annotation(&mut self, page_idx: usize, annot: Annotation) {
        self.annotations.entry(page_idx).or_default().push(annot);
    }

    pub fn render_page_to_display_list(&self, page_idx: usize, dl: &mut DisplayList) {
        if let Some(page) = self.pages.get(page_idx) {
            // 1. Page background (Paper canvas)
            dl.push(DrawCommand::FillRect {
                rect: Rect::new(0.0, 0.0, page.width, page.height),
                color: Color::WHITE,
            });

            // 2. Vector paths
            for path in &page.vector_paths {
                if let Some(fill) = path.fill_color {
                    dl.push(DrawCommand::FillRect {
                        rect: path.bounds,
                        color: fill,
                    });
                }
                if let Some(stroke) = path.stroke_color {
                    dl.push(DrawCommand::StrokeRect {
                        rect: path.bounds,
                        color: stroke,
                        width: path.line_width.max(1.0),
                    });
                }
            }

            // 3. Text elements
            for elem in &page.text_elements {
                dl.push(DrawCommand::DrawTextRun {
                    text: elem.text.clone(),
                    font_family: elem.font_name.clone(),
                    font_size_px: elem.font_size,
                    bold: false,
                    italic: false,
                    color: elem.color,
                    origin: Point::new(elem.bounds.x, elem.baseline),
                });
            }

            // 4. Annotations layer
            if let Some(annots) = self.annotations.get(&page_idx) {
                for annot in annots {
                    annot.render_to_display_list(dl);
                }
            }
        }
    }

    pub fn to_writer_document(&self) -> wps_writer::ast::Document {
        PdfConverter::to_writer_document(&self.title, &self.pages)
    }

    pub fn to_sheets_matrix(&self) -> wps_sheets::SparseMatrix {
        PdfConverter::to_sheets_matrix(&self.pages)
    }

    pub fn to_sheets_workbook(&self, mode: PdfToExcelMode) -> wps_sheets::Workbook {
        PdfConverter::to_sheets_workbook(&self.title, &self.pages, mode)
    }

    pub fn to_slides_deck(&self) -> wps_slides::scene_graph::PresentationDeck {
        PdfConverter::to_slides_deck(&self.title, &self.pages)
    }

    /// Encrypts the PDF document using AES-256 standard encryption
    pub fn encrypt(&self, user_password: &str, owner_password: &str) -> Vec<u8> {
        let raw_bytes = self.export_to_bytes();
        PdfSecurity::encrypt_aes256(&raw_bytes, user_password, owner_password, &self.permissions)
    }

    /// Splits document into multiple chunk PDFs
    pub fn split(&self, mode: SplitMode) -> Vec<(String, PdfDocument)> {
        let chunks = PageOrganizer::split_pdf(&self.pages, mode);
        chunks.into_iter().map(|(name, pages)| {
            let mut doc = PdfDocument::new(&name);
            doc.pages = pages;
            (name, doc)
        }).collect()
    }

    /// Compresses all page streams in the document
    pub fn compress(&mut self, level: CompressionLevel) -> usize {
        PageOrganizer::compress_pages(&mut self.pages, level)
    }

    /// Irreversibly redacts an area on a specific page
    pub fn redact_region(&mut self, page_idx: usize, rect: Rect, label: Option<&str>) {
        if let Some(page) = self.pages.get_mut(page_idx) {
            PdfSecurity::redact_region(page, rect, label);
        }
    }

    /// Serializes the PDF document into a standard ISO 32000 PDF file byte stream
    pub fn export_to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n");

        let mut offsets: Vec<usize> = Vec::new();

        // 1 0 obj: Catalog
        offsets.push(out.len());
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // 2 0 obj: Pages
        offsets.push(out.len());
        let mut kids = String::new();
        for i in 0..self.pages.len() {
            kids.push_str(&format!("{} 0 R ", 3 + i * 2));
        }
        out.extend_from_slice(format!(
            "2 0 obj\n<< /Type /Pages /Kids [ {}] /Count {} >>\nendobj\n",
            kids, self.pages.len()
        ).as_bytes());

        // Pages and Content streams
        for (i, page) in self.pages.iter().enumerate() {
            let page_obj_id = 3 + i * 2;
            let stream_obj_id = page_obj_id + 1;

            // Page Object
            offsets.push(out.len());
            out.extend_from_slice(format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Rotate {} /Contents {} 0 R >>\nendobj\n",
                page_obj_id, page.width, page.height, page.rotation, stream_obj_id
            ).as_bytes());

            // Content Stream Object
            offsets.push(out.len());
            let stream_data = if page.raw_stream.is_empty() {
                let mut s = Vec::new();
                for path in &page.vector_paths {
                    if let Some(fill) = path.fill_color {
                        let r = fill.r as f64 / 255.0;
                        let g = fill.g as f64 / 255.0;
                        let b = fill.b as f64 / 255.0;
                        let pdf_y = page.height - path.bounds.y - path.bounds.height;
                        s.extend_from_slice(format!("{:.3} {:.3} {:.3} rg\n{:.2} {:.2} {:.2} {:.2} re\nf\n", r, g, b, path.bounds.x, pdf_y, path.bounds.width, path.bounds.height).as_bytes());
                    }
                    if let Some(stroke) = path.stroke_color {
                        let r = stroke.r as f64 / 255.0;
                        let g = stroke.g as f64 / 255.0;
                        let b = stroke.b as f64 / 255.0;
                        let pdf_y = page.height - path.bounds.y - path.bounds.height;
                        s.extend_from_slice(format!("{:.3} {:.3} {:.3} RG\n{:.2} w\n{:.2} {:.2} {:.2} {:.2} re\nS\n", r, g, b, path.line_width.max(1.0), path.bounds.x, pdf_y, path.bounds.width, path.bounds.height).as_bytes());
                    }
                }
                for elem in &page.text_elements {
                    let pdf_y = page.height - elem.bounds.y - elem.bounds.height;
                    let safe_text = elem.text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
                    let r = elem.color.r as f64 / 255.0;
                    let g = elem.color.g as f64 / 255.0;
                    let b = elem.color.b as f64 / 255.0;
                    s.extend_from_slice(format!("BT\n/F1 {:.2} Tf\n{:.3} {:.3} {:.3} rg\n1 0 0 1 {:.2} {:.2} Tm\n({}) Tj\nET\n", elem.font_size, r, g, b, elem.bounds.x, pdf_y, safe_text).as_bytes());
                }
                s
            } else {
                page.raw_stream.clone()
            };

            let stream_len = stream_data.len();
            out.extend_from_slice(format!(
                "{} 0 obj\n<< /Length {} >>\nstream\n",
                stream_obj_id, stream_len
            ).as_bytes());
            out.extend_from_slice(&stream_data);
            out.extend_from_slice(b"\nendstream\nendobj\n");
        }

        // Xref Table
        let xref_offset = out.len();
        let total_objects = 1 + offsets.len();
        out.extend_from_slice(format!("xref\n0 {}\n", total_objects).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for off in offsets {
            out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }

        // Trailer
        out.extend_from_slice(format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            total_objects, xref_offset
        ).as_bytes());

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_content_stream_parser() {
        let stream = b"
            BT
            /F1 16 Tf
            0.2 0.4 0.8 rg
            1 0 0 1 72 700 Tm
            (Quarterly Financial Performance) Tj
            ET
            BT
            /F1 12 Tf
            0 0 0 rg
            1 0 0 1 72 650 Tm
            (Total revenue increased by 24% year-over-year.) Tj
            ET
        ";

        let parser = PdfParser::new(b"");
        let (text_elements, _) = parser.interpret_content_stream(stream, 595.0, 842.0);

        assert_eq!(text_elements.len(), 2);
        assert_eq!(text_elements[0].text, "Quarterly Financial Performance");
        assert_eq!(text_elements[0].font_size, 16.0);
        assert_eq!(text_elements[1].text, "Total revenue increased by 24% year-over-year.");
    }

    #[test]
    fn test_spatial_reconstruction_xy_cut() {
        let page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: vec![
                PdfTextElement {
                    text: "Executive Summary".to_string(),
                    bounds: Rect::new(50.0, 50.0, 200.0, 24.0),
                    font_name: "Helvetica-Bold".to_string(),
                    font_size: 24.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 68.0,
                },
                PdfTextElement {
                    text: "This document outlines strategic goals for OpenWPS.".to_string(),
                    bounds: Rect::new(50.0, 90.0, 350.0, 12.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 12.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 100.0,
                },
            ],
            vector_paths: Vec::new(),
            raw_stream: Vec::new(),
        };

        let layout = SpatialReconstructor::reconstruct_page(&page);
        assert_eq!(layout.blocks.len(), 2);
        assert_eq!(layout.blocks[0].kind, BlockKind::Title);
        assert_eq!(layout.blocks[0].plain_text(), "Executive Summary");
        assert_eq!(layout.blocks[1].kind, BlockKind::Paragraph);
    }

    #[test]
    fn test_table_detection_and_sheets_conversion() {
        let page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: vec![
                PdfTextElement {
                    text: "Q1".to_string(),
                    bounds: Rect::new(60.0, 110.0, 30.0, 12.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 10.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 118.0,
                },
                PdfTextElement {
                    text: "1500".to_string(),
                    bounds: Rect::new(160.0, 110.0, 40.0, 12.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 10.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 118.0,
                },
            ],
            vector_paths: vec![
                PdfVectorPath {
                    operations: vec![
                        PathOp::Rectangle(Rect::new(50.0, 100.0, 100.0, 50.0)),
                        PathOp::Rectangle(Rect::new(150.0, 100.0, 100.0, 50.0)),
                    ],
                    bounds: Rect::new(50.0, 100.0, 200.0, 50.0),
                    stroke_color: Some(Color::rgb(0, 0, 0)),
                    fill_color: None,
                    line_width: 1.0,
                },
            ],
            raw_stream: Vec::new(),
        };

        let matrix = PdfConverter::to_sheets_matrix(&[page]);
        // Cell (0, 0) should be Q1, Cell (0, 1) should be numeric 1500
        let c00 = matrix.get_cell(wps_sheets::CellCoordinate::new(0, 0));
        assert!(c00.is_some());
        assert_eq!(c00.unwrap().value, wps_sheets::CellValue::Text("Q1".to_string()));

        let c01 = matrix.get_cell(wps_sheets::CellCoordinate::new(0, 1));
        assert!(c01.is_some());
        assert_eq!(c01.unwrap().value, wps_sheets::CellValue::Number(1500.0));
    }

    #[test]
    fn test_inline_editor_word_reflow() {
        let mut page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: vec![
                PdfTextElement {
                    text: "Initial text block".to_string(),
                    bounds: Rect::new(50.0, 100.0, 120.0, 14.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 14.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 112.0,
                },
            ],
            vector_paths: Vec::new(),
            raw_stream: Vec::new(),
        };

        let mut session = PdfEditSession::new(&page, Point::new(60.0, 105.0)).expect("Target block found");
        session.insert_text(" modified with extra content that exceeds width");
        session.commit(&mut page);

        assert!(!page.text_elements.is_empty());
        assert!(page.raw_stream.len() > 0);
    }

    #[test]
    fn test_annotations_and_display_list() {
        let mut doc = PdfDocument::new("Test Annotations");
        let page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: Vec::new(),
            vector_paths: Vec::new(),
            raw_stream: Vec::new(),
        };
        doc.add_page(page);

        // Add 4 distinct annotations
        doc.add_annotation(0, Annotation::highlight("a1", 0, Rect::new(10.0, 20.0, 100.0, 15.0)));
        doc.add_annotation(0, Annotation::sticky_note("a2", 0, Point::new(30.0, 50.0), "Auditor", "Check this!"));
        doc.add_annotation(0, Annotation::stamp("a3", 0, Rect::new(400.0, 50.0, 120.0, 50.0), StampType::Approved, "Lead Architect", "2026-10-27"));
        doc.add_annotation(0, Annotation::measurement("a4", 0, Point::new(10.0, 200.0), Point::new(110.0, 200.0), 0.352778, "mm"));

        let mut dl = DisplayList::new();
        doc.render_page_to_display_list(0, &mut dl);

        assert!(dl.commands.len() >= 5); // Background + 4 annotations commands
    }

    #[test]
    fn test_page_organizer_operations() {
        let mut pages = vec![
            PdfPage { page_index: 0, width: 595.0, height: 842.0, rotation: 0, crop_box: Rect::new(0.0, 0.0, 595.0, 842.0), text_elements: Vec::new(), vector_paths: Vec::new(), raw_stream: Vec::new() },
            PdfPage { page_index: 1, width: 595.0, height: 842.0, rotation: 0, crop_box: Rect::new(0.0, 0.0, 595.0, 842.0), text_elements: Vec::new(), vector_paths: Vec::new(), raw_stream: Vec::new() },
            PdfPage { page_index: 2, width: 595.0, height: 842.0, rotation: 0, crop_box: Rect::new(0.0, 0.0, 595.0, 842.0), text_elements: Vec::new(), vector_paths: Vec::new(), raw_stream: Vec::new() },
        ];

        // 1. Reorder
        PageOrganizer::reorder_pages(&mut pages, &[2, 0, 1]);
        assert_eq!(pages.len(), 3);

        // 2. Rotate
        PageOrganizer::rotate_page(&mut pages[0], 90);
        assert_eq!(pages[0].rotation, 90);
        assert_eq!(pages[0].width, 842.0);
        assert_eq!(pages[0].height, 595.0);

        // 3. Bates numbering
        PageOrganizer::apply_bates_numbering(&mut pages, "CONFIDENTIAL-", 100, 6, BatesPosition::BottomRight);
        assert_eq!(pages[0].text_elements[0].text, "CONFIDENTIAL-000100");
    }

    #[test]
    fn test_permanent_redaction() {
        let mut page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: vec![
                PdfTextElement {
                    text: "CONFIDENTIAL SECRET 9876".to_string(),
                    bounds: Rect::new(50.0, 50.0, 150.0, 12.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 12.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 60.0,
                },
                PdfTextElement {
                    text: "Public information".to_string(),
                    bounds: Rect::new(50.0, 150.0, 150.0, 12.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 12.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 160.0,
                },
            ],
            vector_paths: Vec::new(),
            raw_stream: b"BT (CONFIDENTIAL SECRET 9876) Tj ET".to_vec(),
        };

        PdfSecurity::redact_region(&mut page, Rect::new(45.0, 45.0, 160.0, 20.0), Some("[REDACTED]"));

        // Sensitive element should be gone
        assert_eq!(page.text_elements.len(), 2); // 1 Public info + 1 [REDACTED] label
        assert_eq!(page.text_elements[0].text, "Public information");
        assert_eq!(page.text_elements[1].text, "[REDACTED]");

        // Raw stream must not contain "CONFIDENTIAL"
        assert!(!page.raw_stream.windows(12).any(|w| w == b"CONFIDENTIAL"));
    }

    #[test]
    fn test_full_pdf_export_and_roundtrip() {
        let mut doc = PdfDocument::new("Annual Report");
        let page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: vec![
                PdfTextElement {
                    text: "Annual Report 2026".to_string(),
                    bounds: Rect::new(50.0, 50.0, 180.0, 18.0),
                    font_name: "Helvetica-Bold".to_string(),
                    font_size: 18.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 65.0,
                },
            ],
            vector_paths: Vec::new(),
            raw_stream: b"BT /F1 18 Tf 1 0 0 1 50 774 Tm (Annual Report 2026) Tj ET\n".to_vec(),
        };
        doc.add_page(page);

        let pdf_bytes = doc.export_to_bytes();
        assert!(pdf_bytes.starts_with(b"%PDF-1.7"));
        assert!(pdf_bytes.ends_with(b"%%EOF\n"));

        // Parse back
        let loaded = PdfDocument::load_from_bytes(&pdf_bytes).expect("PDF loaded successfully");
        assert_eq!(loaded.pages.len(), 1);
        assert_eq!(loaded.pages[0].text_elements[0].text, "Annual Report 2026");
    }
}
