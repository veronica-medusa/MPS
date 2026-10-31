use std::collections::HashMap;
use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use wps_render::{DisplayList, DrawCommand};
use crate::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutRun {
    pub text: String,
    pub font_family: String,
    pub font_size_pt: f64,
    pub bold: bool,
    pub italic: bool,
    pub color: Color,
    pub bounds: Rect,
    pub baseline: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutLine {
    pub x_pt: f64,
    pub y_pt: f64,
    pub width_pt: f64,
    pub height_pt: f64,
    pub baseline_pt: f64,
    pub runs: Vec<LaidOutRun>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutTableCell {
    pub bounds: Rect,
    pub text: String,
    pub lines: Vec<LaidOutLine>,
    pub background: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutTableRow {
    pub bounds: Rect,
    pub cells: Vec<LaidOutTableCell>,
    pub is_header: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaidOutItem {
    Line(LaidOutLine),
    TableRows(Vec<LaidOutTableRow>),
    FloatingImage { bounds: Rect, media_id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidOutPage {
    pub page_number: usize,
    pub width_pt: f64,
    pub height_pt: f64,
    pub margin_left_pt: f64,
    pub margin_top_pt: f64,
    pub margin_right_pt: f64,
    pub margin_bottom_pt: f64,
    pub items: Vec<LaidOutItem>,
}

impl LaidOutPage {
    pub fn render_to_display_list(&self, dl: &mut DisplayList) {
        // Page background sheet with paper shadow
        dl.push(DrawCommand::FillRect {
            rect: Rect::new(0.0, 0.0, self.width_pt, self.height_pt),
            color: Color::WHITE,
        });

        // 1px subtle page border
        dl.push(DrawCommand::StrokeRect {
            rect: Rect::new(0.0, 0.0, self.width_pt, self.height_pt),
            color: Color::rgb(222, 226, 230),
            width: 1.0,
        });

        // Render page items
        for item in &self.items {
            match item {
                LaidOutItem::Line(line) => {
                    for run in &line.runs {
                        dl.push(DrawCommand::DrawTextRun {
                            text: run.text.clone(),
                            font_family: run.font_family.clone(),
                            font_size_px: run.font_size_pt * (96.0 / 72.0),
                            bold: run.bold,
                            italic: run.italic,
                            color: run.color,
                            origin: Point::new(run.bounds.x, run.baseline),
                        });
                    }
                }
                LaidOutItem::TableRows(rows) => {
                    for row in rows {
                        for cell in &row.cells {
                            if let Some(bg) = cell.background {
                                dl.push(DrawCommand::FillRect {
                                    rect: cell.bounds,
                                    color: bg,
                                });
                            }
                            // Cell borders
                            dl.push(DrawCommand::StrokeRect {
                                rect: cell.bounds,
                                color: Color::rgb(206, 212, 218),
                                width: 1.0,
                            });
                            // Cell text lines
                            for line in &cell.lines {
                                for run in &line.runs {
                                    dl.push(DrawCommand::DrawTextRun {
                                        text: run.text.clone(),
                                        font_family: run.font_family.clone(),
                                        font_size_px: run.font_size_pt * (96.0 / 72.0),
                                        bold: run.bold,
                                        italic: run.italic,
                                        color: run.color,
                                        origin: Point::new(run.bounds.x, run.baseline),
                                    });
                                }
                            }
                        }
                    }
                }
                LaidOutItem::FloatingImage { bounds, .. } => {
                    dl.push(DrawCommand::FillRect {
                        rect: *bounds,
                        color: Color::rgb(233, 236, 239),
                    });
                    dl.push(DrawCommand::StrokeRect {
                        rect: *bounds,
                        color: Color::rgb(173, 181, 189),
                        width: 1.0,
                    });
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Paginator {
    // Cache for laid out paragraph lines by ParagraphId
    #[allow(dead_code)]
    line_cache: HashMap<String, Vec<LaidOutLine>>,
}

impl Paginator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Paginates an entire Document AST into a sequence of LaidOutPage structs
    pub fn paginate_document(&mut self, doc: &Document) -> Vec<LaidOutPage> {
        let mut pages = Vec::new();

        for section in &doc.sections {
            let page_w_pt = section.page_setup.width_twips as f64 / 20.0;
            let page_h_pt = section.page_setup.height_twips as f64 / 20.0;
            let margin_l = section.page_setup.margins.left_twips as f64 / 20.0;
            let margin_t = section.page_setup.margins.top_twips as f64 / 20.0;
            let margin_r = section.page_setup.margins.right_twips as f64 / 20.0;
            let margin_b = section.page_setup.margins.bottom_twips as f64 / 20.0;

            let content_w = page_w_pt - margin_l - margin_r;
            let _content_h = page_h_pt - margin_t - margin_b;

            let mut current_page = LaidOutPage {
                page_number: pages.len() + 1,
                width_pt: page_w_pt,
                height_pt: page_h_pt,
                margin_left_pt: margin_l,
                margin_top_pt: margin_t,
                margin_right_pt: margin_r,
                margin_bottom_pt: margin_b,
                items: Vec::new(),
            };

            let mut cur_y = margin_t;

            for block in &section.elements {
                match block {
                    BlockNode::Paragraph(p) => {
                        let lines = self.layout_paragraph(p, doc, content_w, margin_l);

                        let space_before = p.space_before_twips.0 as f64 / 20.0;
                        let space_after = p.space_after_twips.0 as f64 / 20.0;

                        // Check page_break_before
                        if p.direct_props.page_break_before == Some(true) && !current_page.items.is_empty() {
                            pages.push(current_page);
                            current_page = LaidOutPage {
                                page_number: pages.len() + 1,
                                width_pt: page_w_pt,
                                height_pt: page_h_pt,
                                margin_left_pt: margin_l,
                                margin_top_pt: margin_t,
                                margin_right_pt: margin_r,
                                margin_bottom_pt: margin_b,
                                items: Vec::new(),
                            };
                            cur_y = margin_t;
                        }

                        cur_y += space_before;

                        // Flow lines into pages
                        let mut line_idx = 0;
                        while line_idx < lines.len() {
                            let line = &lines[line_idx];

                            // Check overflow
                            if cur_y + line.height_pt > (page_h_pt - margin_b) && !current_page.items.is_empty() {
                                // Widow/orphan control
                                if p.direct_props.widow_control.unwrap_or(true) && lines.len() > 1 && line_idx == lines.len() - 1 {
                                    // Move previous line as well to avoid 1 orphan line at top of next page
                                    if let Some(LaidOutItem::Line(prev_line)) = current_page.items.pop() {
                                        pages.push(current_page);
                                        current_page = LaidOutPage {
                                            page_number: pages.len() + 1,
                                            width_pt: page_w_pt,
                                            height_pt: page_h_pt,
                                            margin_left_pt: margin_l,
                                            margin_top_pt: margin_t,
                                            margin_right_pt: margin_r,
                                            margin_bottom_pt: margin_b,
                                            items: Vec::new(),
                                        };
                                        cur_y = margin_t;
                                        let mut shifted = prev_line.clone();
                                        shifted.y_pt = cur_y;
                                        cur_y += shifted.height_pt;
                                        current_page.items.push(LaidOutItem::Line(shifted));
                                    }
                                } else {
                                    pages.push(current_page);
                                    current_page = LaidOutPage {
                                        page_number: pages.len() + 1,
                                        width_pt: page_w_pt,
                                        height_pt: page_h_pt,
                                        margin_left_pt: margin_l,
                                        margin_top_pt: margin_t,
                                        margin_right_pt: margin_r,
                                        margin_bottom_pt: margin_b,
                                        items: Vec::new(),
                                    };
                                    cur_y = margin_t;
                                }
                            }

                            let mut shifted_line = line.clone();
                            shifted_line.y_pt = cur_y;
                            for r in &mut shifted_line.runs {
                                r.bounds.y = cur_y;
                                r.baseline = cur_y + shifted_line.baseline_pt;
                            }

                            cur_y += shifted_line.height_pt;
                            current_page.items.push(LaidOutItem::Line(shifted_line));
                            line_idx += 1;
                        }

                        cur_y += space_after;
                    }
                    BlockNode::Table(table) => {
                        let laid_out_table_rows = self.layout_table(table, content_w, margin_l);
                        let header_row = laid_out_table_rows.iter().find(|r| r.is_header).cloned();

                        let mut current_table_slice: Vec<LaidOutTableRow> = Vec::new();

                        for row in laid_out_table_rows {
                            let row_h = row.bounds.height;

                            // Check overflow
                            if cur_y + row_h > (page_h_pt - margin_b) && !current_page.items.is_empty() {
                                if !current_table_slice.is_empty() {
                                    current_page.items.push(LaidOutItem::TableRows(std::mem::take(&mut current_table_slice)));
                                }

                                pages.push(current_page);
                                current_page = LaidOutPage {
                                    page_number: pages.len() + 1,
                                    width_pt: page_w_pt,
                                    height_pt: page_h_pt,
                                    margin_left_pt: margin_l,
                                    margin_top_pt: margin_t,
                                    margin_right_pt: margin_r,
                                    margin_bottom_pt: margin_b,
                                    items: Vec::new(),
                                };
                                cur_y = margin_t;

                                // TABLE HEADER REPEATING MECHANIC:
                                // If the table has an is_header_repeat row, clone it at the top of the new page!
                                if let Some(ref hdr) = header_row {
                                    let mut repeated = hdr.clone();
                                    repeated.bounds.y = cur_y;
                                    for c in &mut repeated.cells {
                                        c.bounds.y = cur_y;
                                        for l in &mut c.lines {
                                            l.y_pt = cur_y + 4.0;
                                            for r in &mut l.runs {
                                                r.bounds.y = cur_y + 4.0;
                                                r.baseline = cur_y + 4.0 + l.baseline_pt;
                                            }
                                        }
                                    }
                                    cur_y += repeated.bounds.height;
                                    current_table_slice.push(repeated);
                                }
                            }

                            let mut shifted_row = row.clone();
                            shifted_row.bounds.y = cur_y;
                            for c in &mut shifted_row.cells {
                                c.bounds.y = cur_y;
                                for l in &mut c.lines {
                                    l.y_pt = cur_y + 4.0;
                                    for r in &mut l.runs {
                                        r.bounds.y = cur_y + 4.0;
                                        r.baseline = cur_y + 4.0 + l.baseline_pt;
                                    }
                                }
                            }

                            cur_y += shifted_row.bounds.height;
                            current_table_slice.push(shifted_row);
                        }

                        if !current_table_slice.is_empty() {
                            current_page.items.push(LaidOutItem::TableRows(current_table_slice));
                        }
                    }
                    BlockNode::FloatingObject(fo) => {
                        let fo_w = fo.bounds_twips.width / 20.0;
                        let fo_h = fo.bounds_twips.height / 20.0;
                        let item = LaidOutItem::FloatingImage {
                            bounds: Rect::new(margin_l, cur_y, fo_w, fo_h),
                            media_id: fo.media_id.clone(),
                        };
                        current_page.items.push(item);
                        cur_y += fo_h + 10.0;
                    }
                }
            }

            if !current_page.items.is_empty() {
                pages.push(current_page);
            }
        }

        if pages.is_empty() {
            pages.push(LaidOutPage {
                page_number: 1,
                width_pt: 595.0,
                height_pt: 842.0,
                margin_left_pt: 72.0,
                margin_top_pt: 72.0,
                margin_right_pt: 72.0,
                margin_bottom_pt: 72.0,
                items: Vec::new(),
            });
        }

        pages
    }

    /// Layouts lines of a single paragraph
    pub fn layout_paragraph(&mut self, p: &ParagraphNode, doc: &Document, max_w: f64, left_x: f64) -> Vec<LaidOutLine> {
        let default_font_size = 11.0;
        let char_w = default_font_size * 0.55;
        let line_h = default_font_size * p.line_spacing_multiple.max(1.0) * 1.3;

        let plain_text = p.plain_text();
        if plain_text.is_empty() {
            return vec![LaidOutLine {
                x_pt: left_x,
                y_pt: 0.0,
                width_pt: 0.0,
                height_pt: line_h,
                baseline_pt: line_h * 0.8,
                runs: Vec::new(),
            }];
        }

        let words: Vec<&str> = plain_text.split_whitespace().collect();
        let mut lines = Vec::new();
        let mut cur_words: Vec<&str> = Vec::new();
        let mut cur_w = 0.0;

        let style = doc.styles.get_style(&p.style_id);
        let font_family = style.and_then(|s| s.run_style.font_family.clone()).unwrap_or_else(|| "Inter".to_string());
        let font_size = style.and_then(|s| s.run_style.font_size_half_pts).map(|h| h as f64 / 2.0).unwrap_or(default_font_size);
        let bold = style.and_then(|s| s.run_style.bold).unwrap_or(p.style_id.contains("Heading"));
        let color = style.and_then(|s| s.run_style.color).unwrap_or(Color::rgb(23, 23, 33));

        for word in words {
            let w_len = word.chars().count() as f64 * char_w;
            let space_w = char_w;

            if !cur_words.is_empty() && cur_w + space_w + w_len > max_w {
                let line_str = cur_words.join(" ");
                let line_width = line_str.chars().count() as f64 * char_w;

                let align_x = match p.alignment {
                    TextAlignment::Center => left_x + (max_w - line_width).max(0.0) * 0.5,
                    TextAlignment::Right => left_x + (max_w - line_width).max(0.0),
                    _ => left_x,
                };

                let run = LaidOutRun {
                    text: line_str,
                    font_family: font_family.clone(),
                    font_size_pt: font_size,
                    bold,
                    italic: false,
                    color,
                    bounds: Rect::new(align_x, 0.0, line_width, font_size),
                    baseline: font_size * 0.8,
                };

                lines.push(LaidOutLine {
                    x_pt: align_x,
                    y_pt: 0.0,
                    width_pt: line_width,
                    height_pt: line_h,
                    baseline_pt: font_size * 0.8,
                    runs: vec![run],
                });

                cur_words.clear();
                cur_w = 0.0;
            }

            if !cur_words.is_empty() {
                cur_w += space_w;
            }
            cur_words.push(word);
            cur_w += w_len;
        }

        if !cur_words.is_empty() {
            let line_str = cur_words.join(" ");
            let line_width = line_str.chars().count() as f64 * char_w;

            let align_x = match p.alignment {
                TextAlignment::Center => left_x + (max_w - line_width).max(0.0) * 0.5,
                TextAlignment::Right => left_x + (max_w - line_width).max(0.0),
                _ => left_x,
            };

            let run = LaidOutRun {
                text: line_str,
                font_family,
                font_size_pt: font_size,
                bold,
                italic: false,
                color,
                bounds: Rect::new(align_x, 0.0, line_width, font_size),
                baseline: font_size * 0.8,
            };

            lines.push(LaidOutLine {
                x_pt: align_x,
                y_pt: 0.0,
                width_pt: line_width,
                height_pt: line_h,
                baseline_pt: font_size * 0.8,
                runs: vec![run],
            });
        }

        lines
    }

    /// Layouts rows and cells of a table
    pub fn layout_table(&mut self, table: &TableNode, max_w: f64, left_x: f64) -> Vec<LaidOutTableRow> {
        let mut laid_out_rows = Vec::new();
        let num_cols = table.grid_columns_twips.len().max(1);
        let col_w = max_w / num_cols as f64;

        let row_h = 24.0;

        for (r_idx, row) in table.rows.iter().enumerate() {
            let is_header = row.is_header || row.is_header_repeat || r_idx == 0;
            let mut laid_out_cells = Vec::new();

            for (c_idx, cell) in row.cells.iter().enumerate() {
                let cell_x = left_x + c_idx as f64 * col_w;
                let cell_rect = Rect::new(cell_x, 0.0, col_w, row_h);

                let cell_text = cell.elements.iter().map(|e| e.plain_text()).collect::<Vec<_>>().join(" ");
                let run = LaidOutRun {
                    text: cell_text.clone(),
                    font_family: "Inter".to_string(),
                    font_size_pt: 10.0,
                    bold: is_header,
                    italic: false,
                    color: Color::rgb(23, 23, 33),
                    bounds: Rect::new(cell_x + 4.0, 4.0, col_w - 8.0, 10.0),
                    baseline: 14.0,
                };

                let line = LaidOutLine {
                    x_pt: cell_x + 4.0,
                    y_pt: 4.0,
                    width_pt: col_w - 8.0,
                    height_pt: 14.0,
                    baseline_pt: 10.0,
                    runs: vec![run],
                };

                laid_out_cells.push(LaidOutTableCell {
                    bounds: cell_rect,
                    text: cell_text,
                    lines: vec![line],
                    background: if is_header {
                        Some(Color::rgb(241, 243, 245))
                    } else {
                        cell.background_color
                    },
                });
            }

            laid_out_rows.push(LaidOutTableRow {
                bounds: Rect::new(left_x, 0.0, max_w, row_h),
                cells: laid_out_cells,
                is_header,
            });
        }

        laid_out_rows
    }
}
