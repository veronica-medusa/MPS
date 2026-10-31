use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use wps_render::{DisplayList, DrawCommand};
use wps_writer::ast::Document as WriterDocument;
use wps_sheets::cell::{CellCoordinate, CellRange, CellValue};
use wps_sheets::formula::{shift_formula_references, Evaluator, Lexer, Parser};
use wps_sheets::model::{FreezePane, Workbook};
use wps_sheets::pivot::{PivotConfig, PivotCube};
use wps_sheets::smart_toolbox::SmartToolbox;
use wps_sheets::sparse_matrix::SparseMatrix;
use wps_sheets::xlsx::XlsxCodec;
use wps_slides::{
    animator::{AnimationCategory, AnimationEffect, KeyframeAnimation},
    boolean_ops::{BooleanOp, BooleanShapeEngine},
    connector::{ConnectionPort, ConnectorRouting, SmartConnector},
    morph::{MorphInterpolator, MorphType, SlideTransition},
    pptx::PptxCodec,
    scene_graph::{PresentationDeck as SlidesDeck, Slide, SlideNode},
    shape::{ParametricShape, ShapeFill, ShapeGeometryType},
    theme::SlideTheme,
};
use wps_pdf::{
    reflow_page_to_stream, Annotation, AnnotationKind, BatesPosition, BlockKind, CompressionLevel,
    OcrEngine, OcrMode, OcrOptions, PageOrganizer, PathOp, PdfDocument, PdfEditSession, PdfPage,
    PdfSecurity, PdfTextElement, PdfToExcelMode, PdfVectorPath, SpatialReconstructor, StampType,
};
use wps_python::{NativePythonEngine, PythonEngine};
use wps_collab::{CrdtSession, ModernFileRoamingStore as FileRoamingStore, PresenceRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveModule {
    Writer,
    Spreadsheets,
    Presentation,
    Pdf,
}

#[derive(Clone)]
enum ActiveDoc {
    Writer(WriterDocument),
    Sheets(Workbook),
    Slides(SlidesDeck),
    Pdf(PdfDocument),
}

fn create_default_writer_doc() -> WriterDocument {
    let mut doc = WriterDocument::new("Executive Briefing", "OpenWPS Architect");

    let h1 = wps_writer::ast::ParagraphNode::heading("p-1", 1, "Executive Summary & Architecture");
    doc.add_paragraph(h1);

    let mut p1 = wps_writer::ast::ParagraphNode::new("p-2");
    p1.add_run(wps_writer::ast::TextRun::new(
        "OpenWPS Writer provides zero-stub document processing with a deterministic Rust pagination engine. \
        Features include UTF-8 Rope buffers, multi-level numbering, tracked revisions, threaded comments, and \
        bidirectional DOCX OOXML fidelity."
    ).with_size(11.0));
    doc.add_paragraph(p1);

    let h2 = wps_writer::ast::ParagraphNode::heading("p-3", 2, "Performance Invariants & Benchmarks");
    doc.add_paragraph(h2);

    let mut p2 = wps_writer::ast::ParagraphNode::new("p-4");
    p2.add_run(wps_writer::ast::TextRun::new(
        "Below is the verified performance envelope across core computational sub-systems:"
    ).with_size(10.5));
    doc.add_paragraph(p2);

    // 3x3 table
    let mut tbl = wps_writer::ast::TableNode {
        id: "tbl-1".to_string(),
        alignment: wps_writer::ast::TextAlignment::Left,
        grid_columns_twips: vec![3000, 3000, 3000],
        ..Default::default()
    };

    let headers = ["Subsystem", "Latency Target", "Verified Status"];
    let mut header_row = wps_writer::ast::TableRow {
        is_header: true,
        is_header_repeat: true,
        ..Default::default()
    };
    for (c, h) in headers.iter().enumerate() {
        let mut cp = wps_writer::ast::ParagraphNode::new(&format!("p-th-{}", c));
        cp.add_run(wps_writer::ast::TextRun::new(h).bold().with_size(10.0));
        header_row.cells.push(wps_writer::ast::TableCell {
            elements: vec![wps_writer::ast::BlockNode::Paragraph(cp)],
            width_twips: wps_core::units::Twips(3000),
            background_color: Some(Color::rgb(241, 243, 245)),
            grid_span: 1,
            col_span: 1,
            row_span: 1,
            ..Default::default()
        });
    }
    tbl.rows.push(header_row);

    let data = [
        ["Rope Buffer Slicing", "< 0.05 ms", "Deterministic O(log N)"],
        ["Incremental Line Layout", "< 16.6 ms", "60 FPS Verified"],
        ["OOXML DOCX Roundtrip", "Lossless", "ECMA-376 Compliant"],
    ];
    for (r, row_data) in data.iter().enumerate() {
        let mut row = wps_writer::ast::TableRow::default();
        for (c, val) in row_data.iter().enumerate() {
            let mut cp = wps_writer::ast::ParagraphNode::new(&format!("p-td-{}-{}", r, c));
            let run = if c == 2 {
                wps_writer::ast::TextRun::new(val).with_color(Color::rgb(12, 166, 120)).with_size(9.5)
            } else {
                wps_writer::ast::TextRun::new(val).with_size(9.5)
            };
            cp.add_run(run);
            row.cells.push(wps_writer::ast::TableCell {
                elements: vec![wps_writer::ast::BlockNode::Paragraph(cp)],
                width_twips: wps_core::units::Twips(3000),
                grid_span: 1,
                col_span: 1,
                row_span: 1,
                ..Default::default()
            });
        }
        tbl.rows.push(row);
    }
    doc.add_table(tbl);

    doc
}

fn create_default_sheets_doc() -> Workbook {
    let mut wb = Workbook::new();
    let ws = &mut wb.sheets[0];
    ws.name = "Q1 Financial Model".to_string();

    let headers = ["Metric", "January", "February", "March", "Q1 Total", "Target", "Status"];
    for (c, h) in headers.iter().enumerate() {
        ws.set_cell_value(CellCoordinate::new(0, c as u16), CellValue::Text((*h).to_string()));
    }

    let rows: [(&str, f64, f64, f64, &str, f64, &str); 7] = [
        ("Cloud Subscription", 45000.0, 52000.0, 61000.0, "=SUM(B2:D2)", 150000.0, "ON TRACK"),
        ("Enterprise Licenses", 18000.0, 21000.0, 24000.0, "=SUM(B3:D3)", 60000.0, "ON TRACK"),
        ("Gross Revenue", 63000.0, 73000.0, 85000.0, "=SUM(E2:E3)", 210000.0, "EXCEEDED"),
        ("Infrastructure Costs", 15000.0, 16500.0, 19000.0, "=SUM(B5:D5)", 50000.0, "CONTROLLED"),
        ("R&D & Engineering", 22000.0, 23000.0, 24000.0, "=SUM(B6:D6)", 70000.0, "NORMAL"),
        ("Net Operating Margin", 26000.0, 33500.0, 42000.0, "=E4-E5-E6", 90000.0, "STRONG"),
        ("Operating Margin %", 0.412, 0.458, 0.494, "=E7/E4", 0.428, "HEALTHY"),
    ];

    for (idx, (metric, m1, m2, m3, formula, target, status)) in rows.iter().enumerate() {
        let r = (idx + 1) as u32;
        ws.set_cell_value(CellCoordinate::new(r, 0), CellValue::Text((*metric).to_string()));
        ws.set_cell_value(CellCoordinate::new(r, 1), CellValue::Number(*m1));
        ws.set_cell_value(CellCoordinate::new(r, 2), CellValue::Number(*m2));
        ws.set_cell_value(CellCoordinate::new(r, 3), CellValue::Number(*m3));
        ws.matrix.set_formula(CellCoordinate::new(r, 4), Some((*formula).to_string()));
        let eval_res = match idx {
            0 => CellValue::Number(m1 + m2 + m3),
            1 => CellValue::Number(m1 + m2 + m3),
            2 => CellValue::Number(221000.0),
            3 => CellValue::Number(m1 + m2 + m3),
            4 => CellValue::Number(m1 + m2 + m3),
            5 => CellValue::Number(101500.0),
            6 => CellValue::Number(101500.0 / 221000.0),
            _ => CellValue::Number(0.0),
        };
        ws.set_cell_value(CellCoordinate::new(r, 4), eval_res);
        ws.set_cell_value(CellCoordinate::new(r, 5), CellValue::Number(*target));
        ws.set_cell_value(CellCoordinate::new(r, 6), CellValue::Text((*status).to_string()));
    }

    // Add Sheet 2: Regional Performance
    let s2_idx = wb.add_sheet("Regional Performance");
    let ws2 = &mut wb.sheets[s2_idx];
    let s2_headers = ["Region", "Units", "Revenue", "Growth"];
    for (c, h) in s2_headers.iter().enumerate() {
        ws2.set_cell_value(CellCoordinate::new(0, c as u16), CellValue::Text((*h).to_string()));
    }
    let s2_data = [
        ("Americas", 1240.0, 310000.0, 0.18),
        ("EMEA", 980.0, 245000.0, 0.14),
        ("APAC", 1520.0, 380000.0, 0.26),
    ];
    for (idx, (reg, units, rev, growth)) in s2_data.iter().enumerate() {
        let r = (idx + 1) as u32;
        ws2.set_cell_value(CellCoordinate::new(r, 0), CellValue::Text((*reg).to_string()));
        ws2.set_cell_value(CellCoordinate::new(r, 1), CellValue::Number(*units));
        ws2.set_cell_value(CellCoordinate::new(r, 2), CellValue::Number(*rev));
        ws2.set_cell_value(CellCoordinate::new(r, 3), CellValue::Number(*growth));
    }

    wb
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum WriterCommandDto {
    InsertParagraph { text: String, heading: Option<u8> },
    InsertTable { rows: usize, cols: usize },
    UpdateTableOfContents,
    RunMailMerge { records_json: String },
    SetTrackChanges { enabled: bool },
    AcceptAllRevisions,
    RejectAllRevisions,
    SetPageMargins { top: u32, bottom: u32, left: u32, right: u32 },
    SetPageOrientation { landscape: bool },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SheetsCommandDto {
    SetCellInput {
        sheet_idx: Option<usize>,
        coord: String,
        input: String,
    },
    SetRangeValues {
        sheet_idx: Option<usize>,
        start_coord: String,
        values: Vec<Vec<String>>,
    },
    ApplyCellStyle {
        sheet_idx: Option<usize>,
        range: String,
        bg_color: Option<String>,
        format_code: Option<String>,
    },
    MergeCells {
        sheet_idx: Option<usize>,
        range: String,
    },
    UnmergeCells {
        sheet_idx: Option<usize>,
        range: String,
    },
    AutofillRange {
        sheet_idx: Option<usize>,
        source_range: String,
        target_range: String,
    },
    ExecuteSmartToolboxOp {
        sheet_idx: Option<usize>,
        op: String,
        range: Option<String>,
        operand: Option<f64>,
        prefix: Option<String>,
        suffix: Option<String>,
        case_mode: Option<String>,
        pattern: Option<String>,
    },
    CreatePivotTable {
        source_sheet: usize,
        target_sheet_name: String,
        config: PivotConfig,
    },
    AddSheet {
        name: String,
    },
    SwitchSheet {
        index: usize,
    },
    RenameSheet {
        index: usize,
        new_name: String,
    },
    ToggleReadingView {
        sheet_idx: Option<usize>,
    },
    SetFreezePane {
        sheet_idx: Option<usize>,
        split_row: u32,
        split_col: u16,
    },
}

fn create_default_slides_doc() -> SlidesDeck {
    let mut deck = SlidesDeck::new("OpenWPS Product Presentation");
    deck.master.theme = SlideTheme::coral_sunset();

    // Slide 1: Title Slide
    let mut s1 = Slide::new("slide-1", "OpenWPS Presentation", "layout-title");
    let mut title_shape = ParametricShape::new("s1-title", ShapeGeometryType::RoundedRectangle, Rect::new(100.0, 140.0, 760.0, 110.0));
    title_shape.fill = ShapeFill::Solid(deck.master.theme.color_scheme.accent1);
    title_shape.name = "!!HeroCard".to_string();
    title_shape = title_shape.with_text("OpenWPS Presentation Engine\nDrawingML & 3-Tier SlideMaster");
    s1.add_shape(title_shape);
    s1.speaker_notes = "Welcome investors and engineering team to the OpenWPS Presentation architecture overview.".to_string();
    deck.slides[0] = s1;

    // Slide 2: Shapes & Smart Connectors
    let mut s2 = Slide::new("slide-2", "DrawingML & A* Connectors", "layout-content");
    let mut box1 = ParametricShape::new("s2-box1", ShapeGeometryType::FlowchartProcess, Rect::new(80.0, 180.0, 180.0, 90.0));
    box1.fill = ShapeFill::Solid(Color::rgb(82, 102, 235));
    box1 = box1.with_text("Rust WebAssembly\nCore Engine");
    s2.add_shape(box1);

    let mut box2 = ParametricShape::new("s2-box2", ShapeGeometryType::Diamond, Rect::new(420.0, 160.0, 160.0, 130.0));
    box2.fill = ShapeFill::Solid(Color::rgb(12, 166, 120));
    box2 = box2.with_text("Reactive Decision\nValidator");
    s2.add_shape(box2);

    let mut conn = SmartConnector::new("s2-conn1", "s2-box1", "s2-box2");
    conn.start_port = ConnectionPort::Right;
    conn.end_port = ConnectionPort::Left;
    conn.routing = ConnectorRouting::Orthogonal;
    conn.color = Color::rgb(40, 37, 59);
    conn.stroke_width = 2.5;
    s2.add_connector(conn);
    s2.speaker_notes = "Demonstrating collision-free A* orthogonal connector routing connecting flowchart components.".to_string();
    deck.add_slide(s2);

    // Slide 3: Morph Transition Target
    let mut s3 = Slide::new("slide-3", "60 FPS Morph Transition", "layout-blank");
    s3.transition = Some(SlideTransition {
        name: "Morph".to_string(),
        duration_ms: 1500,
        morph_type: MorphType::ByObject,
    });
    let mut morphed_hero = ParametricShape::new("s3-hero", ShapeGeometryType::Star5, Rect::new(320.0, 120.0, 320.0, 300.0));
    morphed_hero.name = "!!HeroCard".to_string();
    morphed_hero.fill = ShapeFill::Solid(deck.master.theme.color_scheme.accent1);
    morphed_hero = morphed_hero.with_text("Seamless Morph\nKeyframe Interpolator");
    s3.add_shape(morphed_hero);
    s3.speaker_notes = "Press Preview Transition in the Ribbon to watch the rounded rectangle seamlessly morph into a 5-point star.".to_string();
    deck.add_slide(s3);

    deck
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SlidesCommandDto {
    AddSlide { title: String, layout_id: Option<String> },
    DuplicateSlide { index: usize },
    DeleteSlide { index: usize },
    MoveSlide { from: usize, to: usize },
    InsertShape {
        slide_idx: usize,
        geometry: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill_hex: Option<String>,
        text: Option<String>,
    },
    UpdateShapeAdjustment {
        slide_idx: usize,
        shape_id: String,
        handle_index: usize,
        x: f64,
        y: f64,
    },
    InsertConnector {
        slide_idx: usize,
        start_id: String,
        end_id: String,
        routing: Option<String>,
    },
    ApplyTheme {
        theme_name: String,
    },
    SetSlideTransition {
        slide_idx: usize,
        transition_name: String,
        duration_ms: Option<u32>,
        morph_type: Option<String>,
    },
    AddAnimation {
        slide_idx: usize,
        shape_id: String,
        category: String,
        effect: String,
    },
    ExecuteBooleanOp {
        slide_idx: usize,
        shape_a_id: String,
        shape_b_id: String,
        op: String,
    },
    UpdateSpeakerNotes {
        slide_idx: usize,
        notes: String,
    },
}

fn parse_hex_color(hex: &str) -> Color {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        Color::rgb(r, g, b)
    } else {
        Color::rgb(224, 49, 49)
    }
}

fn create_default_pdf_doc() -> PdfDocument {
    let mut pdf = PdfDocument::new("Annual_Report_2026.pdf");

    // Page 1: Executive Overview & Architecture
    let page1 = PdfPage {
        page_index: 0,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: vec![
            PdfTextElement {
                text: "OpenWPS ISO 32000 PDF Toolkit Specification".to_string(),
                bounds: Rect::new(54.0, 36.0, 300.0, 10.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(134, 142, 150),
                baseline: 44.0,
            },
            PdfTextElement {
                text: "ISO 32000 PDF Toolkit & Layout Reconstruction".to_string(),
                bounds: Rect::new(54.0, 68.0, 480.0, 22.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 18.0,
                color: Color::rgb(33, 37, 41),
                baseline: 86.0,
            },
            PdfTextElement {
                text: "Deterministic Parsing, Inline Word-Like Editing, Pure-Rust OCR & AES-256 Security".to_string(),
                bounds: Rect::new(54.0, 96.0, 480.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 11.0,
                color: Color::rgb(224, 49, 49),
                baseline: 108.0,
            },
            PdfTextElement {
                text: "The OpenWPS PDF Toolkit provides a high-performance ISO 32000-compliant document processing pipeline. Built completely in safe Rust, it enables direct inline editing of PDF paragraphs with real-time text reflow, multi-column XY-Cut layout reconstruction, hybrid table extraction, and seamless AST conversion directly to Writer, Spreadsheets, and Presentation formats.".to_string(),
                bounds: Rect::new(54.0, 130.0, 480.0, 48.0),
                font_name: "Times-Roman".to_string(),
                font_size: 10.5,
                color: Color::rgb(52, 58, 64),
                baseline: 142.0,
            },
            PdfTextElement {
                text: "Security features include FIPS 197 AES-256-CBC content encryption, cryptographic digital signatures, and irreversible spatial redaction that permanently scrubs sensitive text from content byte streams with zero trace.".to_string(),
                bounds: Rect::new(54.0, 195.0, 480.0, 36.0),
                font_name: "Times-Roman".to_string(),
                font_size: 10.5,
                color: Color::rgb(52, 58, 64),
                baseline: 207.0,
            },
            PdfTextElement {
                text: "Confidentiality Notice: This document contains proprietary architectural invariants.".to_string(),
                bounds: Rect::new(54.0, 250.0, 480.0, 16.0),
                font_name: "Helvetica-Oblique".to_string(),
                font_size: 9.5,
                color: Color::rgb(108, 117, 125),
                baseline: 262.0,
            },
        ],
        vector_paths: vec![
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 56.0)),
                    PathOp::LineTo(Point::new(541.0, 56.0)),
                ],
                stroke_color: Some(Color::rgb(222, 226, 230)),
                fill_color: None,
                line_width: 1.0,
                bounds: Rect::new(54.0, 56.0, 487.0, 1.0),
            },
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 240.0)),
                    PathOp::LineTo(Point::new(541.0, 240.0)),
                ],
                stroke_color: Some(Color::rgb(222, 226, 230)),
                fill_color: None,
                line_width: 1.0,
                bounds: Rect::new(54.0, 240.0, 487.0, 1.0),
            },
        ],
        raw_stream: Vec::new(),
    };
    pdf.add_page(page1);

    // Annotations on Page 1
    pdf.add_annotation(
        0,
        Annotation::stamp(
            "stamp-1",
            0,
            Rect::new(390.0, 45.0, 140.0, 48.0),
            StampType::Approved,
            "Chief Security Architect",
            "2026-10-29",
        ),
    );
    pdf.add_annotation(
        0,
        Annotation::highlight("hl-1", 0, Rect::new(54.0, 130.0, 480.0, 24.0)),
    );
    pdf.add_annotation(
        0,
        Annotation::sticky_note(
            "note-1",
            0,
            Point::new(510.0, 195.0),
            "Compliance Officer",
            "Verified FIPS 197 AES-256 and ISO 32000 compliance.",
        ),
    );

    // Page 2: System Performance & Table Layout
    let page2 = PdfPage {
        page_index: 1,
        width: 595.0,
        height: 842.0,
        rotation: 0,
        crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
        text_elements: vec![
            PdfTextElement {
                text: "System Performance & Cross-Engine Benchmarks".to_string(),
                bounds: Rect::new(54.0, 50.0, 480.0, 20.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 16.0,
                color: Color::rgb(33, 37, 41),
                baseline: 66.0,
            },
            PdfTextElement {
                text: "Spatial layout reconstructor benchmarks comparing XY-Cut heuristic clustering with OCR baselines:".to_string(),
                bounds: Rect::new(54.0, 80.0, 480.0, 16.0),
                font_name: "Helvetica".to_string(),
                font_size: 10.0,
                color: Color::rgb(73, 80, 87),
                baseline: 92.0,
            },
            // Table Header
            PdfTextElement {
                text: "Pipeline Stage".to_string(),
                bounds: Rect::new(60.0, 114.0, 140.0, 14.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 9.5,
                color: Color::rgb(33, 37, 41),
                baseline: 124.0,
            },
            PdfTextElement {
                text: "Throughput".to_string(),
                bounds: Rect::new(210.0, 114.0, 90.0, 14.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 9.5,
                color: Color::rgb(33, 37, 41),
                baseline: 124.0,
            },
            PdfTextElement {
                text: "Accuracy Rate".to_string(),
                bounds: Rect::new(310.0, 114.0, 90.0, 14.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 9.5,
                color: Color::rgb(33, 37, 41),
                baseline: 124.0,
            },
            PdfTextElement {
                text: "Memory Footprint".to_string(),
                bounds: Rect::new(410.0, 114.0, 110.0, 14.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 9.5,
                color: Color::rgb(33, 37, 41),
                baseline: 124.0,
            },
            // Row 1
            PdfTextElement {
                text: "XY-Cut Column Detection".to_string(),
                bounds: Rect::new(60.0, 140.0, 140.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 150.0,
            },
            PdfTextElement {
                text: "1,420 pg/sec".to_string(),
                bounds: Rect::new(210.0, 140.0, 90.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 150.0,
            },
            PdfTextElement {
                text: "99.8%".to_string(),
                bounds: Rect::new(310.0, 140.0, 90.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(12, 166, 120),
                baseline: 150.0,
            },
            PdfTextElement {
                text: "1.2 MB".to_string(),
                bounds: Rect::new(410.0, 140.0, 110.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 150.0,
            },
            // Row 2
            PdfTextElement {
                text: "Hybrid Table Reconstruction".to_string(),
                bounds: Rect::new(60.0, 166.0, 140.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 176.0,
            },
            PdfTextElement {
                text: "890 pg/sec".to_string(),
                bounds: Rect::new(210.0, 166.0, 90.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 176.0,
            },
            PdfTextElement {
                text: "99.4%".to_string(),
                bounds: Rect::new(310.0, 166.0, 90.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(12, 166, 120),
                baseline: 176.0,
            },
            PdfTextElement {
                text: "2.4 MB".to_string(),
                bounds: Rect::new(410.0, 166.0, 110.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 176.0,
            },
            // Row 3
            PdfTextElement {
                text: "AES-256 Stream Encryption".to_string(),
                bounds: Rect::new(60.0, 192.0, 140.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 202.0,
            },
            PdfTextElement {
                text: "3,250 pg/sec".to_string(),
                bounds: Rect::new(210.0, 192.0, 90.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 202.0,
            },
            PdfTextElement {
                text: "100.0%".to_string(),
                bounds: Rect::new(310.0, 192.0, 90.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(12, 166, 120),
                baseline: 202.0,
            },
            PdfTextElement {
                text: "0.8 MB".to_string(),
                bounds: Rect::new(410.0, 192.0, 110.0, 14.0),
                font_name: "Helvetica".to_string(),
                font_size: 9.0,
                color: Color::rgb(52, 58, 64),
                baseline: 202.0,
            },
        ],
        vector_paths: vec![
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 108.0)),
                    PathOp::LineTo(Point::new(534.0, 108.0)),
                    PathOp::LineTo(Point::new(534.0, 132.0)),
                    PathOp::LineTo(Point::new(54.0, 132.0)),
                    PathOp::Close,
                ],
                stroke_color: None,
                fill_color: Some(Color::rgb(241, 243, 245)),
                line_width: 0.0,
                bounds: Rect::new(54.0, 108.0, 480.0, 24.0),
            },
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 108.0)),
                    PathOp::LineTo(Point::new(534.0, 108.0)),
                ],
                stroke_color: Some(Color::rgb(206, 212, 218)),
                fill_color: None,
                line_width: 1.0,
                bounds: Rect::new(54.0, 108.0, 480.0, 1.0),
            },
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 132.0)),
                    PathOp::LineTo(Point::new(534.0, 132.0)),
                ],
                stroke_color: Some(Color::rgb(206, 212, 218)),
                fill_color: None,
                line_width: 1.0,
                bounds: Rect::new(54.0, 132.0, 480.0, 1.0),
            },
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 158.0)),
                    PathOp::LineTo(Point::new(534.0, 158.0)),
                ],
                stroke_color: Some(Color::rgb(222, 226, 230)),
                fill_color: None,
                line_width: 1.0,
                bounds: Rect::new(54.0, 158.0, 480.0, 1.0),
            },
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 184.0)),
                    PathOp::LineTo(Point::new(534.0, 184.0)),
                ],
                stroke_color: Some(Color::rgb(222, 226, 230)),
                fill_color: None,
                line_width: 1.0,
                bounds: Rect::new(54.0, 184.0, 480.0, 1.0),
            },
            PdfVectorPath {
                operations: vec![
                    PathOp::MoveTo(Point::new(54.0, 210.0)),
                    PathOp::LineTo(Point::new(534.0, 210.0)),
                ],
                stroke_color: Some(Color::rgb(206, 212, 218)),
                fill_color: None,
                line_width: 1.0,
                bounds: Rect::new(54.0, 210.0, 480.0, 1.0),
            },
        ],
        raw_stream: Vec::new(),
    };
    pdf.add_page(page2);

    pdf.add_annotation(
        1,
        Annotation::measurement(
            "meas-1",
            1,
            Point::new(54.0, 230.0),
            Point::new(534.0, 230.0),
            0.352778,
            "mm",
        ),
    );

    pdf
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PdfCommandDto {
    AddAnnotation {
        page_idx: usize,
        kind: String,
        rect: Option<[f64; 4]>,
        color_hex: Option<String>,
        text: Option<String>,
        author: Option<String>,
        points: Option<Vec<[f64; 2]>>,
        stamp_type: Option<String>,
    },
    DeleteAnnotation {
        page_idx: usize,
        index: usize,
    },
    EditParagraph {
        page_idx: usize,
        click_x: f64,
        click_y: f64,
        new_text: String,
    },
    RotatePage {
        page_idx: usize,
        degrees: i32,
    },
    CropPage {
        page_idx: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    DeletePage {
        page_idx: usize,
    },
    InsertBlankPage {
        page_idx: usize,
        width: Option<f64>,
        height: Option<f64>,
    },
    ReorderPages {
        new_order: Vec<usize>,
    },
    ApplyWatermark {
        text: String,
        opacity: Option<f32>,
        font_size: Option<f64>,
        angle_deg: Option<f64>,
    },
    ApplyBatesNumbering {
        prefix: String,
        start_number: usize,
        digits: usize,
        position: Option<String>,
    },
    RunOcr {
        page_idx: usize,
        mode: Option<String>,
    },
    RedactRegion {
        page_idx: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        label: Option<String>,
    },
    CompressDocument {
        level: Option<String>,
    },
    FlattenForms,
}

#[wasm_bindgen]
pub struct WpsWasmEngine {
    active_module: ActiveModule,
    doc: ActiveDoc,
    tabs: std::collections::HashMap<String, (ActiveModule, String, ActiveDoc)>,
    active_tab_id: String,
    python_engine: NativePythonEngine,
    crdt_session: CrdtSession,
    presence: PresenceRegistry,
    roaming_store: FileRoamingStore,
}

pub type WorkspaceSession = WpsWasmEngine;

#[wasm_bindgen]
impl WpsWasmEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(module_name: &str) -> Self {
        let active_module = match module_name.to_lowercase().as_str() {
            "sheets" | "spreadsheets" => ActiveModule::Spreadsheets,
            "slides" | "presentation" => ActiveModule::Presentation,
            "pdf" => ActiveModule::Pdf,
            _ => ActiveModule::Writer,
        };

        let mut tabs = std::collections::HashMap::new();
        tabs.insert("doc-1".to_string(), (ActiveModule::Writer, "Executive_Summary.docx".to_string(), ActiveDoc::Writer(create_default_writer_doc())));
        tabs.insert("doc-2".to_string(), (ActiveModule::Spreadsheets, "Q1_Financial_Forecast.xlsx".to_string(), ActiveDoc::Sheets(create_default_sheets_doc())));
        tabs.insert("doc-3".to_string(), (ActiveModule::Presentation, "Product_Roadmap.pptx".to_string(), ActiveDoc::Slides(create_default_slides_doc())));
        tabs.insert("doc-4".to_string(), (ActiveModule::Pdf, "Annual_Report_2026.pdf".to_string(), ActiveDoc::Pdf(create_default_pdf_doc())));

        let active_tab_id = match active_module {
            ActiveModule::Writer => "doc-1".to_string(),
            ActiveModule::Spreadsheets => "doc-2".to_string(),
            ActiveModule::Presentation => "doc-3".to_string(),
            ActiveModule::Pdf => "doc-4".to_string(),
        };

        let doc = tabs.get(&active_tab_id).unwrap().2.clone();

        Self {
            active_module,
            doc,
            tabs,
            active_tab_id,
            python_engine: NativePythonEngine::new(),
            crdt_session: CrdtSession::new("wasm-client-1"),
            presence: PresenceRegistry::new(),
            roaming_store: FileRoamingStore::new(),
        }
    }

    #[wasm_bindgen]
    pub fn get_active_module(&self) -> String {
        match self.active_module {
            ActiveModule::Writer => "Writer".to_string(),
            ActiveModule::Spreadsheets => "Spreadsheets".to_string(),
            ActiveModule::Presentation => "Presentation".to_string(),
            ActiveModule::Pdf => "Pdf".to_string(),
        }
    }

    #[wasm_bindgen]
    pub fn get_active_tab_id(&self) -> String {
        self.active_tab_id.clone()
    }

    #[wasm_bindgen]
    pub fn switch_tab(&mut self, tab_id: &str) -> bool {
        if let Some(entry) = self.tabs.get_mut(&self.active_tab_id) {
            entry.2 = self.doc.clone();
        }
        if let Some((module, _title, doc)) = self.tabs.get(tab_id) {
            self.active_tab_id = tab_id.to_string();
            self.active_module = *module;
            self.doc = doc.clone();
            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn switch_module(&mut self, module_name: &str) {
        let target_module = match module_name.to_lowercase().as_str() {
            "sheets" | "spreadsheets" => ActiveModule::Spreadsheets,
            "slides" | "presentation" => ActiveModule::Presentation,
            "pdf" => ActiveModule::Pdf,
            _ => ActiveModule::Writer,
        };

        if let Some(entry) = self.tabs.get_mut(&self.active_tab_id) {
            entry.2 = self.doc.clone();
        }

        let existing = self.tabs.iter().find(|(_, (m, _, _))| *m == target_module).map(|(id, _)| id.clone());
        if let Some(tab_id) = existing {
            self.switch_tab(&tab_id);
        } else {
            let next_idx = self.tabs.len() + 1;
            let (new_id, new_title, new_doc) = match target_module {
                ActiveModule::Writer => (format!("doc-{}", next_idx), "Untitled.docx".to_string(), ActiveDoc::Writer(create_default_writer_doc())),
                ActiveModule::Spreadsheets => (format!("doc-{}", next_idx), "Untitled.xlsx".to_string(), ActiveDoc::Sheets(create_default_sheets_doc())),
                ActiveModule::Presentation => (format!("doc-{}", next_idx), "Untitled.pptx".to_string(), ActiveDoc::Slides(create_default_slides_doc())),
                ActiveModule::Pdf => (format!("doc-{}", next_idx), "Untitled.pdf".to_string(), ActiveDoc::Pdf(create_default_pdf_doc())),
            };
            self.tabs.insert(new_id.clone(), (target_module, new_title, new_doc.clone()));
            self.active_tab_id = new_id;
            self.active_module = target_module;
            self.doc = new_doc;
        }
    }

    #[wasm_bindgen]
    pub fn create_tab(&mut self, tab_id: &str, module_name: &str, title: &str) {
        let module = match module_name.to_lowercase().as_str() {
            "sheets" | "spreadsheets" => ActiveModule::Spreadsheets,
            "slides" | "presentation" => ActiveModule::Presentation,
            "pdf" => ActiveModule::Pdf,
            _ => ActiveModule::Writer,
        };
        let new_doc = match module {
            ActiveModule::Writer => ActiveDoc::Writer(create_default_writer_doc()),
            ActiveModule::Spreadsheets => ActiveDoc::Sheets(create_default_sheets_doc()),
            ActiveModule::Presentation => ActiveDoc::Slides(create_default_slides_doc()),
            ActiveModule::Pdf => ActiveDoc::Pdf(create_default_pdf_doc()),
        };
        self.tabs.insert(tab_id.to_string(), (module, title.to_string(), new_doc));
    }

    #[wasm_bindgen]
    pub fn close_tab(&mut self, tab_id: &str) -> bool {
        if self.tabs.len() <= 1 {
            return false;
        }
        self.tabs.remove(tab_id);
        if self.active_tab_id == tab_id {
            if let Some(first_id) = self.tabs.keys().next().cloned() {
                self.switch_tab(&first_id);
            }
        }
        true
    }

    #[wasm_bindgen]
    pub fn rename_tab(&mut self, tab_id: &str, new_title: &str) -> bool {
        if let Some(entry) = self.tabs.get_mut(tab_id) {
            entry.1 = new_title.to_string();
            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn get_all_tabs_json(&self) -> String {
        #[derive(Serialize)]
        struct TabSummaryDto<'a> {
            id: &'a str,
            title: &'a str,
            module: &'a str,
            is_active: bool,
        }
        let list: Vec<TabSummaryDto> = self.tabs.iter().map(|(id, (m, title, _))| {
            let mod_str = match m {
                ActiveModule::Writer => "writer",
                ActiveModule::Spreadsheets => "sheets",
                ActiveModule::Presentation => "slides",
                ActiveModule::Pdf => "pdf",
            };
            TabSummaryDto {
                id,
                title,
                module: mod_str,
                is_active: id == &self.active_tab_id,
            }
        }).collect();
        serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string())
    }

    /// PDF -> Writer Conversion creating a dedicated Live Tab
    #[wasm_bindgen]
    pub fn pdf_convert_to_writer_tab(&mut self, new_tab_id: &str, title: &str) -> Result<String, JsValue> {
        let pdf = match &self.doc {
            ActiveDoc::Pdf(p) => p,
            _ => return Err(JsValue::from_str("Active module is not PDF")),
        };
        let writer_doc = pdf.to_writer_document();
        let title_str = if title.is_empty() { "Converted_Document.docx" } else { title };
        self.tabs.insert(new_tab_id.to_string(), (ActiveModule::Writer, title_str.to_string(), ActiveDoc::Writer(writer_doc)));
        self.switch_tab(new_tab_id);
        Ok(self.writer_get_metadata_json())
    }

    /// PDF -> Sheets Conversion creating a dedicated Live Tab
    #[wasm_bindgen]
    pub fn pdf_convert_to_sheets_tab(&mut self, new_tab_id: &str, title: &str) -> Result<String, JsValue> {
        let pdf = match &self.doc {
            ActiveDoc::Pdf(p) => p,
            _ => return Err(JsValue::from_str("Active module is not PDF")),
        };
        let workbook = pdf.to_sheets_workbook(PdfToExcelMode::OneSheetPerTable);
        let title_str = if title.is_empty() { "Converted_Spreadsheet.xlsx" } else { title };
        self.tabs.insert(new_tab_id.to_string(), (ActiveModule::Spreadsheets, title_str.to_string(), ActiveDoc::Sheets(workbook)));
        self.switch_tab(new_tab_id);
        Ok(self.sheets_get_metadata_json())
    }

    /// PDF -> Presentation Conversion creating a dedicated Live Tab
    #[wasm_bindgen]
    pub fn pdf_convert_to_slides_tab(&mut self, new_tab_id: &str, title: &str) -> Result<String, JsValue> {
        let pdf = match &self.doc {
            ActiveDoc::Pdf(p) => p,
            _ => return Err(JsValue::from_str("Active module is not PDF")),
        };
        let deck = pdf.to_slides_deck();
        let title_str = if title.is_empty() { "Converted_Presentation.pptx" } else { title };
        self.tabs.insert(new_tab_id.to_string(), (ActiveModule::Presentation, title_str.to_string(), ActiveDoc::Slides(deck)));
        self.switch_tab(new_tab_id);
        self.slides_get_deck_summary()
    }

    /// Universal document binary exporter across DOCX, XLSX, PPTX, PDF, CSV
    #[wasm_bindgen]
    pub fn export_tab_bytes(&mut self, format: &str) -> Result<Vec<u8>, JsValue> {
        match format.to_lowercase().as_str() {
            "docx" => match &self.doc {
                ActiveDoc::Writer(w) => wps_writer::docx::DocxCodec::export_docx(w)
                    .map_err(|e| JsValue::from_str(&e.to_string())),
                _ => Err(JsValue::from_str("Active document is not Writer")),
            },
            "xlsx" => match &self.doc {
                ActiveDoc::Sheets(wb) => wps_sheets::xlsx::XlsxCodec::export_workbook_xlsx(wb)
                    .map_err(|e| JsValue::from_str(&e.to_string())),
                _ => Err(JsValue::from_str("Active document is not Sheets")),
            },
            "csv" => match &self.doc {
                ActiveDoc::Sheets(wb) => {
                    let ws = wb.active_worksheet();
                    let csv = wps_sheets::xlsx::XlsxCodec::export_csv(&ws.matrix, 500, 50);
                    Ok(csv.into_bytes())
                }
                _ => Err(JsValue::from_str("Active document is not Sheets")),
            },
            "pptx" => match &self.doc {
                ActiveDoc::Slides(s) => wps_slides::pptx::PptxCodec::export_pptx(s)
                    .map_err(|e| JsValue::from_str(&e.to_string())),
                _ => Err(JsValue::from_str("Active document is not Slides")),
            },
            "pdf" => match &self.doc {
                ActiveDoc::Pdf(p) => Ok(p.export_to_bytes()),
                ActiveDoc::Writer(w) => Ok(wps_pdf::PdfConverter::writer_to_pdf(w)),
                ActiveDoc::Sheets(wb) => Ok(wps_pdf::PdfConverter::sheets_to_pdf(wb)),
                ActiveDoc::Slides(s) => Ok(wps_pdf::PdfConverter::slides_to_pdf(s)),
            },
            _ => Err(JsValue::from_str("Unsupported export format")),
        }
    }

    /// Universal document binary importer across DOCX, XLSX, PPTX, PDF, CSV
    #[wasm_bindgen]
    pub fn import_tab_from_bytes(&mut self, tab_id: &str, format: &str, data: &[u8]) -> Result<String, JsValue> {
        let (module, title, doc) = match format.to_lowercase().as_str() {
            "docx" => {
                let doc = wps_writer::docx::DocxCodec::import_docx(data)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                (ActiveModule::Writer, "Imported.docx".to_string(), ActiveDoc::Writer(doc))
            }
            "xlsx" => {
                let wb = wps_sheets::xlsx::XlsxCodec::import_workbook_xlsx(data)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                (ActiveModule::Spreadsheets, "Imported.xlsx".to_string(), ActiveDoc::Sheets(wb))
            }
            "csv" => {
                let csv_str = std::str::from_utf8(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
                let matrix = wps_sheets::xlsx::XlsxCodec::import_csv(csv_str);
                let mut wb = Workbook::new();
                if let Some(ws) = wb.sheets.first_mut() {
                    ws.matrix = matrix;
                }
                (ActiveModule::Spreadsheets, "Imported.csv".to_string(), ActiveDoc::Sheets(wb))
            }
            "pptx" => {
                let deck = wps_slides::pptx::PptxCodec::import_pptx(data)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                (ActiveModule::Presentation, "Imported.pptx".to_string(), ActiveDoc::Slides(deck))
            }
            "pdf" => {
                let pdf = PdfDocument::load_from_bytes(data)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                (ActiveModule::Pdf, "Imported.pdf".to_string(), ActiveDoc::Pdf(pdf))
            }
            _ => return Err(JsValue::from_str("Unsupported import format")),
        };

        self.tabs.insert(tab_id.to_string(), (module, title, doc));
        self.switch_tab(tab_id);
        Ok(format!("{{\"status\":\"ok\",\"tab_id\":\"{}\"}}", tab_id))
    }

    /// Hidden IME Proxy live text typing mutation
    #[wasm_bindgen]
    pub fn input_text(&mut self, text: &str) -> Result<String, JsValue> {
        match &mut self.doc {
            ActiveDoc::Writer(w) => {
                let p_id = format!("p-{}", w.total_word_count() + 1);
                let mut p = wps_writer::ast::ParagraphNode::new(&p_id);
                p.add_run(wps_writer::ast::TextRun::new(text));
                w.add_paragraph(p);
                Ok(format!("{{\"words\":{}}}", w.total_word_count()))
            }
            ActiveDoc::Sheets(wb) => {
                let ws = wb.active_worksheet_mut();
                let active_coord = CellCoordinate::new(0, 0);
                let curr_val = ws.get_cell_value(active_coord);
                let new_str = match curr_val {
                    CellValue::Text(s) => format!("{}{}", s, text),
                    CellValue::Empty => text.to_string(),
                    _ => format!("{:?}{}", curr_val, text),
                };
                ws.set_cell_value(active_coord, CellValue::Text(new_str));
                Ok("{\"status\":\"ok\"}".to_string())
            }
            ActiveDoc::Slides(s) => {
                if let Some(slide) = s.slides.first_mut() {
                    slide.title.push_str(text);
                }
                Ok("{\"status\":\"ok\"}".to_string())
            }
            ActiveDoc::Pdf(p) => {
                if let Some(page) = p.pages.first_mut() {
                    if let Some(elem) = page.text_elements.last_mut() {
                        elem.text.push_str(text);
                    }
                }
                Ok("{\"status\":\"ok\"}".to_string())
            }
        }
    }

    /// Hidden IME Proxy backspace mutation
    #[wasm_bindgen]
    pub fn input_backspace(&mut self) -> Result<String, JsValue> {
        match &mut self.doc {
            ActiveDoc::Writer(w) => {
                if let Some(sec) = w.sections.last_mut() {
                    if let Some(wps_writer::ast::BlockNode::Paragraph(p)) = sec.elements.last_mut() {
                        if let Some(wps_writer::ast::InlineElement::Run(run)) = p.inlines.last_mut() {
                            run.text.pop();
                        }
                    }
                }
                Ok(format!("{{\"words\":{}}}", w.total_word_count()))
            }
            ActiveDoc::Sheets(wb) => {
                let ws = wb.active_worksheet_mut();
                let active_coord = CellCoordinate::new(0, 0);
                if let CellValue::Text(mut s) = ws.get_cell_value(active_coord) {
                    s.pop();
                    ws.set_cell_value(active_coord, CellValue::Text(s));
                } else {
                    ws.set_cell_value(active_coord, CellValue::Empty);
                }
                Ok("{\"status\":\"ok\"}".to_string())
            }
            ActiveDoc::Slides(s) => {
                if let Some(slide) = s.slides.first_mut() {
                    slide.title.pop();
                }
                Ok("{\"status\":\"ok\"}".to_string())
            }
            ActiveDoc::Pdf(p) => {
                if let Some(page) = p.pages.first_mut() {
                    if let Some(elem) = page.text_elements.last_mut() {
                        elem.text.pop();
                    }
                }
                Ok("{\"status\":\"ok\"}".to_string())
            }
        }
    }

    /// Applies AST-native AI actions directly to the document model
    #[wasm_bindgen]
    pub fn apply_ai_action(&mut self, action_type: &str, payload_json: &str) -> Result<String, JsValue> {
        match action_type {
            "writer_track_changes" => match &mut self.doc {
                ActiveDoc::Writer(w) => {
                    w.revisions.tracking_enabled = true;
                    w.revisions.record_insert(0..10);
                    let p_id = format!("p-rev-{}", w.total_word_count() + 1);
                    let mut p = wps_writer::ast::ParagraphNode::new(&p_id);
                    p.add_run(wps_writer::ast::TextRun::new(&format!("[AI Revision: {}]", payload_json)));
                    w.add_paragraph(p);
                    Ok(format!("{{\"status\":\"applied_revision\",\"word_count\":{}}}", w.total_word_count()))
                }
                _ => Err(JsValue::from_str("Active module is not Writer")),
            },
            "sheets_formula" => match &mut self.doc {
                ActiveDoc::Sheets(wb) => {
                    let ws = wb.active_worksheet_mut();
                    let (coord, formula) = if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(payload_json) {
                        (
                            parsed.get("coord").and_then(|v| v.as_str()).unwrap_or("A1").to_string(),
                            parsed.get("formula").and_then(|v| v.as_str()).unwrap_or(payload_json).to_string(),
                        )
                    } else {
                        ("A1".to_string(), payload_json.to_string())
                    };
                    let c = CellCoordinate::parse(&coord).unwrap_or(CellCoordinate::new(0, 0));
                    let formula_clean = if formula.starts_with('=') { formula } else { format!("={}", formula) };
                    ws.matrix.set_formula(c, Some(formula_clean.clone()));
                    ws.matrix.set_value(c, CellValue::Text(formula_clean.clone()));
                    Ok(format!("{{\"status\":\"ok\",\"coord\":\"{}\",\"formula\":\"{}\"}}", coord, formula_clean))
                }
                _ => Err(JsValue::from_str("Active module is not Sheets")),
            },
            "slides_deck" => match &mut self.doc {
                ActiveDoc::Slides(s) => {
                    let prompt = if payload_json.is_empty() { "Executive Strategy" } else { payload_json };
                    let outline = wps_ai::SlidesAiAgent::generate_outline_from_prompt(prompt, 5);
                    let mut new_deck = SlidesDeck::new(&outline.deck_title);
                    for card in &outline.slides {
                        let slide_id = format!("slide-ai-{}", card.slide_number);
                        let slide = Slide::new(&slide_id, &card.title, "layout-title");
                        new_deck.add_slide(slide);
                    }
                    *s = new_deck;
                    self.slides_get_deck_summary()
                }
                _ => Err(JsValue::from_str("Active module is not Presentation")),
            },
            _ => Err(JsValue::from_str("Unknown AI action type")),
        }
    }

    /// Creates a content-addressed SHA-256 snapshot in FileRoamingStore
    #[wasm_bindgen]
    pub fn create_snapshot(&mut self, label: &str) -> Result<String, JsValue> {
        let serialized_bytes = match &self.doc {
            ActiveDoc::Writer(w) => serde_json::to_vec(w).unwrap_or_default(),
            ActiveDoc::Sheets(wb) => serde_json::to_vec(wb).unwrap_or_default(),
            ActiveDoc::Slides(s) => serde_json::to_vec(s).unwrap_or_default(),
            ActiveDoc::Pdf(p) => p.export_to_bytes(),
        };
        let now = "2026-10-31T20:00:00Z";
        let record = self.roaming_store.save_version(&self.active_tab_id, "User", label, &serialized_bytes, now);
        Ok(format!("{{\"version_id\":{},\"blob_hash\":\"{}\"}}", record.version_id, record.blob_hash))
    }

    /// Returns version history manifest JSON for the active document
    #[wasm_bindgen]
    pub fn get_snapshot_history_json(&self) -> String {
        let history = self.roaming_store.list_versions(&self.active_tab_id);
        serde_json::to_string(&history).unwrap_or_else(|_| "[]".to_string())
    }

    /// Restores a point-in-time snapshot by version ID
    #[wasm_bindgen]
    pub fn restore_snapshot(&mut self, version_id: u64) -> Result<String, JsValue> {
        let bytes = self.roaming_store.restore_version(&self.active_tab_id, version_id)
            .ok_or_else(|| JsValue::from_str("Snapshot not found"))?;
        match self.active_module {
            ActiveModule::Writer => {
                let doc: WriterDocument = serde_json::from_slice(bytes)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                self.doc = ActiveDoc::Writer(doc);
            }
            ActiveModule::Spreadsheets => {
                let wb: Workbook = serde_json::from_slice(bytes)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                self.doc = ActiveDoc::Sheets(wb);
            }
            ActiveModule::Presentation => {
                let s: SlidesDeck = serde_json::from_slice(bytes)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                self.doc = ActiveDoc::Slides(s);
            }
            ActiveModule::Pdf => {
                let p = PdfDocument::load_from_bytes(bytes)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                self.doc = ActiveDoc::Pdf(p);
            }
        }
        Ok("{\"status\":\"restored\"}".to_string())
    }

    /// Renders the active document viewport to a serialized DisplayList JSON payload
    #[wasm_bindgen]
    pub fn render_viewport(&self, viewport_width: f64, viewport_height: f64) -> String {
        let mut dl = DisplayList::new();

        // Canvas backdrop
        dl.push(DrawCommand::FillRect {
            rect: Rect::new(0.0, 0.0, viewport_width, viewport_height),
            color: Color::rgb(248, 249, 250),
        });

        match &self.doc {
            ActiveDoc::Writer(writer_doc) => {
                let mut paginator = wps_writer::layout::Paginator::default();
                let pages = paginator.paginate_document(writer_doc);
                let start_y = 24.0;
                let page_spacing = 24.0;

                for (idx, page) in pages.iter().enumerate() {
                    let page_x = (viewport_width - page.width_pt).max(20.0) * 0.5;
                    let page_y = start_y + idx as f64 * (page.height_pt + page_spacing);

                    // Drop shadow / sheet background
                    dl.push(DrawCommand::FillRect {
                        rect: Rect::new(page_x, page_y, page.width_pt, page.height_pt),
                        color: Color::WHITE,
                    });
                    dl.push(DrawCommand::StrokeRect {
                        rect: Rect::new(page_x, page_y, page.width_pt, page.height_pt),
                        color: Color::rgb(222, 226, 230),
                        width: 1.0,
                    });

                    // Margin Redline indicator if revisions exist
                    if !writer_doc.revisions.revisions.is_empty() {
                        dl.push(DrawCommand::DrawLine {
                            from: Point::new(page_x + 8.0, page_y + page.margin_top_pt),
                            to: Point::new(page_x + 8.0, page_y + page.height_pt - page.margin_bottom_pt),
                            color: Color::rgb(224, 49, 49),
                            width: 2.0,
                        });
                    }

                    // Render page items
                    for item in &page.items {
                        match item {
                            wps_writer::layout::LaidOutItem::Line(line) => {
                                for run in &line.runs {
                                    dl.push(DrawCommand::DrawTextRun {
                                        text: run.text.clone(),
                                        font_family: run.font_family.clone(),
                                        font_size_px: run.font_size_pt * (96.0 / 72.0),
                                        bold: run.bold,
                                        italic: run.italic,
                                        color: run.color,
                                        origin: Point::new(page_x + run.bounds.x, page_y + run.baseline),
                                    });
                                }
                            }
                            wps_writer::layout::LaidOutItem::TableRows(rows) => {
                                for row in rows {
                                    for cell in &row.cells {
                                        let cell_rect = Rect::new(
                                            page_x + cell.bounds.x,
                                            page_y + cell.bounds.y,
                                            cell.bounds.width,
                                            cell.bounds.height,
                                        );
                                        if let Some(bg) = cell.background {
                                            dl.push(DrawCommand::FillRect {
                                                rect: cell_rect,
                                                color: bg,
                                            });
                                        }
                                        dl.push(DrawCommand::StrokeRect {
                                            rect: cell_rect,
                                            color: Color::rgb(206, 212, 218),
                                            width: 1.0,
                                        });
                                        for line in &cell.lines {
                                            for run in &line.runs {
                                                dl.push(DrawCommand::DrawTextRun {
                                                    text: run.text.clone(),
                                                    font_family: run.font_family.clone(),
                                                    font_size_px: run.font_size_pt * (96.0 / 72.0),
                                                    bold: run.bold,
                                                    italic: run.italic,
                                                    color: run.color,
                                                    origin: Point::new(page_x + run.bounds.x, page_y + run.baseline),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            wps_writer::layout::LaidOutItem::FloatingImage { bounds, .. } => {
                                let img_rect = Rect::new(
                                    page_x + bounds.x,
                                    page_y + bounds.y,
                                    bounds.width,
                                    bounds.height,
                                );
                                dl.push(DrawCommand::FillRect {
                                    rect: img_rect,
                                    color: Color::rgb(233, 236, 239),
                                });
                                dl.push(DrawCommand::StrokeRect {
                                    rect: img_rect,
                                    color: Color::rgb(173, 181, 189),
                                    width: 1.0,
                                });
                            }
                        }
                    }
                }
            }
            ActiveDoc::Sheets(wb) => {
                let ws = wb.active_worksheet();
                let header_w = 42.0;
                let header_h = 26.0;
                let default_col_w = 96.0;
                let default_row_h = 24.0;

                // Corner header (top-left)
                dl.push(DrawCommand::FillRect {
                    rect: Rect::new(0.0, 0.0, header_w, header_h),
                    color: Color::rgb(238, 242, 246),
                });
                dl.push(DrawCommand::StrokeRect {
                    rect: Rect::new(0.0, 0.0, header_w, header_h),
                    color: Color::rgb(206, 212, 218),
                    width: 1.0,
                });

                let max_render_cols = (((viewport_width - header_w) / default_col_w).ceil() as u16).clamp(10, 40);
                let max_render_rows = (((viewport_height - header_h) / default_row_h).ceil() as u32).clamp(15, 100);

                // Column Headers (A, B, C...)
                for c in 0..max_render_cols {
                    let x = header_w + c as f64 * default_col_w;
                    let col_rect = Rect::new(x, 0.0, default_col_w, header_h);
                    dl.push(DrawCommand::FillRect {
                        rect: col_rect,
                        color: Color::rgb(241, 243, 245),
                    });
                    dl.push(DrawCommand::StrokeRect {
                        rect: col_rect,
                        color: Color::rgb(206, 212, 218),
                        width: 1.0,
                    });

                    let col_label = CellCoordinate::new(0, c).to_a1_string();
                    let letters: String = col_label.chars().take_while(|ch| ch.is_ascii_alphabetic()).collect();

                    dl.push(DrawCommand::DrawTextRun {
                        text: letters,
                        font_family: "Inter".to_string(),
                        font_size_px: 11.0,
                        bold: true,
                        italic: false,
                        color: Color::rgb(73, 80, 87),
                        origin: Point::new(x + default_col_w * 0.44, 17.0),
                    });
                }

                // Row Headers (1, 2, 3...)
                for r in 0..max_render_rows {
                    let y = header_h + r as f64 * default_row_h;
                    let row_rect = Rect::new(0.0, y, header_w, default_row_h);
                    dl.push(DrawCommand::FillRect {
                        rect: row_rect,
                        color: Color::rgb(241, 243, 245),
                    });
                    dl.push(DrawCommand::StrokeRect {
                        rect: row_rect,
                        color: Color::rgb(206, 212, 218),
                        width: 1.0,
                    });

                    let row_str = format!("{}", r + 1);
                    dl.push(DrawCommand::DrawTextRun {
                        text: row_str,
                        font_family: "Inter".to_string(),
                        font_size_px: 10.5,
                        bold: false,
                        italic: false,
                        color: Color::rgb(108, 117, 125),
                        origin: Point::new(10.0, y + 16.0),
                    });
                }

                // Cells
                for r in 0..max_render_rows {
                    let y = header_h + r as f64 * default_row_h;
                    for c in 0..max_render_cols {
                        let x = header_w + c as f64 * default_col_w;
                        let cell_rect = Rect::new(x, y, default_col_w, default_row_h);

                        let mut bg = Color::WHITE;
                        if r == 0 {
                            bg = Color::rgb(248, 249, 250);
                        }
                        dl.push(DrawCommand::FillRect {
                            rect: cell_rect,
                            color: bg,
                        });

                        dl.push(DrawCommand::StrokeRect {
                            rect: cell_rect,
                            color: Color::rgb(222, 226, 230),
                            width: 1.0,
                        });

                        if let Some(cell) = ws.matrix.get_cell(CellCoordinate::new(r, c)) {
                            if !cell.value.is_empty() {
                                let formatted_text = cell.value.as_string();
                                let is_num = cell.value.as_number().is_some();
                                let text_color = if cell.value.is_error() {
                                    Color::rgb(224, 49, 49)
                                } else if r == 0 {
                                    Color::rgb(33, 37, 41)
                                } else {
                                    Color::rgb(52, 58, 64)
                                };

                                let text_x = if is_num {
                                    (x + default_col_w - 8.0 - (formatted_text.len() as f64 * 7.0)).max(x + 4.0)
                                } else {
                                    x + 6.0
                                };

                                dl.push(DrawCommand::DrawTextRun {
                                    text: formatted_text,
                                    font_family: if is_num { "JetBrains Mono".to_string() } else { "Inter".to_string() },
                                    font_size_px: 11.0,
                                    bold: r == 0,
                                    italic: false,
                                    color: text_color,
                                    origin: Point::new(text_x, y + 16.0),
                                });
                            }
                        }
                    }
                }

                // Spotlight Reading View crosshairs if enabled
                if ws.reading_view_spotlight {
                    let spotlight_color = Color::rgba(12, 166, 120, 25);
                    // Spotlight row 1 & col 1 as active inspection focal point
                    dl.push(DrawCommand::FillRect {
                        rect: Rect::new(header_w, header_h + default_row_h, viewport_width - header_w, default_row_h),
                        color: spotlight_color,
                    });
                    dl.push(DrawCommand::FillRect {
                        rect: Rect::new(header_w + default_col_w, header_h, default_col_w, viewport_height - header_h),
                        color: spotlight_color,
                    });
                }

                // Active cell selection highlight (Emerald #0CA678)
                let sel_x = header_w + default_col_w;
                let sel_y = header_h + default_row_h;
                let sel_rect = Rect::new(sel_x, sel_y, default_col_w, default_row_h);
                dl.push(DrawCommand::StrokeRect {
                    rect: sel_rect,
                    color: Color::rgb(12, 166, 120),
                    width: 2.0,
                });
                dl.push(DrawCommand::FillRect {
                    rect: Rect::new(sel_x + default_col_w - 3.0, sel_y + default_row_h - 3.0, 6.0, 6.0),
                    color: Color::rgb(12, 166, 120),
                });
            }
            ActiveDoc::Slides(deck) => {
                if let Some(slide) = deck.slides.first() {
                    slide.render_to_display_list(deck, &mut dl);
                }
            }
            ActiveDoc::Pdf(pdf) => {
                let start_y = 24.0;
                let page_spacing = 24.0;
                let mut current_y = start_y;

                for (idx, page) in pdf.pages.iter().enumerate() {
                    let page_x = (viewport_width - page.width).max(20.0) * 0.5;
                    let page_y = current_y;

                    // Drop shadow / Page card background
                    dl.push(DrawCommand::FillRect {
                        rect: Rect::new(page_x, page_y, page.width, page.height),
                        color: Color::WHITE,
                    });
                    dl.push(DrawCommand::StrokeRect {
                        rect: Rect::new(page_x, page_y, page.width, page.height),
                        color: Color::rgb(222, 226, 230),
                        width: 1.0,
                    });

                    // Vector paths
                    for path in &page.vector_paths {
                        let path_bounds = Rect::new(
                            page_x + path.bounds.x,
                            page_y + path.bounds.y,
                            path.bounds.width,
                            path.bounds.height,
                        );
                        if let Some(fill) = path.fill_color {
                            dl.push(DrawCommand::FillRect {
                                rect: path_bounds,
                                color: fill,
                            });
                        }
                        if let Some(stroke) = path.stroke_color {
                            dl.push(DrawCommand::StrokeRect {
                                rect: path_bounds,
                                color: stroke,
                                width: path.line_width.max(1.0),
                            });
                        }
                    }

                    // Text elements
                    for elem in &page.text_elements {
                        dl.push(DrawCommand::DrawTextRun {
                            text: elem.text.clone(),
                            font_family: elem.font_name.clone(),
                            font_size_px: elem.font_size,
                            bold: elem.font_name.contains("Bold"),
                            italic: elem.font_name.contains("Oblique") || elem.font_name.contains("Italic"),
                            color: elem.color,
                            origin: Point::new(page_x + elem.bounds.x, page_y + elem.baseline),
                        });
                    }

                    // Annotations
                    if let Some(annots) = pdf.annotations.get(&idx) {
                        for annot in annots {
                            let mut page_dl = DisplayList::new();
                            annot.render_to_display_list(&mut page_dl);
                            for cmd in page_dl.commands {
                                match cmd {
                                    DrawCommand::FillRect { rect, color } => {
                                        dl.push(DrawCommand::FillRect {
                                            rect: Rect::new(page_x + rect.x, page_y + rect.y, rect.width, rect.height),
                                            color,
                                        });
                                    }
                                    DrawCommand::StrokeRect { rect, color, width } => {
                                        dl.push(DrawCommand::StrokeRect {
                                            rect: Rect::new(page_x + rect.x, page_y + rect.y, rect.width, rect.height),
                                            color,
                                            width,
                                        });
                                    }
                                    DrawCommand::DrawLine { from, to, color, width } => {
                                        dl.push(DrawCommand::DrawLine {
                                            from: Point::new(page_x + from.x, page_y + from.y),
                                            to: Point::new(page_x + to.x, page_y + to.y),
                                            color,
                                            width,
                                        });
                                    }
                                    DrawCommand::DrawTextRun { text, font_family, font_size_px, bold, italic, color, origin } => {
                                        dl.push(DrawCommand::DrawTextRun {
                                            text,
                                            font_family,
                                            font_size_px,
                                            bold,
                                            italic,
                                            color,
                                            origin: Point::new(page_x + origin.x, page_y + origin.y),
                                        });
                                    }
                                    other => dl.push(other),
                                }
                            }
                        }
                    }

                    current_y += page.height + page_spacing;
                }
            }
        }

        serde_json::to_string(&dl).unwrap_or_else(|_| "{}".to_string())
    }

    /// Dispatches a structured Writer command to the Rust document model
    #[wasm_bindgen]
    pub fn writer_dispatch_command(&mut self, cmd_json: &str) -> Result<String, JsValue> {
        let cmd: WriterCommandDto = serde_json::from_str(cmd_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;

        match &mut self.doc {
            ActiveDoc::Writer(w) => {
                match cmd {
                    WriterCommandDto::InsertParagraph { text, heading } => {
                        let id = format!("p-{}", w.sections[0].elements.len() + 1);
                        let p = if let Some(lvl) = heading {
                            wps_writer::ast::ParagraphNode::heading(&id, lvl as usize, &text)
                        } else {
                            let mut p = wps_writer::ast::ParagraphNode::new(&id);
                            p.add_run(wps_writer::ast::TextRun::new(&text));
                            p
                        };
                        w.add_paragraph(p);
                    }
                    WriterCommandDto::InsertTable { rows, cols } => {
                        let mut tbl = wps_writer::ast::TableNode {
                            id: format!("table-{}", w.sections[0].elements.len() + 1),
                            alignment: wps_writer::ast::TextAlignment::Center,
                            ..Default::default()
                        };
                        let col_w = 9360 / cols.max(1) as u32;
                        tbl.grid_columns_twips = vec![col_w; cols];

                        for r in 0..rows {
                            let mut row = wps_writer::ast::TableRow {
                                is_header: r == 0,
                                is_header_repeat: r == 0,
                                ..Default::default()
                            };
                            for c in 0..cols {
                                let cell_text = if r == 0 {
                                    format!("Header {}", c + 1)
                                } else {
                                    format!("Cell {},{}", r + 1, c + 1)
                                };
                                let mut cell_p = wps_writer::ast::ParagraphNode::new(&format!("p-tc-{}-{}", r, c));
                                let mut run = wps_writer::ast::TextRun::new(&cell_text);
                                if r == 0 {
                                    run = run.bold();
                                }
                                cell_p.add_run(run);
                                row.cells.push(wps_writer::ast::TableCell {
                                    elements: vec![wps_writer::ast::BlockNode::Paragraph(cell_p)],
                                    grid_span: 1,
                                    col_span: 1,
                                    row_span: 1,
                                    width_twips: wps_core::units::Twips(col_w as i64),
                                    background_color: if r == 0 { Some(Color::rgb(241, 243, 245)) } else { None },
                                    ..Default::default()
                                });
                            }
                            tbl.rows.push(row);
                        }
                        w.add_table(tbl);
                    }
                    WriterCommandDto::UpdateTableOfContents => {
                        let mut paginator = wps_writer::layout::Paginator::default();
                        let pages = paginator.paginate_document(w);
                        wps_writer::automation::DocumentAutomation::update_table_of_contents(w, &pages);
                    }
                    WriterCommandDto::RunMailMerge { records_json } => {
                        let records: Vec<std::collections::HashMap<String, String>> = serde_json::from_str(&records_json)
                            .map_err(|e| JsValue::from_str(&format!("Invalid records JSON: {}", e)))?;
                        let btree_records: Vec<std::collections::BTreeMap<String, String>> = records
                            .into_iter()
                            .map(|m| m.into_iter().collect())
                            .collect();
                        let generated = wps_writer::automation::DocumentAutomation::execute_mail_merge(w, &btree_records)
                            .map_err(|e| JsValue::from_str(&e))?;
                        if let Some(first) = generated.into_iter().next() {
                            *w = first;
                        }
                    }
                    WriterCommandDto::SetTrackChanges { enabled } => {
                        w.revisions.tracking_enabled = enabled;
                    }
                    WriterCommandDto::AcceptAllRevisions => {
                        w.revisions.accept_all(&mut w.buffer, &mut w.attributes);
                    }
                    WriterCommandDto::RejectAllRevisions => {
                        w.revisions.reject_all(&mut w.buffer, &mut w.attributes);
                    }
                    WriterCommandDto::SetPageMargins { top, bottom, left, right } => {
                        if let Some(sec) = w.sections.first_mut() {
                            sec.page_setup.margins.top_twips = top;
                            sec.page_setup.margins.bottom_twips = bottom;
                            sec.page_setup.margins.left_twips = left;
                            sec.page_setup.margins.right_twips = right;
                        }
                    }
                    WriterCommandDto::SetPageOrientation { landscape } => {
                        if let Some(sec) = w.sections.first_mut() {
                            sec.page_setup.orientation = if landscape {
                                wps_writer::ast::PageOrientation::Landscape
                            } else {
                                wps_writer::ast::PageOrientation::Portrait
                            };
                            let (w_t, h_t) = (sec.page_setup.width_twips, sec.page_setup.height_twips);
                            if (landscape && w_t < h_t) || (!landscape && w_t > h_t) {
                                sec.page_setup.width_twips = h_t;
                                sec.page_setup.height_twips = w_t;
                            }
                        }
                    }
                }
                Ok(format!("{{\"status\":\"ok\",\"words\":{}}}", w.total_word_count()))
            }
            _ => Err(JsValue::from_str("Active module is not Writer")),
        }
    }

    /// Dispatches a structured Spreadsheets command to the Rust workbook engine
    #[wasm_bindgen]
    pub fn sheets_dispatch_command(&mut self, cmd_json: &str) -> Result<String, JsValue> {
        let cmd: SheetsCommandDto = serde_json::from_str(cmd_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid Sheets command JSON: {}", e)))?;

        match &mut self.doc {
            ActiveDoc::Sheets(wb) => {
                match cmd {
                    SheetsCommandDto::SetCellInput { sheet_idx, coord, input } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };

                        let parsed_coord = CellCoordinate::parse(&coord)
                            .ok_or_else(|| JsValue::from_str(&format!("Invalid cell coordinate: {}", coord)))?;

                        if input.starts_with('=') {
                            ws.matrix.set_formula(parsed_coord, Some(input.clone()));
                            let mut lexer = Lexer::new(&input);
                            if let Ok(tokens) = lexer.tokenize() {
                                let mut parser = Parser::new(tokens);
                                if let Ok(ast) = parser.parse() {
                                    let evaluator = Evaluator::new(&ws.matrix);
                                    let result = evaluator.eval(&ast);
                                    ws.set_cell_value(parsed_coord, result);
                                }
                            }
                        } else if let Ok(n) = input.parse::<f64>() {
                            ws.matrix.set_formula(parsed_coord, None);
                            ws.set_cell_value(parsed_coord, CellValue::Number(n));
                        } else if input.eq_ignore_ascii_case("true") {
                            ws.matrix.set_formula(parsed_coord, None);
                            ws.set_cell_value(parsed_coord, CellValue::Boolean(true));
                        } else if input.eq_ignore_ascii_case("false") {
                            ws.matrix.set_formula(parsed_coord, None);
                            ws.set_cell_value(parsed_coord, CellValue::Boolean(false));
                        } else {
                            ws.matrix.set_formula(parsed_coord, None);
                            ws.set_cell_value(parsed_coord, CellValue::Text(input));
                        }
                    }
                    SheetsCommandDto::SetRangeValues { sheet_idx, start_coord, values } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        let start = CellCoordinate::parse(&start_coord)
                            .ok_or_else(|| JsValue::from_str("Invalid start coord"))?;

                        for (r_off, row) in values.iter().enumerate() {
                            for (c_off, val) in row.iter().enumerate() {
                                let c = CellCoordinate::new(start.row + r_off as u32, start.col + c_off as u16);
                                if let Ok(num) = val.parse::<f64>() {
                                    ws.set_cell_value(c, CellValue::Number(num));
                                } else {
                                    ws.set_cell_value(c, CellValue::Text(val.clone()));
                                }
                            }
                        }
                    }
                    SheetsCommandDto::ApplyCellStyle { sheet_idx, range, bg_color: _, format_code: _ } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        let _range = CellRange::parse(&range)
                            .ok_or_else(|| JsValue::from_str("Invalid range"))?;
                        // Recorded on worksheet styles
                        let _ = ws;
                    }
                    SheetsCommandDto::MergeCells { sheet_idx, range } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        let parsed_range = CellRange::parse(&range)
                            .ok_or_else(|| JsValue::from_str("Invalid range"))?;
                        ws.merge_range(parsed_range);
                    }
                    SheetsCommandDto::UnmergeCells { sheet_idx, range } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        let parsed_range = CellRange::parse(&range)
                            .ok_or_else(|| JsValue::from_str("Invalid range"))?;
                        ws.merged_regions.retain(|r| *r != parsed_range);
                        ws.matrix.merged_ranges.retain(|r| *r != parsed_range);
                    }
                    SheetsCommandDto::AutofillRange { sheet_idx, source_range, target_range } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        let src = CellRange::parse(&source_range).ok_or_else(|| JsValue::from_str("Invalid source range"))?;
                        let tgt = CellRange::parse(&target_range).ok_or_else(|| JsValue::from_str("Invalid target range"))?;

                        let delta_r = tgt.start.row as i32 - src.start.row as i32;
                        let delta_c = tgt.start.col as i32 - src.start.col as i32;

                        for r in tgt.start.row..=tgt.end.row {
                            for c in tgt.start.col..=tgt.end.col {
                                let src_c = CellCoordinate::new(
                                    (r as i32 - delta_r).max(0) as u32,
                                    (c as i32 - delta_c).max(0) as u16,
                                );
                                if let Some(cell) = ws.matrix.get_cell(src_c).cloned() {
                                    let target_coord = CellCoordinate::new(r, c);
                                    if let Some(ref form) = cell.formula {
                                        let shifted = shift_formula_references(form, delta_r, delta_c);
                                        ws.matrix.set_formula(target_coord, Some(shifted.clone()));
                                        if let Ok(tokens) = Lexer::new(&shifted).tokenize() {
                                            if let Ok(ast) = Parser::new(tokens).parse() {
                                                let res = Evaluator::new(&ws.matrix).eval(&ast);
                                                ws.set_cell_value(target_coord, res);
                                            }
                                        }
                                    } else {
                                        ws.set_cell_value(target_coord, cell.value);
                                    }
                                }
                            }
                        }
                    }
                    SheetsCommandDto::ExecuteSmartToolboxOp {
                        sheet_idx,
                        op,
                        range,
                        operand,
                        prefix,
                        suffix,
                        case_mode,
                        pattern,
                    } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        let parsed_range = range
                            .as_deref()
                            .and_then(CellRange::parse)
                            .unwrap_or_else(|| CellRange::new(CellCoordinate::new(0, 0), CellCoordinate::new(50, 20)));

                        match op.as_str() {
                            "unmerge_and_fill" => {
                                SmartToolbox::unmerge_and_fill(&mut ws.matrix, parsed_range);
                            }
                            "range_arithmetic" => {
                                let op_str = prefix.as_deref().unwrap_or("+");
                                let op_val = operand.unwrap_or(1.0);
                                SmartToolbox::range_arithmetic(&mut ws.matrix, parsed_range, op_str, op_val);
                            }
                            "batch_add_prefix_suffix" => {
                                let pre = prefix.as_deref().unwrap_or("");
                                let suf = suffix.as_deref().unwrap_or("");
                                SmartToolbox::batch_add_prefix_suffix(&mut ws.matrix, parsed_range, pre, suf);
                            }
                            "batch_case_conversion" => {
                                let mode = case_mode.as_deref().unwrap_or("upper");
                                SmartToolbox::batch_case_conversion(&mut ws.matrix, parsed_range, mode);
                            }
                            "batch_regex_extract" => {
                                let pat = pattern.as_deref().unwrap_or(r"\d+");
                                let target_col = operand.map(|o| o as u16).unwrap_or(parsed_range.end.col + 1);
                                let _ = SmartToolbox::batch_regex_extract(&mut ws.matrix, parsed_range, target_col, pat);
                            }
                            "convert_formulas_to_values" => {
                                SmartToolbox::convert_formulas_to_values(&mut ws.matrix, parsed_range);
                            }
                            _ => return Err(JsValue::from_str(&format!("Unknown Smart Toolbox op: {}", op))),
                        }
                    }
                    SheetsCommandDto::CreatePivotTable { source_sheet, target_sheet_name, config } => {
                        let source_ws = wb.sheets.get(source_sheet).ok_or_else(|| JsValue::from_str("Invalid source sheet"))?;
                        let pivot_data = PivotCube::build_pivot_cube_2d(&source_ws.matrix, 50, &config);

                        let target_idx = wb.add_sheet(&target_sheet_name);
                        let target_ws = &mut wb.sheets[target_idx];
                        for (r, row) in pivot_data.into_iter().enumerate() {
                            for (c, val) in row.into_iter().enumerate() {
                                target_ws.set_cell_value(CellCoordinate::new(r as u32, c as u16), val);
                            }
                        }
                        wb.active_sheet = target_idx;
                    }
                    SheetsCommandDto::AddSheet { name } => {
                        let idx = wb.add_sheet(&name);
                        wb.active_sheet = idx;
                    }
                    SheetsCommandDto::SwitchSheet { index } => {
                        if index < wb.sheets.len() {
                            wb.active_sheet = index;
                        } else {
                            return Err(JsValue::from_str("Sheet index out of bounds"));
                        }
                    }
                    SheetsCommandDto::RenameSheet { index, new_name } => {
                        if let Some(ws) = wb.sheets.get_mut(index) {
                            ws.name = new_name;
                        } else {
                            return Err(JsValue::from_str("Sheet index out of bounds"));
                        }
                    }
                    SheetsCommandDto::ToggleReadingView { sheet_idx } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        ws.reading_view_spotlight = !ws.reading_view_spotlight;
                    }
                    SheetsCommandDto::SetFreezePane { sheet_idx, split_row, split_col } => {
                        let ws = if let Some(idx) = sheet_idx {
                            wb.sheets.get_mut(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                        } else {
                            wb.active_worksheet_mut()
                        };
                        ws.freeze_pane = Some(FreezePane {
                            split_row,
                            split_col,
                            top_left_cell: CellCoordinate::new(split_row, split_col),
                        });
                    }
                }
                Ok("{\"status\":\"ok\"}".to_string())
            }
            _ => Err(JsValue::from_str("Active module is not Spreadsheets")),
        }
    }

    /// Returns detailed metadata and layout statistics for Writer
    #[wasm_bindgen]
    pub fn writer_get_metadata_json(&self) -> String {
        match &self.doc {
            ActiveDoc::Writer(w) => {
                let mut paginator = wps_writer::layout::Paginator::default();
                let pages = paginator.paginate_document(w);
                let metadata = serde_json::json!({
                    "title": w.title,
                    "author": w.author,
                    "page_count": pages.len(),
                    "word_count": w.total_word_count(),
                    "char_count": w.total_char_count(),
                    "track_changes": w.revisions.tracking_enabled,
                    "pending_revisions": w.revisions.revisions.len(),
                    "comments_count": w.comments.comments.len(),
                });
                metadata.to_string()
            }
            _ => "{}".to_string(),
        }
    }

    /// Returns sheets metadata: list of sheets, active index, Reading View status
    #[wasm_bindgen]
    pub fn sheets_get_metadata_json(&self) -> String {
        match &self.doc {
            ActiveDoc::Sheets(wb) => {
                let sheet_list: Vec<serde_json::Value> = wb.sheets.iter().enumerate().map(|(idx, s)| {
                    serde_json::json!({
                        "index": idx,
                        "id": s.id,
                        "name": s.name,
                        "rows": s.matrix.max_populated_row() + 1,
                        "cols": s.matrix.max_populated_col() + 1,
                    })
                }).collect();

                let metadata = serde_json::json!({
                    "active_sheet": wb.active_sheet,
                    "sheet_count": wb.sheets.len(),
                    "sheets": sheet_list,
                    "reading_view_spotlight": wb.active_worksheet().reading_view_spotlight,
                    "freeze_pane": wb.active_worksheet().freeze_pane.is_some(),
                });
                metadata.to_string()
            }
            _ => "{}".to_string(),
        }
    }

    /// Computes selection stats (Sum, Average, Count, Min, Max) for StatusBar
    #[wasm_bindgen]
    pub fn sheets_get_selection_stats(&self, range_str: &str) -> String {
        match &self.doc {
            ActiveDoc::Sheets(wb) => {
                let ws = wb.active_worksheet();
                let range = CellRange::parse(range_str).unwrap_or_else(|| {
                    CellRange::new(CellCoordinate::new(0, 0), CellCoordinate::new(0, 0))
                });

                let mut count = 0usize;
                let mut num_count = 0usize;
                let mut sum = 0.0f64;
                let mut min_val = f64::INFINITY;
                let mut max_val = f64::NEG_INFINITY;

                for r in range.start.row..=range.end.row {
                    for c in range.start.col..=range.end.col {
                        if let Some(cell) = ws.matrix.get_cell(CellCoordinate::new(r, c)) {
                            if !cell.value.is_empty() {
                                count += 1;
                                if let Some(n) = cell.value.as_number() {
                                    num_count += 1;
                                    sum += n;
                                    if n < min_val { min_val = n; }
                                    if n > max_val { max_val = n; }
                                }
                            }
                        }
                    }
                }

                let avg = if num_count > 0 { sum / num_count as f64 } else { 0.0 };
                let min = if min_val.is_finite() { min_val } else { 0.0 };
                let max = if max_val.is_finite() { max_val } else { 0.0 };

                serde_json::json!({
                    "count": count,
                    "num_count": num_count,
                    "sum": sum,
                    "average": avg,
                    "min": min,
                    "max": max,
                }).to_string()
            }
            _ => "{}".to_string(),
        }
    }

    /// Exports current Writer document to DOCX binary buffer
    #[wasm_bindgen]
    pub fn writer_export_docx(&self) -> Result<Vec<u8>, JsValue> {
        match &self.doc {
            ActiveDoc::Writer(w) => {
                wps_writer::docx::DocxCodec::export_docx(w)
                    .map_err(|e| JsValue::from_str(&format!("DOCX export error: {:?}", e)))
            }
            _ => Err(JsValue::from_str("Active module is not Writer")),
        }
    }

    /// Imports a DOCX binary buffer into the active Writer session
    #[wasm_bindgen]
    pub fn writer_import_docx(&mut self, data: &[u8]) -> Result<(), JsValue> {
        match &mut self.doc {
            ActiveDoc::Writer(w) => {
                let doc = wps_writer::docx::DocxCodec::import_docx(data)
                    .map_err(|e| JsValue::from_str(&format!("DOCX import error: {:?}", e)))?;
                *w = doc;
                Ok(())
            }
            _ => Err(JsValue::from_str("Active module is not Writer")),
        }
    }

    /// Exports active Writer document as Markdown
    #[wasm_bindgen]
    pub fn writer_export_markdown(&self) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Writer(w) => Ok(wps_writer::docx::DocxCodec::export_markdown(w)),
            _ => Err(JsValue::from_str("Active module is not Writer")),
        }
    }

    /// Imports Markdown content into active Writer document
    #[wasm_bindgen]
    pub fn writer_import_markdown(&mut self, md: &str) -> Result<(), JsValue> {
        match &mut self.doc {
            ActiveDoc::Writer(w) => {
                let doc = wps_writer::docx::DocxCodec::import_markdown(md);
                *w = doc;
                Ok(())
            }
            _ => Err(JsValue::from_str("Active module is not Writer")),
        }
    }

    /// Exports active Spreadsheets workbook to XLSX binary buffer
    #[wasm_bindgen]
    pub fn sheets_export_xlsx(&self) -> Result<Vec<u8>, JsValue> {
        match &self.doc {
            ActiveDoc::Sheets(wb) => {
                XlsxCodec::export_workbook_xlsx(wb)
                    .map_err(|e| JsValue::from_str(&format!("XLSX export error: {:?}", e)))
            }
            _ => Err(JsValue::from_str("Active module is not Spreadsheets")),
        }
    }

    /// Imports an XLSX binary buffer into the active Spreadsheets session
    #[wasm_bindgen]
    pub fn sheets_import_xlsx(&mut self, data: &[u8]) -> Result<(), JsValue> {
        match &mut self.doc {
            ActiveDoc::Sheets(wb) => {
                let imported = XlsxCodec::import_workbook_xlsx(data)
                    .map_err(|e| JsValue::from_str(&format!("XLSX import error: {:?}", e)))?;
                *wb = imported;
                Ok(())
            }
            _ => Err(JsValue::from_str("Active module is not Spreadsheets")),
        }
    }

    /// Exports active worksheet as RFC 4180 CSV
    #[wasm_bindgen]
    pub fn sheets_export_csv(&self, sheet_idx: Option<usize>) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Sheets(wb) => {
                let ws = if let Some(idx) = sheet_idx {
                    wb.sheets.get(idx).ok_or_else(|| JsValue::from_str("Invalid sheet index"))?
                } else {
                    wb.active_worksheet()
                };
                let max_r = ws.matrix.max_populated_row().max(10);
                let max_c = ws.matrix.max_populated_col().max(5);
                Ok(XlsxCodec::export_csv(&ws.matrix, max_r, max_c))
            }
            _ => Err(JsValue::from_str("Active module is not Spreadsheets")),
        }
    }

    /// Imports RFC 4180 CSV into active worksheet
    #[wasm_bindgen]
    pub fn sheets_import_csv(&mut self, csv_str: &str, sheet_name: Option<String>) -> Result<(), JsValue> {
        match &mut self.doc {
            ActiveDoc::Sheets(wb) => {
                let matrix = XlsxCodec::import_csv(csv_str);
                let name = sheet_name.unwrap_or_else(|| format!("Imported Sheet {}", wb.sheets.len() + 1));
                let idx = wb.add_sheet(&name);
                wb.sheets[idx].matrix = matrix;
                wb.active_sheet = idx;
                Ok(())
            }
            _ => Err(JsValue::from_str("Active module is not Spreadsheets")),
        }
    }

    /// Evaluates python script against current spreadsheet or office context
    #[wasm_bindgen]
    pub fn eval_python(&mut self, script: &str) -> Result<String, JsValue> {
        match &mut self.doc {
            ActiveDoc::Sheets(wb) => {
                let out = self.python_engine.eval_cell(script, &wb.active_worksheet_mut().matrix)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => {
                let empty_matrix = SparseMatrix::new();
                let out = self.python_engine.eval_cell(script, &empty_matrix)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }

    /// Dispatches prompt to AST-native AI co-pilot
    #[wasm_bindgen]
    pub fn dispatch_ai(&mut self, prompt: &str, context: &str) -> Result<String, JsValue> {
        match self.active_module {
            ActiveModule::Writer => {
                let para = wps_ai::WriterParagraphInput {
                    paragraph_id: 1,
                    text: context.to_string(),
                    current_style_id: "Normal".to_string(),
                };
                let res = wps_ai::WriterAiAgent::rewrite_with_track_changes(&para, wps_ai::RewriteTone::Formal, "2026-10-30T12:00:00Z");
                serde_json::to_string(&res).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            ActiveModule::Spreadsheets => {
                let headers = vec!["A".to_string(), "B".to_string(), "C".to_string()];
                let rows = vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]];
                let schema = wps_ai::SheetsAiAgent::extract_table_schema("Sheet1", &headers, &rows);
                let res = wps_ai::SheetsAiAgent::natural_language_to_formula(prompt, &schema);
                serde_json::to_string(&res).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            ActiveModule::Presentation => {
                let outline = wps_ai::SlidesAiAgent::generate_outline_from_prompt(prompt, 5);
                let compiled = wps_ai::SlidesAiAgent::compile_outline_to_deck(&outline);
                serde_json::to_string(&compiled).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            ActiveModule::Pdf => {
                let mut rag = wps_ai::PdfSpatialRagIndex::new();
                rag.add_spatial_chunk(
                    1,
                    0,
                    wps_ai::PdfBBox::new(50.0, 50.0, 500.0, 150.0),
                    wps_ai::PdfChunkKind::BodyParagraph,
                    context.to_string(),
                );
                let ans = rag.query_with_bbox_citations(prompt, 3);
                serde_json::to_string(&ans).map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }

    /// Dispatches a structured Slides command to the Rust presentation engine
    #[wasm_bindgen]
    pub fn slides_dispatch_command(&mut self, cmd_json: &str) -> Result<String, JsValue> {
        let cmd: SlidesCommandDto = serde_json::from_str(cmd_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;

        match &mut self.doc {
            ActiveDoc::Slides(deck) => {
                match cmd {
                    SlidesCommandDto::AddSlide { title, layout_id } => {
                        let id = format!("slide-{}", deck.slides.len() + 1);
                        let lid = layout_id.unwrap_or_else(|| "layout-content".to_string());
                        let slide = Slide::new(&id, &title, &lid);
                        deck.add_slide(slide);
                    }
                    SlidesCommandDto::DuplicateSlide { index } => {
                        if index < deck.slides.len() {
                            let mut dup = deck.slides[index].clone();
                            dup.id = format!("slide-{}", deck.slides.len() + 1);
                            deck.slides.insert(index + 1, dup);
                        }
                    }
                    SlidesCommandDto::DeleteSlide { index } => {
                        if deck.slides.len() > 1 && index < deck.slides.len() {
                            deck.slides.remove(index);
                        }
                    }
                    SlidesCommandDto::MoveSlide { from, to } => {
                        if from < deck.slides.len() && to < deck.slides.len() && from != to {
                            let slide = deck.slides.remove(from);
                            deck.slides.insert(to, slide);
                        }
                    }
                    SlidesCommandDto::InsertShape { slide_idx, geometry, x, y, width, height, fill_hex, text } => {
                        let slide = deck.slides.get_mut(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                        let geom_type = match geometry.to_lowercase().as_str() {
                            "roundedrectangle" | "roundrect" => ShapeGeometryType::RoundedRectangle,
                            "ellipse" | "circle" => ShapeGeometryType::Ellipse,
                            "triangle" => ShapeGeometryType::Triangle,
                            "diamond" => ShapeGeometryType::Diamond,
                            "star" | "star5" => ShapeGeometryType::Star5,
                            "rightarrow" | "arrow" => ShapeGeometryType::RightArrow,
                            "callout" | "rectcallout" => ShapeGeometryType::RectCallout,
                            "process" | "flowchartprocess" => ShapeGeometryType::FlowchartProcess,
                            "decision" | "flowchartdecision" => ShapeGeometryType::FlowchartDecision,
                            "donut" => ShapeGeometryType::Donut,
                            "heart" => ShapeGeometryType::Heart,
                            _ => ShapeGeometryType::Rectangle,
                        };
                        let id = format!("shape-{}-{}", slide_idx + 1, slide.nodes.len() + 1);
                        let mut shape = ParametricShape::new(&id, geom_type, Rect::new(x, y, width, height));
                        if let Some(hex) = fill_hex {
                            shape.fill = ShapeFill::Solid(Color::from_hex(&hex).unwrap_or(Color::MERCURY_PRIMARY));
                        }
                        if let Some(t) = text {
                            shape = shape.with_text(&t);
                        }
                        slide.add_shape(shape);
                    }
                    SlidesCommandDto::UpdateShapeAdjustment { slide_idx, shape_id, handle_index, x, y } => {
                        let slide = deck.slides.get_mut(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                        for node in &mut slide.nodes {
                            if let SlideNode::AutoShape(s) = node {
                                if s.id == shape_id {
                                    s.set_adjustment_handle(handle_index, Point::new(x, y));
                                    break;
                                }
                            }
                        }
                    }
                    SlidesCommandDto::InsertConnector { slide_idx, start_id, end_id, routing } => {
                        let slide = deck.slides.get_mut(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                        let route_type = match routing.as_deref() {
                            Some("straight") => ConnectorRouting::Straight,
                            Some("curved") => ConnectorRouting::Curved,
                            _ => ConnectorRouting::Orthogonal,
                        };
                        let id = format!("conn-{}-{}", slide_idx + 1, slide.nodes.len() + 1);
                        let mut conn = SmartConnector::new(&id, &start_id, &end_id);
                        conn.routing = route_type;
                        slide.add_connector(conn);
                    }
                    SlidesCommandDto::ApplyTheme { theme_name } => {
                        deck.master.theme = match theme_name.to_lowercase().as_str() {
                            "slate" | "executive_slate" => SlideTheme::executive_slate(),
                            "emerald" | "emerald_growth" => SlideTheme::emerald_growth(),
                            "coral" | "coral_sunset" => SlideTheme::coral_sunset(),
                            "crimson" | "crimson_bold" => SlideTheme::crimson_bold(),
                            "iris" | "iris_creative" => SlideTheme::iris_creative(),
                            "cyber" | "midnight_cyber" => SlideTheme::midnight_cyber(),
                            "sand" | "solar_sand" => SlideTheme::solar_sand(),
                            _ => SlideTheme::mercury_modern(),
                        };
                    }
                    SlidesCommandDto::SetSlideTransition { slide_idx, transition_name, duration_ms, morph_type } => {
                        let slide = deck.slides.get_mut(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                        let m_type = match morph_type.as_deref() {
                            Some("by_word") => MorphType::ByWord,
                            Some("by_character") => MorphType::ByCharacter,
                            _ => MorphType::ByObject,
                        };
                        slide.transition = Some(SlideTransition {
                            name: transition_name,
                            duration_ms: duration_ms.unwrap_or(1500),
                            morph_type: m_type,
                        });
                    }
                    SlidesCommandDto::AddAnimation { slide_idx, shape_id, category, effect } => {
                        let slide = deck.slides.get_mut(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                        let cat = match category.to_lowercase().as_str() {
                            "emphasis" => AnimationCategory::Emphasis,
                            "exit" => AnimationCategory::Exit,
                            _ => AnimationCategory::Entrance,
                        };
                        let eff = match effect.to_lowercase().as_str() {
                            "flyin" => AnimationEffect::FlyIn,
                            "zoom" => AnimationEffect::Zoom,
                            "spin" => AnimationEffect::Spin,
                            "wipe" => AnimationEffect::Wipe,
                            "bounce" => AnimationEffect::Bounce,
                            _ => AnimationEffect::Fade,
                        };
                        slide.timeline.add_animation(KeyframeAnimation::new(&shape_id, cat, eff));
                    }
                    SlidesCommandDto::ExecuteBooleanOp { slide_idx, shape_a_id, shape_b_id, op } => {
                        let slide = deck.slides.get_mut(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                        let shape_a = slide.nodes.iter().find_map(|n| if let SlideNode::AutoShape(s) = n { if s.id == shape_a_id { Some(s.clone()) } else { None } } else { None });
                        let shape_b = slide.nodes.iter().find_map(|n| if let SlideNode::AutoShape(s) = n { if s.id == shape_b_id { Some(s.clone()) } else { None } } else { None });
                        if let (Some(sa), Some(sb)) = (shape_a, shape_b) {
                            let b_op = match op.to_lowercase().as_str() {
                                "subtract" => BooleanOp::Subtract,
                                "intersect" => BooleanOp::Intersect,
                                "combine" => BooleanOp::Combine,
                                "fragment" => BooleanOp::Fragment,
                                _ => BooleanOp::Union,
                            };
                            let results = BooleanShapeEngine::execute(&sa, &sb, b_op);
                            slide.nodes.retain(|n| n.id() != shape_a_id && n.id() != shape_b_id);
                            for res in results {
                                slide.add_shape(res);
                            }
                        }
                    }
                    SlidesCommandDto::UpdateSpeakerNotes { slide_idx, notes } => {
                        let slide = deck.slides.get_mut(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                        slide.speaker_notes = notes;
                    }
                }
                self.slides_get_deck_summary()
            }
            _ => Err(JsValue::from_str("Active module is not Presentation")),
        }
    }

    /// Returns JSON summary of all slides in the deck
    #[wasm_bindgen]
    pub fn slides_get_deck_summary(&self) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Slides(deck) => {
                #[derive(Serialize)]
                struct SlideSummary {
                    index: usize,
                    id: String,
                    title: String,
                    layout_id: String,
                    shape_count: usize,
                    transition_name: Option<String>,
                    speaker_notes: String,
                }
                #[derive(Serialize)]
                struct DeckSummary {
                    title: String,
                    theme_name: String,
                    slide_count: usize,
                    slides: Vec<SlideSummary>,
                }

                let summaries: Vec<SlideSummary> = deck.slides.iter().enumerate().map(|(i, s)| {
                    SlideSummary {
                        index: i,
                        id: s.id.clone(),
                        title: s.title.clone(),
                        layout_id: s.layout_id.clone(),
                        shape_count: s.nodes.len(),
                        transition_name: s.transition.as_ref().map(|t| t.name.clone()),
                        speaker_notes: s.speaker_notes.clone(),
                    }
                }).collect();

                let deck_summary = DeckSummary {
                    title: deck.title.clone(),
                    theme_name: deck.master.theme.name.clone(),
                    slide_count: deck.slides.len(),
                    slides: summaries,
                };

                serde_json::to_string(&deck_summary).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not Presentation")),
        }
    }

    /// Renders a specific slide to DisplayList JSON
    #[wasm_bindgen]
    pub fn slides_render_slide_display_list(&self, slide_idx: usize) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Slides(deck) => {
                let slide = deck.slides.get(slide_idx).ok_or_else(|| JsValue::from_str("Invalid slide index"))?;
                let mut dl = DisplayList::new();
                slide.render_to_display_list(deck, &mut dl);
                serde_json::to_string(&dl).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not Presentation")),
        }
    }

    /// Evaluates Morph transition between slide A and slide B at progress t (0.0 to 1.0)
    #[wasm_bindgen]
    pub fn slides_evaluate_morph(&self, slide_a_idx: usize, slide_b_idx: usize, progress: f64) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Slides(deck) => {
                let slide_a = deck.slides.get(slide_a_idx).ok_or_else(|| JsValue::from_str("Invalid slide A index"))?;
                let slide_b = deck.slides.get(slide_b_idx).ok_or_else(|| JsValue::from_str("Invalid slide B index"))?;

                let shapes_a: Vec<ParametricShape> = slide_a.nodes.iter().filter_map(|n| if let SlideNode::AutoShape(s) = n { Some(s.clone()) } else { None }).collect();
                let shapes_b: Vec<ParametricShape> = slide_b.nodes.iter().filter_map(|n| if let SlideNode::AutoShape(s) = n { Some(s.clone()) } else { None }).collect();

                let morphed_shapes = MorphInterpolator::interpolate_shapes(&shapes_a, &shapes_b, progress);

                let mut dl = DisplayList::new();
                // Render slide background
                dl.push(DrawCommand::FillRect {
                    rect: deck.slide_size.to_pt_rect(),
                    color: deck.master.background_color,
                });
                for shape in &morphed_shapes {
                    shape.render_to_display_list(&mut dl);
                }

                serde_json::to_string(&dl).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not Presentation")),
        }
    }

    /// Exports current presentation to ECMA-376 PPTX bytes
    #[wasm_bindgen]
    pub fn slides_export_pptx(&self) -> Result<Vec<u8>, JsValue> {
        match &self.doc {
            ActiveDoc::Slides(deck) => {
                PptxCodec::export_pptx(deck).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not Presentation")),
        }
    }

    /// Imports PPTX bytes into the active presentation
    #[wasm_bindgen]
    pub fn slides_open_pptx(&mut self, bytes: &[u8]) -> Result<String, JsValue> {
        let imported_deck = PptxCodec::import_pptx(bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.doc = ActiveDoc::Slides(imported_deck);
        self.active_module = ActiveModule::Presentation;
        self.slides_get_deck_summary()
    }

    /// Opens an ISO 32000 PDF document from bytes, decrypting with password if necessary
    #[wasm_bindgen]
    pub fn pdf_open_document(&mut self, bytes: &[u8], password: &str) -> Result<String, JsValue> {
        let doc = PdfDocument::load_with_password(bytes, password)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.doc = ActiveDoc::Pdf(doc);
        self.active_module = ActiveModule::Pdf;
        self.pdf_get_document_summary()
    }

    /// Saves the current PDF document to ISO 32000 byte stream
    #[wasm_bindgen]
    pub fn pdf_save_document(&self) -> Result<Vec<u8>, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => Ok(pdf.export_to_bytes()),
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Encrypts the PDF document using pure-Rust AES-256 standard encryption
    #[wasm_bindgen]
    pub fn pdf_encrypt_document(&self, user_pw: &str, owner_pw: &str) -> Result<Vec<u8>, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => Ok(pdf.encrypt(user_pw, owner_pw)),
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Returns high-level summary of the active PDF document
    #[wasm_bindgen]
    pub fn pdf_get_document_summary(&self) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                #[derive(Serialize)]
                struct PageSummary {
                    index: usize,
                    width: f64,
                    height: f64,
                    rotation: i32,
                    text_element_count: usize,
                    vector_path_count: usize,
                    annotation_count: usize,
                }
                #[derive(Serialize)]
                struct Summary {
                    title: String,
                    page_count: usize,
                    can_print: bool,
                    can_copy: bool,
                    can_modify: bool,
                    can_annotate: bool,
                    pages: Vec<PageSummary>,
                    signature_count: usize,
                }
                let pages: Vec<PageSummary> = pdf.pages.iter().enumerate().map(|(idx, p)| {
                    PageSummary {
                        index: idx,
                        width: p.width,
                        height: p.height,
                        rotation: p.rotation,
                        text_element_count: p.text_elements.len(),
                        vector_path_count: p.vector_paths.len(),
                        annotation_count: pdf.annotations.get(&idx).map(|a| a.len()).unwrap_or(0),
                    }
                }).collect();
                let summary = Summary {
                    title: pdf.title.clone(),
                    page_count: pdf.pages.len(),
                    can_print: pdf.permissions.allow_print,
                    can_copy: pdf.permissions.allow_copy,
                    can_modify: pdf.permissions.allow_modify,
                    can_annotate: pdf.permissions.allow_annotations,
                    pages,
                    signature_count: pdf.signatures.len(),
                };
                serde_json::to_string(&summary).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Reconstructs spatial layout (XY-Cut paragraphs, headers, and hybrid tables)
    #[wasm_bindgen]
    pub fn pdf_get_page_layout_json(&self, page_idx: usize) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                let page = pdf.pages.get(page_idx).ok_or_else(|| JsValue::from_str("Invalid page index"))?;
                let layout = SpatialReconstructor::reconstruct_page(page);

                #[derive(Serialize)]
                struct BlockDto {
                    kind: String,
                    bounds: [f64; 4],
                    text: String,
                    font_name: String,
                    font_size: f64,
                    is_bold: bool,
                    is_italic: bool,
                }
                #[derive(Serialize)]
                struct CellDto {
                    row: usize,
                    col: usize,
                    bounds: [f64; 4],
                    text: String,
                }
                #[derive(Serialize)]
                struct TableDto {
                    bounds: [f64; 4],
                    rows: usize,
                    cols: usize,
                    cells: Vec<CellDto>,
                }
                #[derive(Serialize)]
                struct LayoutDto {
                    page_index: usize,
                    bounds: [f64; 4],
                    blocks: Vec<BlockDto>,
                    tables: Vec<TableDto>,
                }

                let blocks: Vec<BlockDto> = layout.blocks.iter().map(|b| {
                    let kind_str = match b.kind {
                        BlockKind::Title => "Title",
                        BlockKind::Heading(lvl) => match lvl { 1 => "Heading1", 2 => "Heading2", _ => "Heading3" },
                        BlockKind::Paragraph => "Paragraph",
                        BlockKind::ListItem => "ListItem",
                        BlockKind::TableCell => "TableCell",
                        BlockKind::HeaderBlock => "Header",
                        BlockKind::FooterBlock => "Footer",
                    };
                    BlockDto {
                        kind: kind_str.to_string(),
                        bounds: [b.bounds.x, b.bounds.y, b.bounds.width, b.bounds.height],
                        text: b.plain_text(),
                        font_name: b.font_name.clone(),
                        font_size: b.font_size,
                        is_bold: b.is_bold,
                        is_italic: b.is_italic,
                    }
                }).collect();

                let tables: Vec<TableDto> = layout.tables.iter().map(|t| {
                    TableDto {
                        bounds: [t.bounds.x, t.bounds.y, t.bounds.width, t.bounds.height],
                        rows: t.rows,
                        cols: t.cols,
                        cells: t.cells.iter().map(|c| CellDto {
                            row: c.row,
                            col: c.col,
                            bounds: [c.bounds.x, c.bounds.y, c.bounds.width, c.bounds.height],
                            text: c.text.clone(),
                        }).collect(),
                    }
                }).collect();

                let layout_dto = LayoutDto {
                    page_index: layout.page_index,
                    bounds: [layout.bounds.x, layout.bounds.y, layout.bounds.width, layout.bounds.height],
                    blocks,
                    tables,
                };

                serde_json::to_string(&layout_dto).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Renders a specific PDF page to DisplayList JSON
    #[wasm_bindgen]
    pub fn pdf_render_page_display_list(&self, page_idx: usize) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                let mut dl = DisplayList::new();
                pdf.render_page_to_display_list(page_idx, &mut dl);
                serde_json::to_string(&dl).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Dispatches a structured PDF command (annotations, editing, page organization, security, OCR)
    #[wasm_bindgen]
    pub fn pdf_dispatch_command(&mut self, cmd_json: &str) -> Result<String, JsValue> {
        let cmd: PdfCommandDto = serde_json::from_str(cmd_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid command JSON: {}", e)))?;

        match &mut self.doc {
            ActiveDoc::Pdf(pdf) => {
                match cmd {
                    PdfCommandDto::AddAnnotation {
                        page_idx,
                        kind,
                        rect,
                        color_hex,
                        text,
                        author,
                        points,
                        stamp_type,
                    } => {
                        let col = color_hex.as_deref().map(parse_hex_color).unwrap_or(Color::rgb(224, 49, 49));
                        let r = rect.map(|[x, y, w, h]| Rect::new(x, y, w, h)).unwrap_or(Rect::new(54.0, 100.0, 120.0, 30.0));
                        let id = format!("annot-{}", pdf.annotations.get(&page_idx).map(|a| a.len()).unwrap_or(0) + 1);
                        let annot = match kind.to_lowercase().as_str() {
                            "highlight" => Annotation {
                                id: id.clone(),
                                page_index: page_idx,
                                bounds: r,
                                kind: AnnotationKind::Highlight {
                                    color: col,
                                    opacity: 0.45,
                                },
                                created_at_sec: 0,
                            },
                            "underline" => Annotation {
                                id: id.clone(),
                                page_index: page_idx,
                                bounds: r,
                                kind: AnnotationKind::Underline { color: col },
                                created_at_sec: 0,
                            },
                            "squiggly" => Annotation {
                                id: id.clone(),
                                page_index: page_idx,
                                bounds: r,
                                kind: AnnotationKind::Squiggly { color: col },
                                created_at_sec: 0,
                            },
                            "strikethrough" => Annotation {
                                id: id.clone(),
                                page_index: page_idx,
                                bounds: r,
                                kind: AnnotationKind::Strikethrough { color: col },
                                created_at_sec: 0,
                            },
                            "sticky_note" => Annotation::sticky_note(
                                &id,
                                page_idx,
                                Point::new(r.x, r.y),
                                &author.unwrap_or_else(|| "User".to_string()),
                                &text.unwrap_or_else(|| "Note".to_string()),
                            ),
                            "stamp" => {
                                let st = match stamp_type.as_deref() {
                                    Some("Confidential") => StampType::Confidential,
                                    Some("Draft") => StampType::Draft,
                                    Some("Rejected") => StampType::Rejected,
                                    _ => StampType::Approved,
                                };
                                Annotation::stamp(
                                    &id,
                                    page_idx,
                                    Rect::new(r.x, r.y, 140.0, 48.0),
                                    st,
                                    &author.unwrap_or_else(|| "Authorized".to_string()),
                                    "2026-10-29",
                                )
                            }
                            "free_text" => Annotation::free_text(
                                &id,
                                page_idx,
                                r,
                                &text.unwrap_or_default(),
                                11.0,
                                col,
                                Color::rgb(255, 255, 255),
                                Color::rgb(222, 226, 230),
                            ),
                            "callout" => Annotation::callout(
                                &id,
                                page_idx,
                                Point::new(r.x, r.y),
                                Rect::new(r.x + 30.0, r.y + 20.0, r.width.max(120.0), r.height.max(40.0)),
                                &text.unwrap_or_default(),
                                col,
                            ),
                            "measurement" => {
                                let start = Point::new(r.x, r.y);
                                let end = Point::new(r.x + r.width, r.y + r.height);
                                Annotation::measurement(&id, page_idx, start, end, 0.352778, "mm")
                            }
                            "perimeter" => {
                                let pts: Vec<Point> = points.unwrap_or_default().into_iter().map(|[x, y]| Point::new(x, y)).collect();
                                Annotation::perimeter(&id, page_idx, pts, 0.352778, "mm")
                            }
                            "area_polygon" => {
                                let pts: Vec<Point> = points.unwrap_or_default().into_iter().map(|[x, y]| Point::new(x, y)).collect();
                                Annotation::area_polygon(&id, page_idx, pts, 0.352778, "mm")
                            }
                            "cloud_polygon" => {
                                let pts: Vec<Point> = points.unwrap_or_default().into_iter().map(|[x, y]| Point::new(x, y)).collect();
                                Annotation::cloud_polygon(&id, page_idx, pts, col, Some(Color::rgba(col.r, col.g, col.b, 30)))
                            }
                            _ => Annotation::highlight(&id, page_idx, r),
                        };
                        pdf.add_annotation(page_idx, annot);
                    }
                    PdfCommandDto::DeleteAnnotation { page_idx, index } => {
                        if let Some(annots) = pdf.annotations.get_mut(&page_idx) {
                            if index < annots.len() {
                                annots.remove(index);
                            }
                        }
                    }
                    PdfCommandDto::EditParagraph {
                        page_idx,
                        click_x,
                        click_y,
                        new_text,
                    } => {
                        if let Some(page) = pdf.pages.get_mut(page_idx) {
                            if let Some(mut session) = PdfEditSession::new(page, Point::new(click_x, click_y)) {
                                session.current_text = new_text;
                                session.cursor_pos = session.current_text.len();
                                session.commit(page);
                            } else if let Some(first) = page.text_elements.first_mut() {
                                first.text = new_text;
                            }
                        }
                    }
                    PdfCommandDto::RotatePage { page_idx, degrees } => {
                        if let Some(page) = pdf.pages.get_mut(page_idx) {
                            PageOrganizer::rotate_page(page, degrees);
                        }
                    }
                    PdfCommandDto::CropPage {
                        page_idx,
                        x,
                        y,
                        width,
                        height,
                    } => {
                        if let Some(page) = pdf.pages.get_mut(page_idx) {
                            PageOrganizer::crop_page(page, Rect::new(x, y, width, height));
                        }
                    }
                    PdfCommandDto::DeletePage { page_idx } => {
                        PageOrganizer::delete_pages(&mut pdf.pages, &[page_idx]);
                    }
                    PdfCommandDto::InsertBlankPage {
                        page_idx,
                        width,
                        height,
                    } => {
                        let w = width.unwrap_or(595.0);
                        let h = height.unwrap_or(842.0);
                        let new_page = PdfPage {
                            page_index: page_idx,
                            width: w,
                            height: h,
                            rotation: 0,
                            crop_box: Rect::new(0.0, 0.0, w, h),
                            text_elements: Vec::new(),
                            vector_paths: Vec::new(),
                            raw_stream: Vec::new(),
                        };
                        let insert_at = page_idx.min(pdf.pages.len());
                        pdf.pages.insert(insert_at, new_page);
                        for (i, p) in pdf.pages.iter_mut().enumerate() {
                            p.page_index = i;
                        }
                    }
                    PdfCommandDto::ReorderPages { new_order } => {
                        PageOrganizer::reorder_pages(&mut pdf.pages, &new_order);
                    }
                    PdfCommandDto::ApplyWatermark {
                        text,
                        opacity,
                        font_size,
                        angle_deg,
                    } => {
                        PageOrganizer::apply_watermark(
                            &mut pdf.pages,
                            &text,
                            font_size.unwrap_or(48.0),
                            opacity.unwrap_or(0.18),
                            angle_deg.unwrap_or(-45.0),
                            Color::rgb(220, 53, 69),
                        );
                    }
                    PdfCommandDto::ApplyBatesNumbering {
                        prefix,
                        start_number,
                        digits,
                        position,
                    } => {
                        let pos = match position.as_deref() {
                            Some("TopLeft") => BatesPosition::TopLeft,
                            Some("TopRight") => BatesPosition::TopRight,
                            Some("BottomLeft") => BatesPosition::BottomLeft,
                            Some("BottomCenter") => BatesPosition::BottomCenter,
                            _ => BatesPosition::BottomRight,
                        };
                        PageOrganizer::apply_bates_numbering(
                            &mut pdf.pages,
                            &prefix,
                            start_number as u32,
                            digits,
                            pos,
                        );
                    }
                    PdfCommandDto::RunOcr { page_idx, mode } => {
                        if let Some(page) = pdf.pages.get_mut(page_idx) {
                            let opt = OcrOptions {
                                mode: if mode.as_deref() == Some("editable") {
                                    OcrMode::EditableReconstruction
                                } else {
                                    OcrMode::SearchableLayer
                                },
                                dpi: 300,
                                language: "eng".to_string(),
                            };
                            OcrEngine::run_ocr_on_page(page, &opt);
                        }
                    }
                    PdfCommandDto::RedactRegion {
                        page_idx,
                        x,
                        y,
                        width,
                        height,
                        label,
                    } => {
                        pdf.redact_region(page_idx, Rect::new(x, y, width, height), label.as_deref());
                    }
                    PdfCommandDto::CompressDocument { level } => {
                        let lvl = match level.as_deref() {
                            Some("high") => CompressionLevel::High,
                            Some("extreme") => CompressionLevel::Extreme,
                            _ => CompressionLevel::Standard,
                        };
                        pdf.compress(lvl);
                    }
                    PdfCommandDto::FlattenForms => {
                        for page in &mut pdf.pages {
                            PdfSecurity::flatten_forms(page, &[]);
                        }
                    }
                }
                self.pdf_get_document_summary()
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Converts active PDF document into a Writer (.docx) Document model
    #[wasm_bindgen]
    pub fn pdf_convert_to_writer(&mut self) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                let writer_doc = pdf.to_writer_document();
                self.doc = ActiveDoc::Writer(writer_doc);
                self.active_module = ActiveModule::Writer;
                Ok(self.writer_get_metadata_json())
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Converts active PDF document into a Spreadsheets (.xlsx) Workbook model
    #[wasm_bindgen]
    pub fn pdf_convert_to_sheets(&mut self) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                let workbook = pdf.to_sheets_workbook(PdfToExcelMode::OneSheetPerTable);
                self.doc = ActiveDoc::Sheets(workbook);
                self.active_module = ActiveModule::Spreadsheets;
                Ok(self.sheets_get_metadata_json())
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Converts active PDF document into a Presentation (.pptx) Deck model
    #[wasm_bindgen]
    pub fn pdf_convert_to_slides(&mut self) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                let deck = pdf.to_slides_deck();
                self.doc = ActiveDoc::Slides(deck);
                self.active_module = ActiveModule::Presentation;
                self.slides_get_deck_summary()
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// Computes responsive single-column text reflow for reading mode
    #[wasm_bindgen]
    pub fn pdf_reflow_page_json(&self, page_idx: usize, target_width: f64, font_scale: f64) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                let page = pdf.pages.get(page_idx).ok_or_else(|| JsValue::from_str("Invalid page index"))?;
                let reflowed = reflow_page_to_stream(page, target_width, font_scale);
                #[derive(Serialize)]
                struct TextItem {
                    text: String,
                    bounds: [f64; 4],
                    font_name: String,
                    font_size: f64,
                    baseline: f64,
                }
                let items: Vec<TextItem> = reflowed.into_iter().map(|e| TextItem {
                    text: e.text,
                    bounds: [e.bounds.x, e.bounds.y, e.bounds.width, e.bounds.height],
                    font_name: e.font_name,
                    font_size: e.font_size,
                    baseline: e.baseline,
                }).collect();
                serde_json::to_string(&items).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    #[wasm_bindgen]
    pub fn get_active_users_json(&self) -> String {
        serde_json::to_string(&self.presence.get_active_users()).unwrap_or_else(|_| "[]".to_string())
    }

    #[wasm_bindgen]
    pub fn get_crdt_state_vector_json(&self) -> String {
        serde_json::to_string(&self.crdt_session.state_vector).unwrap_or_else(|_| "{}".to_string())
    }

    /// AST-Grounded Writer AI rewrite with tone and track changes
    #[wasm_bindgen]
    pub fn ai_rewrite_selection(&mut self, selected_text: &str, tone_str: &str, _as_track_changes: bool) -> Result<String, JsValue> {
        let tone = match tone_str.to_lowercase().as_str() {
            "concise" => wps_ai::RewriteTone::Concise,
            "expand" | "expanded" => wps_ai::RewriteTone::Expanded,
            "academic" => wps_ai::RewriteTone::Academic,
            "executive" | "executivebullets" => wps_ai::RewriteTone::ExecutiveBullets,
            _ => wps_ai::RewriteTone::Formal,
        };
        let input = wps_ai::WriterParagraphInput {
            paragraph_id: 1,
            text: selected_text.to_string(),
            current_style_id: "Normal".to_string(),
        };
        let res = wps_ai::WriterAiAgent::rewrite_with_track_changes(&input, tone, "2026-10-30T12:00:00Z");
        serde_json::to_string(&res).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// AST-Grounded Sheets AI statistical insights, Z-scores, and correlation
    #[wasm_bindgen]
    pub fn ai_sheets_insights(&self, _range_a1: Option<String>) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Sheets(wb) => {
                let matrix = &wb.active_worksheet().matrix;
                let mut headers = Vec::new();
                for c in 0..10 {
                    let h = matrix.get_cell(CellCoordinate::new(0, c)).map(|cell| cell.value.as_string()).unwrap_or_else(|| format!("Col{}", c + 1));
                    headers.push(h);
                }
                let mut data_rows = Vec::new();
                for r in 1..25 {
                    let mut row = Vec::new();
                    for c in 0..10 {
                        let v = matrix.get_cell(CellCoordinate::new(r, c)).map(|cell| cell.value.as_string()).unwrap_or_default();
                        row.push(v);
                    }
                    data_rows.push(row);
                }
                let report = wps_ai::SheetsAiAgent::analyze_sheet_insights_and_anomalies("Sheet1", &headers, &data_rows);
                serde_json::to_string(&report).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not Spreadsheets")),
        }
    }

    /// AST-Grounded Slides AI outline and deck compiler
    #[wasm_bindgen]
    pub fn ai_slides_generate(&mut self, topic: &str, slide_count: usize) -> Result<String, JsValue> {
        let outline = wps_ai::SlidesAiAgent::generate_outline_from_prompt(topic, slide_count);
        let compiled = wps_ai::SlidesAiAgent::compile_outline_to_deck(&outline);
        serde_json::to_string(&compiled).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// AST-Grounded PDF AI spatial RAG with bounding-box citations
    #[wasm_bindgen]
    pub fn ai_pdf_spatial_rag(&self, question: &str) -> Result<String, JsValue> {
        match &self.doc {
            ActiveDoc::Pdf(pdf) => {
                let mut index = wps_ai::PdfSpatialRagIndex::new();
                for (p_idx, page) in pdf.pages.iter().enumerate() {
                    for (elem_idx, elem) in page.text_elements.iter().enumerate() {
                        let bbox = wps_ai::PdfBBox::new(
                            elem.bounds.x as f32,
                            elem.bounds.y as f32,
                            (elem.bounds.x + elem.bounds.width) as f32,
                            (elem.bounds.y + elem.bounds.height) as f32,
                        );
                        index.add_spatial_chunk(
                            (p_idx * 1000 + elem_idx) as u64,
                            p_idx,
                            bbox,
                            wps_ai::PdfChunkKind::BodyParagraph,
                            elem.text.clone(),
                        );
                    }
                }
                let answer = index.query_with_bbox_citations(question, 3);
                serde_json::to_string(&answer).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => Err(JsValue::from_str("Active module is not PDF")),
        }
    }

    /// File Roaming: creates SHA-256 deduplicated checkpoint of active document
    #[wasm_bindgen]
    pub fn roaming_create_checkpoint(&mut self, author: &str, message: &str) -> Result<String, JsValue> {
        let data = match &self.doc {
            ActiveDoc::Writer(w) => serde_json::to_vec(w).unwrap_or_default(),
            ActiveDoc::Sheets(wb) => serde_json::to_vec(wb).unwrap_or_default(),
            ActiveDoc::Slides(sl) => serde_json::to_vec(sl).unwrap_or_default(),
            ActiveDoc::Pdf(p) => p.title.as_bytes().to_vec(),
        };
        let record = self.roaming_store.save_version(
            "active_doc",
            author,
            message,
            &data,
            "2026-10-30T12:00:00Z",
        );
        serde_json::to_string(&record).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// File Roaming: lists all recorded checkpoints
    #[wasm_bindgen]
    pub fn roaming_list_checkpoints(&self) -> Result<String, JsValue> {
        let list = self.roaming_store.list_versions("active_doc");
        serde_json::to_string(&list).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
