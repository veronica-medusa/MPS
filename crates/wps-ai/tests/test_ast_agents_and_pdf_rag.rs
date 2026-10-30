use wps_ai::pdf_ai::{PdfAi, PdfSpatialIndex};
use wps_ai::sheets_ai::SheetsAi;
use wps_ai::slides_ai::{DeckOutline, PresentationThemeConfig, SlideOutlineItem, SlidesAi};
use wps_ai::writer_ai::{RewriteTone, WriterAi};
use wps_pdf::layout_reconstructor::{
    EditableParagraphBlock, PdfRect, ReconstructedTextLine, ReconstructedWord, RgbaColor,
    SemanticRole, TextAlignment,
};
use wps_sheets::cell::{CellCoordinate, CellRange, CellValue};
use wps_sheets::sparse_matrix::SparseMatrix;
use wps_slides::scene_graph::SlideNode;
use wps_writer::ast::{BlockElement, Document as WriterDocument, Paragraph, TextRun};
use wps_writer::revisions::RevisionManager;

#[test]
fn test_writer_ai_track_changes_redline_generation() {
    let mut doc = WriterDocument::new("Quarterly Review", "Author");
    let mut p = Paragraph::new("p-1");
    let original_text = "We did ok this quarter.";
    p.add_run(TextRun::new(original_text));
    doc.add_paragraph(p);

    let mut rev_mgr = RevisionManager::new();
    let res = WriterAi::rewrite_selection(
        &mut doc,
        &mut rev_mgr,
        original_text,
        RewriteTone::Formal,
        true, // as_track_changes
    );

    assert_eq!(res.revisions_created.len(), 2);
    assert_eq!(rev_mgr.revisions.len(), 2);

    let del_rev = &rev_mgr.revisions[0];
    assert_eq!(del_rev.author, "WPS AI");
    match &del_rev.revision_type {
        wps_writer::revisions::RevisionType::Delete { deleted_text } => {
            assert_eq!(deleted_text, original_text);
        }
        _ => panic!("Expected Delete revision"),
    }

    let ins_rev = &rev_mgr.revisions[1];
    assert_eq!(ins_rev.author, "WPS AI");
    assert!(matches!(ins_rev.revision_type, wps_writer::revisions::RevisionType::Insert));

    // Verify document plain text contains the new text
    match &doc.sections[0].elements[0] {
        BlockElement::Paragraph(p) => {
            assert!(p.plain_text().contains("substantial operational progress"));
        }
        _ => panic!("Expected paragraph"),
    }
}

#[test]
fn test_sheets_ai_nl_to_formula_and_zscore_anomaly_detection() {
    let mut matrix = SparseMatrix::new();

    // Setup headers in Row 0
    matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Date".into()));
    matrix.set_value(CellCoordinate::new(0, 1), CellValue::Text("Region".into()));
    matrix.set_value(CellCoordinate::new(0, 2), CellValue::Text("Revenue".into()));

    // Setup rows 1..10 with normal numbers, and row 11 with extreme outlier
    for r in 1..=10 {
        matrix.set_value(CellCoordinate::new(r, 0), CellValue::Text(format!("2026-07-{:02}", r)));
        matrix.set_value(CellCoordinate::new(r, 1), CellValue::Text("APAC".into()));
        matrix.set_value(CellCoordinate::new(r, 2), CellValue::Number(100.0 + (r as f64) * 2.0));
    }
    // Extreme outlier in Row 11: 5000.0 (normal is ~110.0)
    matrix.set_value(CellCoordinate::new(11, 0), CellValue::Text("2026-07-11".into()));
    matrix.set_value(CellCoordinate::new(11, 1), CellValue::Text("APAC".into()));
    matrix.set_value(CellCoordinate::new(11, 2), CellValue::Number(5000.0));

    let range = CellRange::new(CellCoordinate::new(0, 0), CellCoordinate::new(11, 2));
    let schema = SheetsAi::extract_schema(&matrix, range);
    assert_eq!(schema.columns.len(), 3);

    // 1. NL to Formula
    let formula = SheetsAi::natural_language_to_formula(&schema, "sum revenue in apac");
    assert!(formula.starts_with('='));
    assert!(formula.contains("SUM"));

    // 2. NL to Python
    let py_code = SheetsAi::natural_language_to_python(&schema, "group by region and sum revenue");
    assert!(py_code.starts_with("=PY("));
    assert!(py_code.contains("groupby"));

    // 3. Z-score Anomaly Detection
    let report = SheetsAi::detect_anomalies_and_insights(&matrix, range);
    assert!(!report.anomalies.is_empty(), "Should detect the 5000.0 outlier");
    let outlier = &report.anomalies[0];
    assert_eq!(outlier.coord, CellCoordinate::new(11, 2));
    assert_eq!(outlier.value, 5000.0);
    assert!(outlier.z_score.abs() > 2.5, "Outlier Z-score must exceed 2.5");
}

#[test]
fn test_slides_ai_outline_compiler_with_connectors_and_morph() {
    let outline = DeckOutline {
        title: "Enterprise Strategy 2026".to_string(),
        slides: vec![
            SlideOutlineItem {
                title: "Process Roadmap".to_string(),
                layout: "ProcessTimeline".to_string(),
                bullets: vec![
                    "Stage 1: Architecture".to_string(),
                    "Stage 2: Deployment".to_string(),
                    "Stage 3: Enterprise Scale".to_string(),
                ],
            },
        ],
    };

    let theme = PresentationThemeConfig::default();
    let deck = SlidesAi::compile_outline_to_presentation(&outline, theme);

    // Initial title slide + 1 generated slide = 2 slides
    assert_eq!(deck.slides.len(), 2);
    let roadmap_slide = &deck.slides[1];
    assert_eq!(roadmap_slide.title, "Process Roadmap");
    assert!(roadmap_slide.transition.is_some());
    assert_eq!(roadmap_slide.transition.as_ref().unwrap().name, "Morph");

    // Check shapes and smart connector
    let has_connector = roadmap_slide.nodes.iter().any(|n| matches!(n, SlideNode::Connector(_)));
    assert!(has_connector, "ProcessTimeline slide must include smart connectors");

    let shapes_count = roadmap_slide.nodes.iter().filter(|n| matches!(n, SlideNode::AutoShape(_))).count();
    assert_eq!(shapes_count, 3, "ProcessTimeline must generate 3 step shapes");
}

#[test]
fn test_pdf_ai_spatial_rag_grounded_citations() {
    let mut index = PdfSpatialIndex::new();

    // Page 0 block: Governance overview
    let word_p0 = ReconstructedWord {
        text: "Executive governance mandates annual operational compliance.".to_string(),
        bbox: PdfRect::new(50.0, 50.0, 400.0, 65.0),
        baseline_y: 65.0,
        font_name: "Helvetica".to_string(),
        font_size_pt: 12.0,
        is_bold: false,
        is_italic: false,
        color: RgbaColor::default(),
        source_run_ids: vec![1],
    };

    let line_p0 = ReconstructedTextLine {
        id: 1,
        text: "Executive governance mandates annual operational compliance.".to_string(),
        words: vec![word_p0],
        bbox: PdfRect::new(50.0, 50.0, 400.0, 65.0),
        baseline_y: 65.0,
        dominant_font_name: "Helvetica".to_string(),
        dominant_font_size_pt: 12.0,
        is_bold: false,
        is_italic: false,
        dominant_color: RgbaColor::default(),
    };

    let block_p0 = EditableParagraphBlock {
        id: 101,
        bbox: PdfRect::new(50.0, 50.0, 400.0, 100.0),
        lines: vec![line_p0],
        alignment: TextAlignment::Left,
        first_line_indent_pt: 0.0,
        left_indent_pt: 0.0,
        right_indent_pt: 0.0,
        line_spacing_ratio: 1.2,
        role: SemanticRole::BodyParagraph,
        font_name: "Helvetica".to_string(),
        font_size_pt: 12.0,
        is_bold: false,
        is_italic: false,
        color: RgbaColor::default(),
    };
    index.index_paragraph(0, &block_p0);

    // Page 1 block: Financial EBITDA figures
    let word_p1 = ReconstructedWord {
        text: "Adjusted EBITDA reached 42.8 million dollars representing 38% margin growth.".to_string(),
        bbox: PdfRect::new(60.0, 200.0, 500.0, 240.0),
        baseline_y: 220.0,
        font_name: "Helvetica".to_string(),
        font_size_pt: 14.0,
        is_bold: true,
        is_italic: false,
        color: RgbaColor::default(),
        source_run_ids: vec![2],
    };

    let line_p1 = ReconstructedTextLine {
        id: 2,
        text: "Adjusted EBITDA reached 42.8 million dollars representing 38% margin growth.".to_string(),
        words: vec![word_p1],
        bbox: PdfRect::new(60.0, 200.0, 500.0, 240.0),
        baseline_y: 220.0,
        dominant_font_name: "Helvetica".to_string(),
        dominant_font_size_pt: 14.0,
        is_bold: true,
        is_italic: false,
        dominant_color: RgbaColor::default(),
    };

    let block_p1 = EditableParagraphBlock {
        id: 202,
        bbox: PdfRect::new(60.0, 200.0, 500.0, 280.0),
        lines: vec![line_p1],
        alignment: TextAlignment::Left,
        first_line_indent_pt: 0.0,
        left_indent_pt: 0.0,
        right_indent_pt: 0.0,
        line_spacing_ratio: 1.2,
        role: SemanticRole::BodyParagraph,
        font_name: "Helvetica".to_string(),
        font_size_pt: 14.0,
        is_bold: true,
        is_italic: false,
        color: RgbaColor::default(),
    };
    index.index_paragraph(1, &block_p1);

    // Query spatial RAG
    let (answer, citations) = PdfAi::query_pdf_rag(&index, "What was the adjusted EBITDA margin?");

    assert!(!citations.is_empty(), "Must return grounded citation chips");
    let top_citation = &citations[0];
    assert_eq!(top_citation.page_index, 1, "Citation must point to Page 1 (0-indexed)");
    assert_eq!(top_citation.block_id, 202);
    assert_eq!(top_citation.bbox, PdfRect::new(60.0, 200.0, 500.0, 280.0));
    assert!(top_citation.snippet_preview.contains("EBITDA"));
    assert!(answer.contains("[1]"), "Answer must cite chip [1]");
}
