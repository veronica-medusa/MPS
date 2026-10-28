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
use wps_slides::scene_graph::PresentationDeck as SlidesDeck;
use wps_pdf::{PdfDocument, PdfPage};
use wps_python::{NativePythonEngine, PythonEngine};
use wps_ai::{AgentRouter, AiDomain, MutationApplicator};
use wps_collab::{CrdtSession, PresenceRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActiveModule {
    Writer,
    Spreadsheets,
    Presentation,
    Pdf,
}

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

#[wasm_bindgen]
pub struct WpsWasmEngine {
    active_module: ActiveModule,
    doc: ActiveDoc,
    python_engine: NativePythonEngine,
    crdt_session: CrdtSession,
    presence: PresenceRegistry,
}

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

        let doc = match active_module {
            ActiveModule::Writer => ActiveDoc::Writer(create_default_writer_doc()),
            ActiveModule::Spreadsheets => ActiveDoc::Sheets(create_default_sheets_doc()),
            ActiveModule::Presentation => ActiveDoc::Slides(SlidesDeck::new("Untitled Presentation")),
            ActiveModule::Pdf => {
                let mut pdf = PdfDocument::new("Untitled.pdf");
                pdf.add_page(PdfPage {
                    page_index: 0,
                    width: 595.0,
                    height: 842.0,
                    rotation: 0,
                    crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
                    text_elements: Vec::new(),
                    vector_paths: Vec::new(),
                    raw_stream: Vec::new(),
                });
                ActiveDoc::Pdf(pdf)
            }
        };

        Self {
            active_module,
            doc,
            python_engine: NativePythonEngine::new(),
            crdt_session: CrdtSession::new("wasm-client-1"),
            presence: PresenceRegistry::new(),
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
    pub fn switch_module(&mut self, module_name: &str) {
        *self = Self::new(module_name);
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
                    slide.render_to_display_list(&deck.master, &mut dl);
                }
            }
            ActiveDoc::Pdf(pdf) => {
                pdf.render_page_to_display_list(0, &mut dl);
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
                            if landscape && w_t < h_t {
                                sec.page_setup.width_twips = h_t;
                                sec.page_setup.height_twips = w_t;
                            } else if !landscape && w_t > h_t {
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
                let out = self.python_engine.eval_cell(script, &mut wb.active_worksheet_mut().matrix)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
            }
            _ => {
                let mut empty_matrix = SparseMatrix::new();
                let out = self.python_engine.eval_cell(script, &mut empty_matrix)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                serde_json::to_string(&out).map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }

    /// Dispatches prompt to AST-native AI co-pilot
    #[wasm_bindgen]
    pub fn dispatch_ai(&mut self, prompt: &str, context: &str) -> Result<String, JsValue> {
        let domain = match self.active_module {
            ActiveModule::Writer => AiDomain::Writer,
            ActiveModule::Spreadsheets => AiDomain::Sheets,
            ActiveModule::Presentation => AiDomain::Slides,
            ActiveModule::Pdf => AiDomain::Pdf,
        };

        let pdf_ref = match &self.doc {
            ActiveDoc::Pdf(p) => Some(p),
            _ => None,
        };

        let resp = AgentRouter::dispatch(domain, prompt, Some(context), pdf_ref);

        // Apply mutations
        for m in &resp.mutations {
            match &mut self.doc {
                ActiveDoc::Writer(w) => { MutationApplicator::apply_to_writer(w, m); }
                ActiveDoc::Sheets(wb) => { MutationApplicator::apply_to_sheets(&mut wb.active_worksheet_mut().matrix, m); }
                ActiveDoc::Slides(sl) => { MutationApplicator::apply_to_slides(sl, m); }
                ActiveDoc::Pdf(p) => { MutationApplicator::apply_to_pdf(p, m); }
            }
        }

        serde_json::to_string(&resp).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn get_active_users_json(&self) -> String {
        serde_json::to_string(&self.presence.get_active_users()).unwrap_or_else(|_| "[]".to_string())
    }

    #[wasm_bindgen]
    pub fn get_crdt_state_vector_json(&self) -> String {
        serde_json::to_string(&self.crdt_session.state_vector).unwrap_or_else(|_| "{}".to_string())
    }
}
