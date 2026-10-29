use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use wps_pdf::{
    Annotation, PageOrganizer, PathOp, PdfConverter, PdfDocument, PdfEditSession, PdfPage,
    PdfTextElement, PdfToExcelMode, PdfVectorPath, SpatialReconstructor, SplitMode, StampType,
};

#[test]
fn test_pdf_generation_and_stream_interpretation() {
    let mut doc = PdfDocument::new("Quarterly Executive Report");

    // Page 1: Heading and 2-column paragraphs
    let page1 = PdfPage {
        page_index: 0,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: vec![
            PdfTextElement {
                text: "OpenWPS Executive Report".to_string(),
                bounds: Rect::new(50.0, 70.0, 320.0, 24.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 24.0,
                color: Color::rgb(0, 0, 0),
                baseline: 90.0,
            },
            // Column 1 (Left: x in 50..250)
            PdfTextElement {
                text: "Column 1 Section Alpha details infrastructure and high concurrency benchmarks.".to_string(),
                bounds: Rect::new(50.0, 130.0, 200.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 142.0,
            },
            PdfTextElement {
                text: "Memory safety guarantees zero null pointer exceptions across all document ASTs.".to_string(),
                bounds: Rect::new(50.0, 160.0, 200.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 172.0,
            },
            // Column 2 (Right: x in 320..520)
            PdfTextElement {
                text: "Column 2 Section Beta outlines global deployment metrics and cloud synchronization.".to_string(),
                bounds: Rect::new(320.0, 130.0, 200.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 142.0,
            },
            PdfTextElement {
                text: "CRDT convergence maintains mathematical consistency across peer agent nodes.".to_string(),
                bounds: Rect::new(320.0, 160.0, 200.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 172.0,
            },
        ],
        vector_paths: Vec::new(),
        raw_stream: b"BT /F1 24 Tf 1 0 0 1 50 752 Tm (OpenWPS Executive Report) Tj ET\n".to_vec(),
    };
    doc.add_page(page1);

    // Page 2: Bordered 4x4 Table
    let mut page2_elements = Vec::new();
    let mut page2_paths = Vec::new();

    // 4x4 table borders: 5 horizontal lines and 5 vertical lines
    for r in 0..=4 {
        let y = 100.0 + r as f64 * 30.0;
        page2_paths.push(PdfVectorPath {
            operations: vec![
                PathOp::MoveTo(Point::new(50.0, y)),
                PathOp::LineTo(Point::new(450.0, y)),
            ],
            bounds: Rect::new(50.0, y, 400.0, 1.0),
            stroke_color: Some(Color::rgb(0, 0, 0)),
            fill_color: None,
            line_width: 1.0,
        });
    }
    for c in 0..=4 {
        let x = 50.0 + c as f64 * 100.0;
        page2_paths.push(PdfVectorPath {
            operations: vec![
                PathOp::MoveTo(Point::new(x, 100.0)),
                PathOp::LineTo(Point::new(x, 220.0)),
            ],
            bounds: Rect::new(x, 100.0, 1.0, 120.0),
            stroke_color: Some(Color::rgb(0, 0, 0)),
            fill_color: None,
            line_width: 1.0,
        });
    }

    // Add cell text for 4x4 table
    for r in 0..4 {
        for c in 0..4 {
            let cx = 55.0 + c as f64 * 100.0;
            let cy = 110.0 + r as f64 * 30.0;
            let val = format!("Cell_{}_{}", r + 1, c + 1);
            page2_elements.push(PdfTextElement {
                text: val,
                bounds: Rect::new(cx, cy, 80.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 10.0,
                color: Color::rgb(0, 0, 0),
                baseline: cy + 10.0,
            });
        }
    }

    doc.add_page(PdfPage {
        page_index: 1,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: page2_elements,
        vector_paths: page2_paths,
        raw_stream: Vec::new(),
    });

    // Page 3: Borderless aligned 4x3 financial table & vector graphics
    let mut page3_elements = Vec::new();
    let financial_data = [
        ("FiscalYear", "RevenueUSD", "GrowthRate"),
        ("2023", "45000000", "0.18"),
        ("2024", "62000000", "0.37"),
        ("2025", "88000000", "0.42"),
    ];

    for (r, row) in financial_data.iter().enumerate() {
        let y = 120.0 + r as f64 * 25.0;
        let cols = [row.0, row.1, row.2];
        for (c, &text) in cols.iter().enumerate() {
            let x = 60.0 + c as f64 * 140.0;
            page3_elements.push(PdfTextElement {
                text: text.to_string(),
                bounds: Rect::new(x, y, 90.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 10.0,
                color: Color::rgb(0, 0, 0),
                baseline: y + 10.0,
            });
        }
    }

    doc.add_page(PdfPage {
        page_index: 2,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: page3_elements,
        vector_paths: vec![
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(60.0, 260.0)),
                    PathOp::LineTo(Point::new(480.0, 260.0)),
                ],
                bounds: Rect::new(60.0, 260.0, 420.0, 2.0),
                stroke_color: Some(Color::rgb(82, 102, 235)),
                fill_color: None,
                line_width: 2.0,
            },
        ],
        raw_stream: Vec::new(),
    });

    // 1. Export document to ISO 32000 PDF byte stream
    let pdf_bytes = doc.export_to_bytes();
    assert!(pdf_bytes.starts_with(b"%PDF-1.7"));
    assert!(pdf_bytes.ends_with(b"%%EOF\n"));

    // 2. Parse back and verify exact page count and elements
    let loaded = PdfDocument::load_from_bytes(&pdf_bytes).expect("PDF loaded successfully");
    assert_eq!(loaded.pages.len(), 3);
    assert_eq!(loaded.pages[0].text_elements[0].text, "OpenWPS Executive Report");
    assert_eq!(loaded.pages[1].text_elements.len(), 16); // 4x4 table
    assert_eq!(loaded.pages[2].text_elements.len(), 12); // 4x3 table
}

#[test]
fn test_xycut_and_table_layout_reconstruction() {
    // 1. Multi-column XY-cut layout test
    let multi_col_page = PdfPage {
        page_index: 0,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: vec![
            // Left Column (x: 50..240)
            PdfTextElement {
                text: "Left Column Paragraph 1 Line A".to_string(),
                bounds: Rect::new(50.0, 100.0, 180.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 110.0,
            },
            PdfTextElement {
                text: "Left Column Paragraph 1 Line B".to_string(),
                bounds: Rect::new(50.0, 120.0, 180.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 130.0,
            },
            PdfTextElement {
                text: "Left Column Paragraph 2 Line A".to_string(),
                bounds: Rect::new(50.0, 160.0, 180.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 170.0,
            },
            // Right Column (x: 320..510)
            PdfTextElement {
                text: "Right Column Paragraph 1 Line A".to_string(),
                bounds: Rect::new(320.0, 100.0, 180.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 110.0,
            },
            PdfTextElement {
                text: "Right Column Paragraph 1 Line B".to_string(),
                bounds: Rect::new(320.0, 120.0, 180.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(0, 0, 0),
                baseline: 130.0,
            },
        ],
        vector_paths: Vec::new(),
        raw_stream: Vec::new(),
    };

    let layout_col = SpatialReconstructor::reconstruct_page(&multi_col_page);
    // Verify reading order: all Left column blocks precede Right column blocks
    assert!(layout_col.blocks.len() >= 2);
    let first_block_text = layout_col.blocks[0].plain_text();
    assert!(first_block_text.contains("Left Column"), "First reading order block must be Left column");
    let last_block_text = layout_col.blocks.last().unwrap().plain_text();
    assert!(last_block_text.contains("Right Column"), "Last reading order block must be Right column");

    // 2. Ruled 4x4 Table Reconstruction
    let mut ruled_page_paths = Vec::new();
    let mut ruled_page_elems = Vec::new();
    for r in 0..=4 {
        let y = 100.0 + r as f64 * 30.0;
        ruled_page_paths.push(PdfVectorPath {
            operations: vec![PathOp::MoveTo(Point::new(50.0, y)), PathOp::LineTo(Point::new(450.0, y))],
            bounds: Rect::new(50.0, y, 400.0, 1.0),
            stroke_color: Some(Color::rgb(0, 0, 0)),
            fill_color: None,
            line_width: 1.0,
        });
    }
    for c in 0..=4 {
        let x = 50.0 + c as f64 * 100.0;
        ruled_page_paths.push(PdfVectorPath {
            operations: vec![PathOp::MoveTo(Point::new(x, 100.0)), PathOp::LineTo(Point::new(x, 220.0))],
            bounds: Rect::new(x, 100.0, 1.0, 120.0),
            stroke_color: Some(Color::rgb(0, 0, 0)),
            fill_color: None,
            line_width: 1.0,
        });
    }
    for r in 0..4 {
        for c in 0..4 {
            ruled_page_elems.push(PdfTextElement {
                text: format!("R{}C{}", r + 1, c + 1),
                bounds: Rect::new(60.0 + c as f64 * 100.0, 110.0 + r as f64 * 30.0, 60.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 10.0,
                color: Color::rgb(0, 0, 0),
                baseline: 120.0 + r as f64 * 30.0,
            });
        }
    }
    let ruled_page = PdfPage {
        page_index: 1,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: ruled_page_elems,
        vector_paths: ruled_page_paths,
        raw_stream: Vec::new(),
    };
    let layout_ruled = SpatialReconstructor::reconstruct_page(&ruled_page);
    assert_eq!(layout_ruled.tables.len(), 1, "Must detect exactly 1 ruled table");
    assert_eq!(layout_ruled.tables[0].rows, 4);
    assert_eq!(layout_ruled.tables[0].cols, 4);

    // 3. Borderless 4x3 Table Reconstruction
    let mut borderless_elems = Vec::new();
    for r in 0..4 {
        let y = 120.0 + r as f64 * 25.0;
        for c in 0..3 {
            let x = 60.0 + c as f64 * 130.0;
            borderless_elems.push(PdfTextElement {
                text: format!("Item_{}_{}", r + 1, c + 1),
                bounds: Rect::new(x, y, 80.0, 12.0),
                font_name: "Helvetica".to_string(),
                font_size: 10.0,
                color: Color::rgb(0, 0, 0),
                baseline: y + 10.0,
            });
        }
    }
    let borderless_page = PdfPage {
        page_index: 2,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: borderless_elems,
        vector_paths: Vec::new(),
        raw_stream: Vec::new(),
    };
    let layout_borderless = SpatialReconstructor::reconstruct_page(&borderless_page);
    assert_eq!(layout_borderless.tables.len(), 1, "Must detect exactly 1 borderless table");
    assert_eq!(layout_borderless.tables[0].rows, 4);
    assert_eq!(layout_borderless.tables[0].cols, 3);
}

#[test]
fn test_direct_inline_pdf_text_editing() {
    let mut page = PdfPage {
        page_index: 0,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: vec![
            PdfTextElement {
                text: "Original Paragraph Text".to_string(),
                bounds: Rect::new(50.0, 100.0, 160.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 14.0,
                color: Color::rgb(0, 0, 0),
                baseline: 112.0,
            },
        ],
        vector_paths: Vec::new(),
        raw_stream: b"BT /F1 14 Tf 1 0 0 1 50 728 Tm (Original Paragraph Text) Tj ET\n".to_vec(),
    };

    // 1. Click paragraph to edit inline like Word
    let mut session = PdfEditSession::new(&page, Point::new(60.0, 105.0)).expect("Paragraph block found");
    session.insert_text(" - Revised & Expanded with High-Precision Flow");
    session.commit(&mut page);

    // 2. Wrap into document and export
    let mut doc = PdfDocument::new("Edited Doc");
    doc.add_page(page);
    let bytes = doc.export_to_bytes();

    // 3. Re-parse and verify updated content stream & reflowed glyphs
    let reloaded = PdfDocument::load_from_bytes(&bytes).expect("Reloaded");
    let combined_text: String = reloaded.pages[0].text_elements.iter().map(|e| e.text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(combined_text.contains("Revised & Expanded"), "Edited content must be persisted in reloaded PDF");
    assert!(reloaded.pages[0].raw_stream.windows(7).any(|w| w == b"Revised"));
}

#[test]
fn test_pdf_to_docx_xlsx_pptx_ast_conversion() {
    let mut doc = PdfDocument::new("Annual Financial Model");

    // Page 1: Heading + Ruled Table
    let mut page1_elements = vec![
        PdfTextElement {
            text: "Financial Highlights".to_string(),
            bounds: Rect::new(50.0, 50.0, 220.0, 18.0),
            font_name: "Helvetica-Bold".to_string(),
            font_size: 18.0,
            color: Color::rgb(0, 0, 0),
            baseline: 65.0,
        },
    ];

    let mut page1_paths = Vec::new();
    // 2x2 table grid
    for r in 0..=2 {
        let y = 100.0 + r as f64 * 30.0;
        page1_paths.push(PdfVectorPath {
            operations: vec![PathOp::MoveTo(Point::new(50.0, y)), PathOp::LineTo(Point::new(250.0, y))],
            bounds: Rect::new(50.0, y, 200.0, 1.0),
            stroke_color: Some(Color::rgb(0, 0, 0)),
            fill_color: None,
            line_width: 1.0,
        });
    }
    for c in 0..=2 {
        let x = 50.0 + c as f64 * 100.0;
        page1_paths.push(PdfVectorPath {
            operations: vec![PathOp::MoveTo(Point::new(x, 100.0)), PathOp::LineTo(Point::new(x, 160.0))],
            bounds: Rect::new(x, 100.0, 1.0, 60.0),
            stroke_color: Some(Color::rgb(0, 0, 0)),
            fill_color: None,
            line_width: 1.0,
        });
    }

    page1_elements.push(PdfTextElement {
        text: "Quarter".to_string(),
        bounds: Rect::new(55.0, 110.0, 80.0, 12.0),
        font_name: "Helvetica-Bold".to_string(),
        font_size: 10.0,
        color: Color::rgb(0, 0, 0),
        baseline: 120.0,
    });
    page1_elements.push(PdfTextElement {
        text: "Revenue".to_string(),
        bounds: Rect::new(155.0, 110.0, 80.0, 12.0),
        font_name: "Helvetica-Bold".to_string(),
        font_size: 10.0,
        color: Color::rgb(0, 0, 0),
        baseline: 120.0,
    });
    page1_elements.push(PdfTextElement {
        text: "Q1".to_string(),
        bounds: Rect::new(55.0, 140.0, 80.0, 12.0),
        font_name: "Helvetica".to_string(),
        font_size: 10.0,
        color: Color::rgb(0, 0, 0),
        baseline: 150.0,
    });
    page1_elements.push(PdfTextElement {
        text: "$24,500.00".to_string(),
        bounds: Rect::new(155.0, 140.0, 80.0, 12.0),
        font_name: "Helvetica".to_string(),
        font_size: 10.0,
        color: Color::rgb(0, 0, 0),
        baseline: 150.0,
    });

    doc.add_page(PdfPage {
        page_index: 0,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: page1_elements,
        vector_paths: page1_paths,
        raw_stream: Vec::new(),
    });

    // 1. PDF to Writer (.docx AST & export)
    let writer_doc = doc.to_writer_document();
    assert_eq!(writer_doc.title, "Annual Financial Model");
    assert!(!writer_doc.sections.is_empty());
    let docx_bytes = wps_writer::DocxCodec::export_docx(&writer_doc).expect("Export DOCX succeeds");
    assert!(docx_bytes.len() > 100);

    // 2. PDF to Sheets (.xlsx Workbook & export)
    let wb = doc.to_sheets_workbook(PdfToExcelMode::OneSheetPerTable);
    assert!(!wb.sheets.is_empty());
    // Verify typed numeric parsing: $24,500.00 -> 24500.0
    let c11 = wb.sheets[0].get_cell_value(wps_sheets::CellCoordinate::new(1, 1));
    assert_eq!(c11, wps_sheets::CellValue::Number(24500.0));
    let xlsx_bytes = wps_sheets::xlsx::XlsxCodec::export_workbook_xlsx(&wb).expect("Export XLSX succeeds");
    assert!(xlsx_bytes.len() > 100);

    // 3. PDF to Slides (.pptx Presentation & export)
    let deck = doc.to_slides_deck();
    assert_eq!(deck.slides.len(), 1);
    let pptx_bytes = wps_slides::pptx::PptxCodec::export_pptx(&deck).expect("Export PPTX succeeds");
    assert!(pptx_bytes.len() > 100);

    // 4. Native Office-to-PDF Exporters
    let from_writer_pdf = PdfConverter::writer_to_pdf(&writer_doc);
    assert!(from_writer_pdf.starts_with(b"%PDF-1.7"));

    let from_sheets_pdf = PdfConverter::sheets_to_pdf(&wb);
    assert!(from_sheets_pdf.starts_with(b"%PDF-1.7"));

    let from_slides_pdf = PdfConverter::slides_to_pdf(&deck);
    assert!(from_slides_pdf.starts_with(b"%PDF-1.7"));
}

#[test]
fn test_annotations_organizer_and_split_merge() {
    let mut doc = PdfDocument::new("Multi-Page Blueprint");

    // Create 6 pages
    for i in 0..6 {
        doc.add_page(PdfPage {
            page_index: i,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: vec![
                PdfTextElement {
                    text: format!("Blueprint Section Page {}", i + 1),
                    bounds: Rect::new(50.0, 50.0, 200.0, 14.0),
                    font_name: "Helvetica".to_string(),
                    font_size: 14.0,
                    color: Color::rgb(0, 0, 0),
                    baseline: 62.0,
                },
            ],
            vector_paths: Vec::new(),
            raw_stream: Vec::new(),
        });
    }

    // Add 4 distinct annotations on Page 0
    doc.add_annotation(0, Annotation::highlight("a1", 0, Rect::new(50.0, 50.0, 150.0, 14.0)));
    doc.add_annotation(0, Annotation::callout("a2", 0, Point::new(40.0, 40.0), Rect::new(100.0, 100.0, 120.0, 40.0), "Check dimensions", Color::rgb(0, 122, 255)));
    doc.add_annotation(0, Annotation::measurement("a3", 0, Point::new(50.0, 200.0), Point::new(150.0, 200.0), 1.0, "mm"));
    doc.add_annotation(0, Annotation::stamp("a4", 0, Rect::new(300.0, 50.0, 120.0, 40.0), StampType::Approved, "Lead Architect", "2026-10-29"));

    // 1. Split 6-page PDF into two 3-page PDFs
    let chunks = doc.split(SplitMode::EveryNPages(3));
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].1.pages.len(), 3);
    assert_eq!(chunks[1].1.pages.len(), 3);

    // 2. Rotate page 2 by 90 degrees in chunk 0
    let mut part1 = chunks[0].1.clone();
    PageOrganizer::rotate_page(&mut part1.pages[1], 90);
    assert_eq!(part1.pages[1].rotation, 90);
    assert_eq!(part1.pages[1].width, 842.0);
    assert_eq!(part1.pages[1].height, 595.0);

    // 3. Merge them back together
    let part2 = chunks[1].1.clone();
    PageOrganizer::merge_pages(&mut part1.pages, part2.pages);
    assert_eq!(part1.pages.len(), 6);

    // 4. Verify annotations survived on page 0
    assert!(doc.annotations.contains_key(&0));
    assert_eq!(doc.annotations[&0].len(), 4);
}

#[test]
fn test_true_redaction_and_aes256_encryption() {
    let page = PdfPage {
        page_index: 0,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: vec![
            PdfTextElement {
                text: "CONFIDENTIAL SSN: 999-00-1234".to_string(),
                bounds: Rect::new(50.0, 100.0, 220.0, 16.0),
                font_name: "Helvetica".to_string(),
                font_size: 14.0,
                color: Color::rgb(0, 0, 0),
                baseline: 112.0,
            },
            PdfTextElement {
                text: "Public verified citizen status".to_string(),
                bounds: Rect::new(50.0, 150.0, 220.0, 16.0),
                font_name: "Helvetica".to_string(),
                font_size: 14.0,
                color: Color::rgb(0, 0, 0),
                baseline: 162.0,
            },
        ],
        vector_paths: Vec::new(),
        raw_stream: b"BT /F1 14 Tf 1 0 0 1 50 728 Tm (CONFIDENTIAL SSN: 999-00-1234) Tj ET\nBT /F1 14 Tf 1 0 0 1 50 678 Tm (Public verified citizen status) Tj ET\n".to_vec(),
    };

    let mut doc = PdfDocument::new("Redaction Test");
    doc.add_page(page);

    // 1. Apply true permanent redaction over the SSN
    doc.redact_region(0, Rect::new(45.0, 95.0, 230.0, 25.0), Some("[REDACTED - PII]"));

    let redacted_bytes = doc.export_to_bytes();

    // Verify raw byte stream NO LONGER contains the sensitive SSN
    assert!(
        !redacted_bytes.windows(11).any(|w| w == b"999-00-1234"),
        "Raw PDF bytes must not contain the sensitive SSN anywhere"
    );

    // Re-parse and verify text element was completely removed
    let loaded = PdfDocument::load_from_bytes(&redacted_bytes).expect("Parsed after redaction");
    let has_ssn = loaded.pages[0].text_elements.iter().any(|e| e.text.contains("999-00-1234"));
    assert!(!has_ssn, "Text elements must not contain the sensitive SSN");
    assert!(loaded.pages[0].text_elements.iter().any(|e| e.text == "[REDACTED - PII]"));

    // 2. Encrypt with AES-256 password protection
    let enc_bytes = loaded.encrypt("SecurePass123", "OwnerPass456");
    assert!(enc_bytes.windows(8).any(|w| w == b"/Encrypt"));

    // Verify opening without password fails
    assert!(PdfDocument::load_with_password(&enc_bytes, "").is_err(), "Opening without password must fail");

    // Verify opening with wrong password fails
    assert!(PdfDocument::load_with_password(&enc_bytes, "WrongPassword").is_err(), "Opening with wrong password must fail");

    // Verify opening with correct user password succeeds
    let unlocked = PdfDocument::load_with_password(&enc_bytes, "SecurePass123").expect("Decryption succeeds");
    assert_eq!(unlocked.pages.len(), 1);
    assert_eq!(unlocked.pages[0].text_elements[0].text, "Public verified citizen status");
}
