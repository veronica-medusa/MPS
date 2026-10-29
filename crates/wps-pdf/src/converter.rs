use wps_core::geometry::Rect;
use wps_core::units::Twips;
use wps_sheets::{CellCoordinate, CellValue, SparseMatrix, Workbook, Worksheet};
use wps_slides::scene_graph::{PresentationDeck, Slide};
use wps_slides::shape::{ParametricShape, ShapeFill, ShapeGeometryType};
use crate::parser::{PdfPage, PdfTextElement};
use crate::spatial::{BlockKind, SpatialReconstructor};
use crate::PdfDocument;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfToExcelMode {
    OneSheetPerTable,
    OneSheetPerPage,
}

pub struct PdfConverter;

impl PdfConverter {
    /// Parses a raw text token into a typed spreadsheet CellValue
    pub fn parse_cell_value(text: &str) -> CellValue {
        let trimmed = text.trim();
        let cleaned = trimmed
            .replace('$', "")
            .replace('£', "")
            .replace('€', "")
            .replace('¥', "")
            .replace(',', "");

        if trimmed.ends_with('%') {
            let pct_str = trimmed.trim_end_matches('%').replace(',', "");
            if let Ok(n) = pct_str.trim().parse::<f64>() {
                CellValue::Number(n / 100.0)
            } else {
                CellValue::Text(trimmed.to_string())
            }
        } else if let Ok(n) = cleaned.parse::<f64>() {
            CellValue::Number(n)
        } else if trimmed.is_empty() {
            CellValue::Empty
        } else {
            CellValue::Text(trimmed.to_string())
        }
    }
    /// Converts a PDF document into a high-fidelity WPS Writer AST Document
    pub fn to_writer_document(title: &str, pages: &[PdfPage]) -> wps_writer::ast::Document {
        let mut doc = wps_writer::ast::Document::new(title, "OpenWPS PDF Converter");
        let mut section = wps_writer::ast::Section::default();

        let mut block_counter = 0;

        for page in pages {
            let layout = SpatialReconstructor::reconstruct_page(page);

            // 1. Convert Text Blocks
            for block in &layout.blocks {
                block_counter += 1;
                let id = format!("p-{}", block_counter);

                let p = match block.kind {
                    BlockKind::Title => {
                        wps_writer::ast::Paragraph::heading(&id, 1, &block.plain_text())
                    }
                    BlockKind::Heading(lvl) => {
                        wps_writer::ast::Paragraph::heading(&id, lvl as usize, &block.plain_text())
                    }
                    BlockKind::ListItem => {
                        let mut para = wps_writer::ast::Paragraph::new(&id);
                        para.indent_left_twips = Twips(720); // 0.5 inch indent
                        para.add_run(
                            wps_writer::ast::TextRun::new(&block.plain_text())
                                .with_size(block.font_size)
                        );
                        para
                    }
                    BlockKind::Paragraph | BlockKind::TableCell | BlockKind::HeaderBlock | BlockKind::FooterBlock => {
                        let mut para = wps_writer::ast::Paragraph::new(&id);
                        let mut run = wps_writer::ast::TextRun::new(&block.plain_text())
                            .with_size(block.font_size);
                        if block.is_bold {
                            run = run.bold();
                        }
                        if block.is_italic {
                            run = run.italic();
                        }
                        para.add_run(run);
                        para
                    }
                };

                section.elements.push(wps_writer::ast::BlockElement::Paragraph(p));
            }

            // 2. Convert Tables
            for table in &layout.tables {
                block_counter += 1;
                let table_id = format!("table-{}", block_counter);
                let mut ast_table = wps_writer::ast::Table {
                    id: table_id,
                    rows: Vec::new(),
                    alignment: wps_writer::ast::TextAlignment::Left,
                    ..Default::default()
                };

                // Group cells by row
                for r in 0..table.rows {
                    let mut row = wps_writer::ast::TableRow {
                        cells: Vec::new(),
                        is_header: r == 0,
                        cant_split: true,
                        is_header_repeat: r == 0,
                        allow_row_split_across_pages: false,
                        ..Default::default()
                    };

                    let row_cells: Vec<_> = table.cells.iter().filter(|c| c.row == r).collect();
                    for c in row_cells {
                        let cell_p = wps_writer::ast::Paragraph {
                            id: format!("tc-p-{}-{}", r, c.col),
                            inlines: vec![wps_writer::ast::InlineElement::Run(
                                wps_writer::ast::TextRun::new(&c.text).with_size(10.0)
                            )],
                            ..Default::default()
                        };

                        let cell_w_twips = Twips((c.bounds.width * 20.0) as i64);
                        row.cells.push(wps_writer::ast::TableCell {
                            elements: vec![wps_writer::ast::BlockElement::Paragraph(cell_p)],
                            col_span: c.col_span,
                            row_span: c.row_span,
                            width_twips: cell_w_twips,
                            background_color: c.background,
                            grid_span: c.col_span as u16,
                            ..Default::default()
                        });
                    }

                    ast_table.rows.push(row);
                }

                section.elements.push(wps_writer::ast::BlockElement::Table(ast_table));
            }
        }

        doc.sections = vec![section];
        doc
    }

    /// Converts all detected tables across PDF pages into a WPS Spreadsheets SparseMatrix
    pub fn to_sheets_matrix(pages: &[PdfPage]) -> SparseMatrix {
        let mut matrix = SparseMatrix::new();
        let mut current_row: u32 = 0;

        for page in pages {
            let layout = SpatialReconstructor::reconstruct_page(page);

            for table in &layout.tables {
                for cell in &table.cells {
                    let target_r = current_row + cell.row as u32;
                    let target_c = cell.col as u16;
                    let coord = CellCoordinate::new(target_r, target_c);

                    // Parse numeric or textual cell values
                    let val = Self::parse_cell_value(&cell.text);
                    matrix.set_value(coord, val);
                }

                current_row += table.rows as u32 + 2; // Separate tables by 2 blank rows
            }
        }

        matrix
    }

    /// Converts all detected tables across PDF pages into a multi-sheet WPS Workbook
    pub fn to_sheets_workbook(_title: &str, pages: &[PdfPage], mode: PdfToExcelMode) -> Workbook {
        let mut wb = Workbook::default();
        wb.sheets.clear();

        match mode {
            PdfToExcelMode::OneSheetPerPage => {
                for (idx, page) in pages.iter().enumerate() {
                    let mut ws = Worksheet::new(&format!("sheet-{}", idx + 1), &format!("Page {}", idx + 1));
                    ws.matrix = Self::to_sheets_matrix(&[page.clone()]);
                    wb.sheets.push(ws);
                }
            }
            PdfToExcelMode::OneSheetPerTable => {
                let mut table_counter = 0;
                for (p_idx, page) in pages.iter().enumerate() {
                    let layout = SpatialReconstructor::reconstruct_page(page);
                    for table in &layout.tables {
                        table_counter += 1;
                        let mut ws = Worksheet::new(
                            &format!("table-{}", table_counter),
                            &format!("Table {} (P{})", table_counter, p_idx + 1),
                        );
                        for cell in &table.cells {
                            let coord = CellCoordinate::new(cell.row as u32, cell.col as u16);
                            let val = Self::parse_cell_value(&cell.text);
                            ws.set_cell_value(coord, val);
                        }
                        wb.sheets.push(ws);
                    }
                }
            }
        }

        if wb.sheets.is_empty() {
            let mut ws = Worksheet::new("sheet-1", "Sheet1");
            ws.matrix = Self::to_sheets_matrix(pages);
            wb.sheets.push(ws);
        }

        wb
    }

    /// Converts a WPS Writer Document AST directly into ISO 32000 PDF file bytes
    pub fn writer_to_pdf(doc: &wps_writer::ast::Document) -> Vec<u8> {
        let mut pdf_doc = PdfDocument::new(&doc.title);
        let mut current_page = PdfPage {
            page_index: 0,
            width: 595.0,
            height: 842.0,
            rotation: 0,
            crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
            text_elements: Vec::new(),
            vector_paths: Vec::new(),
            raw_stream: Vec::new(),
        };

        let mut y = 72.0;

        for section in &doc.sections {
            for element in &section.elements {
                match element {
                    wps_writer::ast::BlockElement::Paragraph(p) => {
                        let text = p.plain_text();
                        let is_heading = p.style_id.starts_with("Heading");
                        let font_size = if is_heading { 16.0 } else { 12.0 };
                        let font_name = if is_heading { "Helvetica-Bold" } else { "Helvetica" };
                        let elem = PdfTextElement {
                            text: text.clone(),
                            bounds: Rect::new(72.0, y, 450.0, font_size),
                            font_name: font_name.to_string(),
                            font_size,
                            color: wps_core::color::Color::rgb(0, 0, 0),
                            baseline: y + font_size * 0.8,
                        };
                        current_page.text_elements.push(elem);

                        let pdf_y = current_page.height - y - font_size;
                        let safe_text = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
                        let chunk = format!(
                            "BT\n/F1 {:.2} Tf\n0 0 0 rg\n1 0 0 1 72.00 {:.2} Tm\n({}) Tj\nET\n",
                            font_size, pdf_y, safe_text
                        );
                        current_page.raw_stream.extend_from_slice(chunk.as_bytes());

                        y += font_size * 1.4;
                        if y > 760.0 {
                            pdf_doc.add_page(current_page);
                            y = 72.0;
                            current_page = PdfPage {
                                page_index: pdf_doc.pages.len(),
                                width: 595.0,
                                height: 842.0,
                                rotation: 0,
                                crop_box: Rect::new(0.0, 0.0, 595.0, 842.0),
                                text_elements: Vec::new(),
                                vector_paths: Vec::new(),
                                raw_stream: Vec::new(),
                            };
                        }
                    }
                    wps_writer::ast::BlockElement::Table(t) => {
                        for row in &t.rows {
                            let mut cell_x = 72.0;
                            for cell in &row.cells {
                                let w = (cell.width_twips.0 as f64 / 20.0).max(60.0);
                                for elem in &cell.elements {
                                    if let wps_writer::ast::BlockElement::Paragraph(p) = elem {
                                        let text = p.plain_text();
                                        current_page.text_elements.push(PdfTextElement {
                                            text: text.clone(),
                                            bounds: Rect::new(cell_x, y, w, 10.0),
                                            font_name: "Helvetica".to_string(),
                                            font_size: 10.0,
                                            color: wps_core::color::Color::rgb(0, 0, 0),
                                            baseline: y + 8.0,
                                        });
                                    }
                                }
                                cell_x += w;
                            }
                            y += 20.0;
                        }
                    }
                    _ => {}
                }
            }
        }

        pdf_doc.add_page(current_page);
        pdf_doc.export_to_bytes()
    }

    /// Converts a WPS Spreadsheets Workbook directly into ISO 32000 PDF file bytes
    pub fn sheets_to_pdf(wb: &Workbook) -> Vec<u8> {
        let mut pdf_doc = PdfDocument::new("Spreadsheet Export");
        for (idx, ws) in wb.sheets.iter().enumerate() {
            let mut page = PdfPage {
                page_index: idx,
                width: 842.0, // Landscape A4 for spreadsheets
                height: 595.0,
                rotation: 0,
                crop_box: Rect::new(0.0, 0.0, 842.0, 595.0),
                text_elements: Vec::new(),
                vector_paths: Vec::new(),
                raw_stream: Vec::new(),
            };

            // Title
            page.text_elements.push(PdfTextElement {
                text: ws.name.clone(),
                bounds: Rect::new(40.0, 40.0, 300.0, 16.0),
                font_name: "Helvetica-Bold".to_string(),
                font_size: 16.0,
                color: wps_core::color::Color::rgb(0, 0, 0),
                baseline: 52.0,
            });

            // Iterate populated cells
            for (coord, cell) in ws.matrix.iter_populated() {
                let x = 40.0 + coord.col as f64 * 70.0;
                let y = 70.0 + coord.row as f64 * 20.0;
                if y < 550.0 && x < 800.0 {
                    let text = match &cell.value {
                        CellValue::Text(s) => s.clone(),
                        CellValue::Number(n) => format!("{:.2}", n),
                        CellValue::Boolean(true) => "TRUE".to_string(),
                        CellValue::Boolean(false) => "FALSE".to_string(),
                        _ => String::new(),
                    };
                    if !text.is_empty() {
                        page.text_elements.push(PdfTextElement {
                            text,
                            bounds: Rect::new(x, y, 65.0, 10.0),
                            font_name: "Helvetica".to_string(),
                            font_size: 10.0,
                            color: wps_core::color::Color::rgb(0, 0, 0),
                            baseline: y + 8.0,
                        });
                    }
                }
            }

            pdf_doc.add_page(page);
        }

        if pdf_doc.pages.is_empty() {
            pdf_doc.add_page(PdfPage {
                page_index: 0,
                width: 842.0,
                height: 595.0,
                rotation: 0,
                crop_box: Rect::new(0.0, 0.0, 842.0, 595.0),
                text_elements: Vec::new(),
                vector_paths: Vec::new(),
                raw_stream: Vec::new(),
            });
        }

        pdf_doc.export_to_bytes()
    }

    /// Converts a WPS Presentation deck directly into ISO 32000 PDF file bytes
    pub fn slides_to_pdf(deck: &PresentationDeck) -> Vec<u8> {
        let mut pdf_doc = PdfDocument::new(&deck.title);
        for (idx, slide) in deck.slides.iter().enumerate() {
            let mut page = PdfPage {
                page_index: idx,
                width: 960.0, // 16:9 widescreen presentation
                height: 540.0,
                rotation: 0,
                crop_box: Rect::new(0.0, 0.0, 960.0, 540.0),
                text_elements: Vec::new(),
                vector_paths: Vec::new(),
                raw_stream: Vec::new(),
            };

            for node in &slide.nodes {
                match node {
                    wps_slides::scene_graph::SlideNode::AutoShape(shape) => {
                        if let Some(t) = &shape.text_frame {
                            let text = t.to_plain_string();
                            page.text_elements.push(PdfTextElement {
                                text,
                                bounds: shape.bounds,
                                font_name: "Helvetica-Bold".to_string(),
                                font_size: 14.0,
                                color: wps_core::color::Color::rgb(0, 0, 0),
                                baseline: shape.bounds.y + 16.0,
                            });
                        }
                    }
                    _ => {}
                }
            }

            pdf_doc.add_page(page);
        }

        if pdf_doc.pages.is_empty() {
            pdf_doc.add_page(PdfPage {
                page_index: 0,
                width: 960.0,
                height: 540.0,
                rotation: 0,
                crop_box: Rect::new(0.0, 0.0, 960.0, 540.0),
                text_elements: Vec::new(),
                vector_paths: Vec::new(),
                raw_stream: Vec::new(),
            });
        }

        pdf_doc.export_to_bytes()
    }

    /// Converts PDF pages into a presentation slide deck
    pub fn to_slides_deck(title: &str, pages: &[PdfPage]) -> PresentationDeck {
        let mut deck = PresentationDeck::new(title);
        deck.slides.clear();

        for (idx, page) in pages.iter().enumerate() {
            let layout = SpatialReconstructor::reconstruct_page(page);
            let slide_id = format!("slide-{}", idx + 1);

            let mut slide_title = format!("Slide {}", idx + 1);
            let mut shapes = Vec::new();

            for (b_idx, block) in layout.blocks.iter().enumerate() {
                if matches!(block.kind, BlockKind::Title | BlockKind::Heading(1)) && slide_title == format!("Slide {}", idx + 1) {
                    slide_title = block.plain_text();
                }

                let shape_id = format!("shape-{}-{}", idx + 1, b_idx + 1);
                let bounds = Rect::new(block.bounds.x, block.bounds.y, block.bounds.width.max(100.0), block.bounds.height.max(30.0));
                let mut shape = ParametricShape::new(&shape_id, ShapeGeometryType::Rectangle, bounds);
                shape.fill = ShapeFill::None;
                shape.stroke = None;
                shape = shape.with_text(&block.plain_text());
                shapes.push(shape);
            }

            let mut slide = Slide::new(&slide_id, &slide_title, "layout-title");
            for s in shapes {
                slide.add_shape(s);
            }
            deck.add_slide(slide);
        }

        if deck.slides.is_empty() {
            deck.add_slide(Slide::new("slide-1", title, "layout-title"));
        }

        deck
    }
}
