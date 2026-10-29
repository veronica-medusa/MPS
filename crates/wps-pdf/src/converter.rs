use wps_core::geometry::Rect;
use wps_core::units::Twips;
use wps_sheets::{CellCoordinate, CellValue, SparseMatrix};
use wps_slides::scene_graph::{PresentationDeck, Slide};
use wps_slides::shape::{ParametricShape, ShapeFill, ShapeGeometryType};
use crate::parser::PdfPage;
use crate::spatial::{BlockKind, SpatialReconstructor};

pub struct PdfConverter;

impl PdfConverter {
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
                    BlockKind::Paragraph | BlockKind::TableCell => {
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
                    let val = if let Ok(n) = cell.text.trim().parse::<f64>() {
                        CellValue::Number(n)
                    } else if cell.text.trim().is_empty() {
                        CellValue::Empty
                    } else {
                        CellValue::Text(cell.text.trim().to_string())
                    };

                    matrix.set_value(coord, val);
                }

                current_row += table.rows as u32 + 2; // Separate tables by 2 blank rows
            }
        }

        matrix
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
