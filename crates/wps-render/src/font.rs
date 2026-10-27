use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;
use wps_core::{Point, Rect};

#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub text: String,
    pub width_px: f64,
    pub height_px: f64,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub family: String,
    pub size_px: f64,
    pub line_height_px: f64,
    pub char_aspect_ratio: f64, // average width/height ratio
}

impl FontMetrics {
    pub fn new(family: &str, size_px: f64) -> Self {
        let char_aspect_ratio = if family.contains("Mono") {
            0.60
        } else {
            0.52
        };
        Self {
            family: family.to_string(),
            size_px,
            line_height_px: size_px * 1.35,
            char_aspect_ratio,
        }
    }

    pub fn measure_char(&self, ch: char) -> f64 {
        let width = UnicodeWidthChar::width(ch).unwrap_or(1);
        width as f64 * self.size_px * self.char_aspect_ratio
    }

    pub fn measure_str(&self, text: &str) -> f64 {
        text.chars().map(|c| self.measure_char(c)).sum()
    }

    /// Breaks a paragraph into visual lines given available width
    pub fn wrap_lines(&self, text: &str, max_width_px: f64) -> Vec<TextLine> {
        let mut lines = Vec::new();
        if text.is_empty() {
            lines.push(TextLine {
                text: String::new(),
                width_px: 0.0,
                height_px: self.line_height_px,
                byte_start: 0,
                byte_end: 0,
            });
            return lines;
        }

        let words = text.split_word_bound_indices();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut line_byte_start = 0;
        let mut last_end = 0;

        for (byte_idx, word) in words {
            let word_w = self.measure_str(word);
            if current_width + word_w > max_width_px && !current_line.is_empty() {
                lines.push(TextLine {
                    text: current_line.clone(),
                    width_px: current_width,
                    height_px: self.line_height_px,
                    byte_start: line_byte_start,
                    byte_end: last_end,
                });
                current_line.clear();
                line_byte_start = byte_idx;
                current_width = 0.0;
            }

            current_line.push_str(word);
            current_width += word_w;
            last_end = byte_idx + word.len();
        }

        if !current_line.is_empty() {
            lines.push(TextLine {
                text: current_line,
                width_px: current_width,
                height_px: self.line_height_px,
                byte_start: line_byte_start,
                byte_end: last_end,
            });
        }

        lines
    }

    /// Resolves caret coordinate given character offset
    pub fn caret_offset_to_point(&self, text: &str, offset: usize, origin: Point) -> Point {
        let prefix: String = text.chars().take(offset).collect();
        let x = origin.x + self.measure_str(&prefix);
        let y = origin.y;
        Point::new(x, y)
    }

    /// Resolves character offset given hit coordinate
    pub fn hit_test(&self, text: &str, click_x: f64, origin_x: f64) -> usize {
        let target_x = click_x - origin_x;
        if target_x <= 0.0 {
            return 0;
        }

        let mut acc_x = 0.0;
        for (i, ch) in text.chars().enumerate() {
            let w = self.measure_char(ch);
            if acc_x + w / 2.0 >= target_x {
                return i;
            }
            acc_x += w;
        }

        text.chars().count()
    }

    /// Computes selection bounding boxes for a character range
    pub fn selection_rect(&self, text: &str, start_char: usize, end_char: usize, origin: Point) -> Rect {
        let (min_c, max_c) = if start_char <= end_char {
            (start_char, end_char)
        } else {
            (end_char, start_char)
        };

        let before: String = text.chars().take(min_c).collect();
        let selected: String = text.chars().skip(min_c).take(max_c - min_c).collect();

        let start_x = origin.x + self.measure_str(&before);
        let width = self.measure_str(&selected);

        Rect::new(start_x, origin.y - self.size_px, width, self.line_height_px)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_wrap_and_hit_test() {
        let metrics = FontMetrics::new("Inter", 16.0);
        let text = "OpenWPS Mercury Design System High Performance Document Processing";
        let lines = metrics.wrap_lines(text, 200.0);

        assert!(lines.len() > 1);

        let hit = metrics.hit_test("Hello", 20.0, 0.0);
        assert!(hit > 0 && hit <= 5);
    }
}
