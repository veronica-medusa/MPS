use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use wps_render::{DisplayList, DrawCommand};
use wps_writer::ast::Document as WriterDocument;
use wps_sheets::cell::CellCoordinate;
use wps_sheets::sparse_matrix::SparseMatrix;
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
    Sheets(SparseMatrix),
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
            ActiveModule::Spreadsheets => ActiveDoc::Sheets(SparseMatrix::new()),
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
            ActiveDoc::Sheets(matrix) => {
                let cell_w = 90.0;
                let cell_h = 24.0;
                let rows = (viewport_height / cell_h).min(50.0) as u32;
                let cols = (viewport_width / cell_w).min(20.0) as u16;

                for r in 0..rows {
                    for c in 0..cols {
                        let cell_rect = Rect::new(c as f64 * cell_w, r as f64 * cell_h, cell_w, cell_h);
                        dl.push(DrawCommand::StrokeRect {
                            rect: cell_rect,
                            color: Color::rgb(233, 236, 239),
                            width: 1.0,
                        });

                        if let Some(cell) = matrix.get_cell(CellCoordinate::new(r, c)) {
                            dl.push(DrawCommand::DrawTextRun {
                                text: cell.value.as_string(),
                                font_family: "JetBrains Mono".to_string(),
                                font_size_px: 11.0,
                                bold: false,
                                italic: false,
                                color: Color::MERCURY_INK,
                                origin: Point::new(cell_rect.x + 4.0, cell_rect.y + 16.0),
                            });
                        }
                    }
                }
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

    /// Evaluates python script against current spreadsheet or office context
    #[wasm_bindgen]
    pub fn eval_python(&mut self, script: &str) -> Result<String, JsValue> {
        match &mut self.doc {
            ActiveDoc::Sheets(matrix) => {
                let out = self.python_engine.eval_cell(script, matrix)
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
                ActiveDoc::Sheets(s) => { MutationApplicator::apply_to_sheets(s, m); }
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
