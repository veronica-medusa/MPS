use serde::{Deserialize, Serialize};
use wps_core::color::Color;
use wps_sheets::cell::{CellCoordinate, CellValue};
use wps_sheets::sparse_matrix::SparseMatrix;
use wps_writer::ast::{Paragraph, TextRun, Document as WriterDocument};
use wps_slides::scene_graph::{Slide, PresentationDeck as SlidesDeck};
use wps_pdf::{PageOrganizer, PdfDocument};
use crate::engine::PyError;

pub struct OfficeContext<'a> {
    pub writer: Option<&'a mut WriterDocument>,
    pub sheets: Option<&'a mut SparseMatrix>,
    pub slides: Option<&'a mut SlidesDeck>,
    pub pdf: Option<&'a mut PdfDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroExecutionLog {
    pub actions_performed: usize,
    pub messages: Vec<String>,
}

pub struct MacroRunner;

impl MacroRunner {
    pub fn run_macro(script: &str, ctx: &mut OfficeContext) -> Result<MacroExecutionLog, PyError> {
        let lines: Vec<&str> = script.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        let mut actions = 0;
        let mut messages = Vec::new();

        for line in lines {
            // Sheets commands: `sheets.set("A1", 100)`
            if line.starts_with("sheets.set(") && line.ends_with(')') {
                let inside = &line[11..line.len() - 1];
                if let Some((addr_str, val_str)) = inside.split_once(',') {
                    let addr = addr_str.trim().trim_matches(|c| c == '\'' || c == '"');
                    let val_trimmed = val_str.trim().trim_matches(|c| c == '\'' || c == '"');

                    let coord = CellCoordinate::parse(addr).ok_or_else(|| PyError::RuntimeError(format!("Invalid coordinate: {}", addr)))?;
                    let val = if let Ok(num) = val_trimmed.parse::<f64>() {
                        CellValue::Number(num)
                    } else {
                        CellValue::Text(val_trimmed.to_string())
                    };

                    if let Some(ref mut sheets) = ctx.sheets {
                        sheets.set_value(coord, val.clone());
                        actions += 1;
                        messages.push(format!("Set sheets[{}] = {:?}", addr, val));
                    }
                }
            }
            // Writer commands: `writer.add_heading("Title", 1)`
            else if line.starts_with("writer.add_heading(") && line.ends_with(')') {
                let inside = &line[19..line.len() - 1];
                if let Some((text_str, lvl_str)) = inside.split_once(',') {
                    let text = text_str.trim().trim_matches(|c| c == '\'' || c == '"');
                    let lvl: usize = lvl_str.trim().parse().unwrap_or(1);

                    if let Some(ref mut writer) = ctx.writer {
                        let id = format!("macro-p-{}", actions + 1);
                        writer.add_paragraph(Paragraph::heading(&id, lvl, text));
                        actions += 1;
                        messages.push(format!("Added Writer heading lvl {}: '{}'", lvl, text));
                    }
                }
            }
            // Writer commands: `writer.add_paragraph("Body")`
            else if line.starts_with("writer.add_paragraph(") && line.ends_with(')') {
                let text = line[21..line.len() - 1].trim().trim_matches(|c| c == '\'' || c == '"');
                if let Some(ref mut writer) = ctx.writer {
                    let id = format!("macro-p-{}", actions + 1);
                    let mut p = Paragraph::new(&id);
                    p.add_run(TextRun::new(text));
                    writer.add_paragraph(p);
                    actions += 1;
                    messages.push(format!("Added Writer paragraph: '{}'", text));
                }
            }
            // Slides commands: `slides.add_slide("Title")`
            else if line.starts_with("slides.add_slide(") && line.ends_with(')') {
                let title = line[17..line.len() - 1].trim().trim_matches(|c| c == '\'' || c == '"');
                if let Some(ref mut slides) = ctx.slides {
                    let slide_id = format!("slide-{}", slides.slides.len() + 1);
                    slides.add_slide(Slide::new(&slide_id, title, "layout-title"));
                    actions += 1;
                    messages.push(format!("Added Slide: '{}'", title));
                }
            }
            // PDF commands: `pdf.watermark("DRAFT")`
            else if line.starts_with("pdf.watermark(") && line.ends_with(')') {
                let text = line[14..line.len() - 1].trim().trim_matches(|c| c == '\'' || c == '"');
                if let Some(ref mut pdf) = ctx.pdf {
                    PageOrganizer::apply_watermark(&mut pdf.pages, text, 48.0, 0.3, 45.0, Color::rgb(180, 180, 180));
                    actions += 1;
                    messages.push(format!("Applied watermark to PDF: '{}'", text));
                }
            }
        }

        Ok(MacroExecutionLog {
            actions_performed: actions,
            messages,
        })
    }
}
