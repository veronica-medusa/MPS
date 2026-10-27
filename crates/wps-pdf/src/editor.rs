use wps_core::color::Color;
use wps_core::geometry::{Point, Rect};
use crate::parser::{PdfPage, PdfTextElement};
use crate::spatial::{SpatialReconstructor, TextLine};

#[derive(Debug, Clone)]
pub struct PdfEditSession {
    pub page_index: usize,
    pub target_block_index: usize,
    pub original_bounds: Rect,
    pub current_text: String,
    pub cursor_pos: usize,
    pub font_name: String,
    pub font_size: f64,
    pub color: Color,
}

impl PdfEditSession {
    pub fn new(page: &PdfPage, click_point: Point) -> Option<Self> {
        let layout = SpatialReconstructor::reconstruct_page(page);

        for (idx, block) in layout.blocks.iter().enumerate() {
            if click_point.x >= block.bounds.x && click_point.x <= block.bounds.x + block.bounds.width &&
               click_point.y >= block.bounds.y && click_point.y <= block.bounds.y + block.bounds.height {
                let text = block.plain_text();
                let cursor_pos = text.len();

                return Some(Self {
                    page_index: page.page_index,
                    target_block_index: idx,
                    original_bounds: block.bounds,
                    current_text: text,
                    cursor_pos,
                    font_name: block.font_name.clone(),
                    font_size: block.font_size,
                    color: Color::rgb(0, 0, 0),
                });
            }
        }

        None
    }

    pub fn insert_text(&mut self, text: &str) {
        if self.cursor_pos >= self.current_text.len() {
            self.current_text.push_str(text);
        } else {
            self.current_text.insert_str(self.cursor_pos, text);
        }
        self.cursor_pos += text.len();
    }

    pub fn delete_backwards(&mut self, count: usize) {
        for _ in 0..count {
            if self.cursor_pos > 0 && !self.current_text.is_empty() {
                self.cursor_pos -= 1;
                self.current_text.remove(self.cursor_pos);
            }
        }
    }

    /// Reflows the current text across the original paragraph bounds with line-breaking
    pub fn reflow_lines(&self) -> Vec<TextLine> {
        let max_width = self.original_bounds.width.max(100.0);
        let char_w = self.font_size * 0.55;
        let line_height = self.font_size * 1.3;

        let words = self.current_text.split_whitespace().collect::<Vec<_>>();
        let mut lines = Vec::new();

        let mut current_line_words: Vec<&str> = Vec::new();
        let mut current_line_len = 0.0;
        let mut cur_y = self.original_bounds.y;

        for word in words {
            let word_w = word.chars().count() as f64 * char_w;
            let space_w = char_w;

            if !current_line_words.is_empty() && (current_line_len + space_w + word_w) > max_width {
                // Emit current line
                let line_str = current_line_words.join(" ");
                let line_w = line_str.chars().count() as f64 * char_w;
                let line_rect = Rect::new(self.original_bounds.x, cur_y, line_w, self.font_size);

                lines.push(TextLine {
                    text: line_str.clone(),
                    bounds: line_rect,
                    font_name: self.font_name.clone(),
                    font_size: self.font_size,
                    color: self.color,
                    elements: vec![PdfTextElement {
                        text: line_str,
                        bounds: line_rect,
                        font_name: self.font_name.clone(),
                        font_size: self.font_size,
                        color: self.color,
                        baseline: cur_y + self.font_size * 0.8,
                    }],
                });

                cur_y += line_height;
                current_line_words.clear();
                current_line_len = 0.0;
            }

            if !current_line_words.is_empty() {
                current_line_len += space_w;
            }
            current_line_words.push(word);
            current_line_len += word_w;
        }

        if !current_line_words.is_empty() {
            let line_str = current_line_words.join(" ");
            let line_w = line_str.chars().count() as f64 * char_w;
            let line_rect = Rect::new(self.original_bounds.x, cur_y, line_w, self.font_size);

            lines.push(TextLine {
                text: line_str.clone(),
                bounds: line_rect,
                font_name: self.font_name.clone(),
                font_size: self.font_size,
                color: self.color,
                elements: vec![PdfTextElement {
                    text: line_str,
                    bounds: line_rect,
                    font_name: self.font_name.clone(),
                    font_size: self.font_size,
                    color: self.color,
                    baseline: cur_y + self.font_size * 0.8,
                }],
            });
        }

        lines
    }

    /// Applies the edit back onto the PdfPage, updating its elements and generating replacement PDF content stream
    pub fn commit(self, page: &mut PdfPage) {
        let reflowed_lines = self.reflow_lines();

        // 1. Remove old elements in target bounding box
        page.text_elements.retain(|elem| {
            let cx = elem.bounds.x + elem.bounds.width * 0.5;
            let cy = elem.bounds.y + elem.bounds.height * 0.5;
            !(cx >= self.original_bounds.x && cx <= self.original_bounds.x + self.original_bounds.width &&
              cy >= self.original_bounds.y && cy <= self.original_bounds.y + self.original_bounds.height)
        });

        // 2. Add reflowed text elements
        for line in &reflowed_lines {
            page.text_elements.extend(line.elements.clone());
        }

        // 3. Regenerate PDF content stream with replacement text blocks
        let mut new_stream = Vec::new();
        for elem in &page.text_elements {
            let pdf_y = page.height - elem.bounds.y - elem.bounds.height;
            let safe_text = elem.text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");

            let r = elem.color.r as f64 / 255.0;
            let g = elem.color.g as f64 / 255.0;
            let b = elem.color.b as f64 / 255.0;

            let chunk = format!(
                "BT\n/F1 {:.2} Tf\n{:.3} {:.3} {:.3} rg\n1 0 0 1 {:.2} {:.2} Tm\n({}) Tj\nET\n",
                elem.font_size, r, g, b, elem.bounds.x, pdf_y, safe_text
            );
            new_stream.extend_from_slice(chunk.as_bytes());
        }

        page.raw_stream = new_stream;
    }
}
