use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wps_core::color::Color;
use wps_core::geometry::Rect;
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
            ActiveModule::Writer => ActiveDoc::Writer(WriterDocument::new("Untitled Document", "OpenWPS User")),
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
                // Render A4 white page
                let page_rect = Rect::new((viewport_width - 595.0).max(20.0) * 0.5, 30.0, 595.0, 842.0);
                dl.push(DrawCommand::FillRect {
                    rect: page_rect,
                    color: Color::WHITE,
                });
                dl.push(DrawCommand::StrokeRect {
                    rect: page_rect,
                    color: Color::rgb(222, 226, 230),
                    width: 1.0,
                });

                // Render paragraphs
                let mut cur_y = page_rect.y + 40.0;
                for sec in &writer_doc.sections {
                    for elem in &sec.elements {
                        if let wps_writer::ast::BlockElement::Paragraph(p) = elem {
                            let text = p.plain_text();
                            dl.push(DrawCommand::DrawTextRun {
                                text,
                                font_family: "Inter".to_string(),
                                font_size_px: 12.0,
                                bold: p.style_name.contains("Heading"),
                                italic: false,
                                color: Color::MERCURY_INK,
                                origin: wps_core::Point::new(page_rect.x + 40.0, cur_y),
                            });
                            cur_y += 24.0;
                        }
                    }
                }
            }
            ActiveDoc::Sheets(matrix) => {
                // Render grid
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
                                origin: wps_core::Point::new(cell_rect.x + 4.0, cell_rect.y + 16.0),
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
