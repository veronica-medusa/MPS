use crate::ast::*;
use wps_core::{Color, Point, Rect};
use wps_render::{DisplayList, DrawCommand, FontMetrics};

#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub page_number: usize,
    pub width_px: f64,
    pub height_px: f64,
    pub display_list: DisplayList,
}

pub struct Paginator {
    dpi: f64,
}

impl Paginator {
    pub fn new(dpi: f64) -> Self {
        Self { dpi }
    }

    pub fn paginate(&self, doc: &Document) -> Vec<RenderedPage> {
        let mut pages = Vec::new();
        let mut current_page_num = 1;

        for section in &doc.sections {
            let page_w_px = section.properties.page_dimensions.width.to_px(self.dpi).0;
            let page_h_px = section.properties.page_dimensions.height.to_px(self.dpi).0;

            let margin_top_px = section.properties.margins.top / 20.0 * (self.dpi / 72.0);
            let margin_bottom_px = section.properties.margins.bottom / 20.0 * (self.dpi / 72.0);
            let margin_left_px = section.properties.margins.left / 20.0 * (self.dpi / 72.0);
            let margin_right_px = section.properties.margins.right / 20.0 * (self.dpi / 72.0);

            let content_w_px = page_w_px - margin_left_px - margin_right_px;
            let content_max_y_px = page_h_px - margin_bottom_px;

            let mut current_dl = DisplayList::new();
            let mut cursor_y_px = margin_top_px;

            // Render Page Background
            current_dl.push(DrawCommand::FillRect {
                rect: Rect::new(0.0, 0.0, page_w_px, page_h_px),
                color: Color::WHITE,
            });
            current_dl.push(DrawCommand::StrokeRect {
                rect: Rect::new(0.0, 0.0, page_w_px, page_h_px),
                color: Color::MERCURY_FOG,
                width: 1.0,
            });

            for elem in &section.elements {
                match elem {
                    BlockElement::Paragraph(p) => {
                        let space_before = (p.space_before_twips.0 as f64 / 20.0) * (self.dpi / 72.0);
                        let space_after = (p.space_after_twips.0 as f64 / 20.0) * (self.dpi / 72.0);

                        cursor_y_px += space_before;

                        // Layout paragraph runs
                        for inline in &p.inlines {
                            if let InlineElement::Run(run) = inline {
                                let metrics = FontMetrics::new(&run.font_family, run.size_pt * (self.dpi / 72.0));
                                let lines = metrics.wrap_lines(&run.text, content_w_px);

                                for line in lines {
                                    // Check if line overflows page
                                    if cursor_y_px + line.height_px > content_max_y_px {
                                        // Push finished page and start new page
                                        pages.push(RenderedPage {
                                            page_number: current_page_num,
                                            width_px: page_w_px,
                                            height_px: page_h_px,
                                            display_list: current_dl,
                                        });

                                        current_page_num += 1;
                                        current_dl = DisplayList::new();
                                        cursor_y_px = margin_top_px;

                                        // Render new page background
                                        current_dl.push(DrawCommand::FillRect {
                                            rect: Rect::new(0.0, 0.0, page_w_px, page_h_px),
                                            color: Color::WHITE,
                                        });
                                        current_dl.push(DrawCommand::StrokeRect {
                                            rect: Rect::new(0.0, 0.0, page_w_px, page_h_px),
                                            color: Color::MERCURY_FOG,
                                            width: 1.0,
                                        });
                                    }

                                    // Align X position
                                    let origin_x = match p.alignment {
                                        TextAlignment::Center => {
                                            margin_left_px + (content_w_px - line.width_px) / 2.0
                                        }
                                        TextAlignment::Right => {
                                            margin_left_px + content_w_px - line.width_px
                                        }
                                        _ => margin_left_px,
                                    };

                                    current_dl.push(DrawCommand::DrawTextRun {
                                        text: line.text,
                                        font_family: run.font_family.clone(),
                                        font_size_px: run.size_pt * (self.dpi / 72.0),
                                        bold: run.bold,
                                        italic: run.italic,
                                        color: run.color,
                                        origin: Point::new(origin_x, cursor_y_px + metrics.size_px),
                                    });

                                    cursor_y_px += line.height_px * p.line_spacing_multiple;
                                }
                            }
                        }

                        cursor_y_px += space_after;
                    }
                    BlockElement::Table(table) => {
                        let row_h_px = 24.0 * (self.dpi / 72.0);
                        let min_cols = (table.rows.first().map(|r| r.cells.len()).unwrap_or(1) as f64).max(3.0);
                        let cell_w_px = content_w_px / min_cols;

                        for row in &table.rows {
                            if cursor_y_px + row_h_px > content_max_y_px {
                                pages.push(RenderedPage {
                                    page_number: current_page_num,
                                    width_px: page_w_px,
                                    height_px: page_h_px,
                                    display_list: current_dl,
                                });

                                current_page_num += 1;
                                current_dl = DisplayList::new();
                                cursor_y_px = margin_top_px;

                                current_dl.push(DrawCommand::FillRect {
                                    rect: Rect::new(0.0, 0.0, page_w_px, page_h_px),
                                    color: Color::WHITE,
                                });
                            }

                            for (col_idx, cell) in row.cells.iter().enumerate() {
                                let cell_x = margin_left_px + col_idx as f64 * cell_w_px;
                                let cell_rect = Rect::new(cell_x, cursor_y_px, cell_w_px, row_h_px);

                                if let Some(bg) = cell.background_color {
                                    current_dl.push(DrawCommand::FillRect { rect: cell_rect, color: bg });
                                }
                                current_dl.push(DrawCommand::StrokeRect {
                                    rect: cell_rect,
                                    color: Color::MERCURY_FOG,
                                    width: 1.0,
                                });
                            }
                            cursor_y_px += row_h_px;
                        }
                    }
                    _ => {}
                }
            }

            // Push final page
            pages.push(RenderedPage {
                page_number: current_page_num,
                width_px: page_w_px,
                height_px: page_h_px,
                display_list: current_dl,
            });
        }

        pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginator() {
        let mut doc = Document::new("Test Doc", "Author");
        let mut p = Paragraph::new("p1");
        p.add_run(TextRun::new("Hello OpenWPS Document Pagination"));
        doc.add_paragraph(p);

        let paginator = Paginator::new(96.0);
        let pages = paginator.paginate(&doc);

        assert_eq!(pages.len(), 1);
        assert!(pages[0].width_px > 0.0);
        assert!(!pages[0].display_list.is_empty());
    }
}
