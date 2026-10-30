//! Cross-Application Python Macro Automation Engine (`wps-python`)
//!
//! Exposes the unified `wp` Python automation API enabling single-transaction scripts
//! across Writer (`wp.writer`), Spreadsheets (`wp.sheets`), Presentation (`wp.slides`),
//! and PDF Toolkit (`wp.pdf`).

use crate::sandbox::{PyMacroReport, PyRuntimeError, WorkspaceMacroContext};
use wps_core::color::Color;
use wps_core::geometry::Rect;
use wps_sheets::cell::{CellCoordinate, CellValue};
use wps_writer::ast::{Paragraph, TextRun};
use wps_slides::scene_graph::Slide;
use wps_slides::shape::{ParametricShape, ShapeFill, ShapeGeometryType};
use wps_pdf::PageOrganizer;

pub struct MacroEngine;

impl MacroEngine {
    /// Executes a multi-line cross-application Python automation macro against the active workspace context.
    pub fn execute(
        script: &str,
        ctx: &mut WorkspaceMacroContext,
    ) -> Result<PyMacroReport, PyRuntimeError> {
        let start_time = std::time::Instant::now();
        let lines: Vec<&str> = script.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        let mut actions = 0;
        let mut messages = Vec::new();

        // Transactional execution of macro instructions
        for line in lines {
            // Comments or blank lines
            if line.starts_with('#') {
                continue;
            }

            // 1. Writer Commands: `wp.writer.*`
            if line.starts_with("wp.writer.") {
                let call = &line[10..];
                Self::handle_writer_call(call, ctx, &mut actions, &mut messages)?;
            }
            // 2. Sheets Commands: `wp.sheets.*`
            else if line.starts_with("wp.sheets.") {
                let call = &line[10..];
                Self::handle_sheets_call(call, ctx, &mut actions, &mut messages)?;
            }
            // 3. Slides Commands: `wp.slides.*`
            else if line.starts_with("wp.slides.") {
                let call = &line[10..];
                Self::handle_slides_call(call, ctx, &mut actions, &mut messages)?;
            }
            // 4. PDF Commands: `wp.pdf.*`
            else if line.starts_with("wp.pdf.") {
                let call = &line[7..];
                Self::handle_pdf_call(call, ctx, &mut actions, &mut messages)?;
            }
        }

        let duration_us = start_time.elapsed().as_micros() as u64;
        Ok(PyMacroReport {
            actions_performed: actions,
            messages,
            duration_us,
            success: true,
        })
    }

    fn handle_writer_call(
        call: &str,
        ctx: &mut WorkspaceMacroContext,
        actions: &mut usize,
        messages: &mut Vec<String>,
    ) -> Result<(), PyRuntimeError> {
        let writer = ctx.writer.as_deref_mut().ok_or_else(|| {
            PyRuntimeError::ExecutionError("No active Writer document open in workspace".to_string())
        })?;

        // `insert_heading("Title", level=1)`
        if call.starts_with("insert_heading(") && call.ends_with(')') {
            let inside = &call[15..call.len() - 1];
            let (text, lvl) = Self::parse_two_args(inside);
            let lvl_num: usize = lvl.and_then(|s| s.parse().ok()).unwrap_or(1);

            let id = format!("macro-h-{}", *actions + 1);
            writer.add_paragraph(Paragraph::heading(&id, lvl_num, &text));
            *actions += 1;
            messages.push(format!("Writer: Added Heading {} '{}'", lvl_num, text));
        }
        // `insert_paragraph("Body text")`
        else if call.starts_with("insert_paragraph(") && call.ends_with(')') {
            let text = Self::parse_single_string_arg(&call[17..call.len() - 1]);
            let id = format!("macro-p-{}", *actions + 1);
            let mut p = Paragraph::new(&id);
            p.add_run(TextRun::new(&text));
            writer.add_paragraph(p);
            *actions += 1;
            messages.push(format!("Writer: Added Paragraph '{}'", text));
        }
        // `replace_text("old", "new")`
        else if call.starts_with("replace_text(") && call.ends_with(')') {
            let inside = &call[13..call.len() - 1];
            let (target, replacement) = Self::parse_two_args(inside);
            let repl_str = replacement.unwrap_or_default();

            for sec in &mut writer.sections {
                for elem in &mut sec.elements {
                    if let wps_writer::ast::BlockElement::Paragraph(p) = elem {
                        for inline in &mut p.inlines {
                            if let wps_writer::ast::InlineElement::Run(run) = inline {
                                if run.text.contains(&target) {
                                    run.text = run.text.replace(&target, &repl_str);
                                    *actions += 1;
                                    messages.push(format!("Writer: Replaced '{}' with '{}'", target, repl_str));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_sheets_call(
        call: &str,
        ctx: &mut WorkspaceMacroContext,
        actions: &mut usize,
        messages: &mut Vec<String>,
    ) -> Result<(), PyRuntimeError> {
        let sheets = ctx.sheets.as_deref_mut().ok_or_else(|| {
            PyRuntimeError::ExecutionError("No active Spreadsheet matrix open in workspace".to_string())
        })?;

        // `set_cell("A1", 42)` or `set_range("A1", ...)`
        if (call.starts_with("set_cell(") || call.starts_with("set_range(")) && call.ends_with(')') {
            let prefix_len = if call.starts_with("set_cell(") { 9 } else { 10 };
            let inside = &call[prefix_len..call.len() - 1];
            let (addr, val_opt) = Self::parse_two_args(inside);
            let coord = CellCoordinate::parse(&addr).ok_or_else(|| {
                PyRuntimeError::ExecutionError(format!("Invalid cell coordinate: {}", addr))
            })?;

            let cell_val = match val_opt.as_deref() {
                Some(s) if s.parse::<f64>().is_ok() => CellValue::Number(s.parse::<f64>().unwrap()),
                Some(s) if s == "True" || s == "true" => CellValue::Boolean(true),
                Some(s) if s == "False" || s == "false" => CellValue::Boolean(false),
                Some(s) => CellValue::Text(s.to_string()),
                None => CellValue::Empty,
            };

            sheets.set_value(coord, cell_val.clone());
            *actions += 1;
            messages.push(format!("Sheets: Set cell {} to {:?}", addr, cell_val));
        }

        Ok(())
    }

    fn handle_slides_call(
        call: &str,
        ctx: &mut WorkspaceMacroContext,
        actions: &mut usize,
        messages: &mut Vec<String>,
    ) -> Result<(), PyRuntimeError> {
        let slides = ctx.slides.as_deref_mut().ok_or_else(|| {
            PyRuntimeError::ExecutionError("No active Presentation deck open in workspace".to_string())
        })?;

        // `add_slide("Slide Title")`
        if call.starts_with("add_slide(") && call.ends_with(')') {
            let title = Self::parse_single_string_arg(&call[10..call.len() - 1]);
            let slide_idx = slides.slides.len();
            let slide_id = format!("slide-{}", slide_idx + 1);
            slides.add_slide(Slide::new(&slide_id, &title, "layout-title-content"));

            if !title.is_empty() {
                let mut shape = ParametricShape::new(
                    &format!("title-{}", slide_idx + 1),
                    ShapeGeometryType::Rectangle,
                    Rect::new(50.0, 40.0, 620.0, 70.0),
                );
                shape.name = title.clone();
                shape.text_frame = Some(wps_slides::shape::TextFrame::simple(&title, 18.0, Color::WHITE));
                shape.fill = ShapeFill::Solid(Color::from_hex("#333333").unwrap_or_else(|_| Color::rgba(51, 51, 51, 255)));
                slides.slides[slide_idx].add_shape(shape);
            }

            *actions += 1;
            messages.push(format!("Slides: Added slide {} with title '{}'", slide_idx + 1, title));
        }

        Ok(())
    }

    fn handle_pdf_call(
        call: &str,
        ctx: &mut WorkspaceMacroContext,
        actions: &mut usize,
        messages: &mut Vec<String>,
    ) -> Result<(), PyRuntimeError> {
        let pdf = ctx.pdf.as_deref_mut().ok_or_else(|| {
            PyRuntimeError::ExecutionError("No active PDF document open in workspace".to_string())
        })?;

        // `add_watermark("CONFIDENTIAL")`
        if call.starts_with("add_watermark(") && call.ends_with(')') {
            let wm_text = Self::parse_single_string_arg(&call[14..call.len() - 1]);
            PageOrganizer::apply_watermark(&mut pdf.pages, &wm_text, 48.0, 0.25, 45.0, Color::rgba(128, 128, 128, 255));
            *actions += 1;
            messages.push(format!("PDF: Added watermark '{}'", wm_text));
        }

        Ok(())
    }

    fn parse_single_string_arg(arg: &str) -> String {
        arg.trim().trim_matches(|c| c == '\'' || c == '"' || c == ' ').to_string()
    }

    fn parse_two_args(inside: &str) -> (String, Option<String>) {
        if let Some((first, second)) = inside.split_once(',') {
            (
                Self::parse_single_string_arg(first),
                Some(Self::parse_single_string_arg(second)),
            )
        } else {
            (Self::parse_single_string_arg(inside), None)
        }
    }
}
