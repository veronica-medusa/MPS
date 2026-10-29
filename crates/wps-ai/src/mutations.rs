use serde::{Deserialize, Serialize};
use wps_core::geometry::Rect;
use wps_sheets::cell::{CellCoordinate, CellValue};
use wps_sheets::sparse_matrix::SparseMatrix;
use wps_writer::ast::{BlockElement, Document as WriterDocument, Paragraph, TextRun};
use wps_slides::scene_graph::{PresentationDeck as SlidesDeck, Slide};
use wps_slides::shape::{ParametricShape, ShapeFill, ShapeGeometryType};
use wps_pdf::{Annotation, PdfDocument};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiMutation {
    WriterInsertParagraph {
        text: String,
        is_heading: bool,
        heading_level: usize,
    },
    WriterReplaceSelection {
        original_snippet: String,
        replacement_text: String,
    },
    SheetsSetFormula {
        coord: CellCoordinate,
        formula: String,
    },
    SheetsInsertTable {
        start_coord: CellCoordinate,
        headers: Vec<String>,
        rows: Vec<Vec<CellValue>>,
    },
    SlidesAddDeckSlide {
        title: String,
        bullet_points: Vec<String>,
    },
    PdfHighlightCitation {
        page_index: usize,
        bbox: Rect,
        citation_text: String,
    },
}

pub struct MutationApplicator;

impl MutationApplicator {
    pub fn apply_to_writer(doc: &mut WriterDocument, mutation: &AiMutation) -> bool {
        match mutation {
            AiMutation::WriterInsertParagraph { text, is_heading, heading_level } => {
                let id = format!("ai-p-{}", doc.total_word_count() + 1);
                let p = if *is_heading {
                    Paragraph::heading(&id, *heading_level, text)
                } else {
                    let mut para = Paragraph::new(&id);
                    para.add_run(TextRun::new(text));
                    para
                };
                doc.add_paragraph(p);
                true
            }
            AiMutation::WriterReplaceSelection { original_snippet, replacement_text } => {
                let mut replaced = false;
                for sec in &mut doc.sections {
                    for elem in &mut sec.elements {
                        if let BlockElement::Paragraph(p) = elem {
                            for inline in &mut p.inlines {
                                if let wps_writer::ast::InlineElement::Run(run) = inline {
                                    if run.text.contains(original_snippet) {
                                        run.text = run.text.replace(original_snippet, replacement_text);
                                        replaced = true;
                                    }
                                }
                            }
                        }
                    }
                }
                replaced
            }
            _ => false,
        }
    }

    pub fn apply_to_sheets(matrix: &mut SparseMatrix, mutation: &AiMutation) -> bool {
        match mutation {
            AiMutation::SheetsSetFormula { coord, formula } => {
                matrix.set_formula(*coord, Some(formula.clone()));
                true
            }
            AiMutation::SheetsInsertTable { start_coord, headers, rows } => {
                // Insert headers
                for (col_offset, h) in headers.iter().enumerate() {
                    let c = CellCoordinate::new(start_coord.row, start_coord.col + col_offset as u16);
                    matrix.set_value(c, CellValue::Text(h.clone()));
                }
                // Insert rows
                for (row_offset, row) in rows.iter().enumerate() {
                    for (col_offset, val) in row.iter().enumerate() {
                        let c = CellCoordinate::new(start_coord.row + 1 + row_offset as u32, start_coord.col + col_offset as u16);
                        matrix.set_value(c, val.clone());
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub fn apply_to_slides(deck: &mut SlidesDeck, mutation: &AiMutation) -> bool {
        match mutation {
            AiMutation::SlidesAddDeckSlide { title, bullet_points } => {
                let slide_id = format!("slide-{}", deck.slides.len() + 1);
                let mut slide = Slide::new(&slide_id, title, "layout-content");

                // Title shape
                let mut title_shape = ParametricShape::new(
                    &format!("{}-title", slide_id),
                    ShapeGeometryType::Rectangle,
                    Rect::new(50.0, 40.0, 860.0, 60.0),
                );
                title_shape.fill = ShapeFill::None;
                title_shape.stroke = None;
                title_shape = title_shape.with_text(title);
                slide.add_shape(title_shape);

                // Body bullets shape
                let bullets_text = bullet_points.iter().map(|b| format!("• {}", b)).collect::<Vec<_>>().join("\n");
                let mut body_shape = ParametricShape::new(
                    &format!("{}-body", slide_id),
                    ShapeGeometryType::Rectangle,
                    Rect::new(50.0, 120.0, 860.0, 360.0),
                );
                body_shape.fill = ShapeFill::None;
                body_shape.stroke = None;
                body_shape = body_shape.with_text(&bullets_text);
                slide.add_shape(body_shape);

                deck.add_slide(slide);
                true
            }
            _ => false,
        }
    }

    pub fn apply_to_pdf(pdf: &mut PdfDocument, mutation: &AiMutation) -> bool {
        match mutation {
            AiMutation::PdfHighlightCitation { page_index, bbox, citation_text } => {
                let annot = Annotation::highlight(
                    &format!("ai-cite-{}", citation_text.len()),
                    *page_index,
                    *bbox,
                );
                pdf.add_annotation(*page_index, annot);
                true
            }
            _ => false,
        }
    }
}
