//! Spatial Layout Reconstruction Engine (`wps-pdf`)
//!
//! Reconstructs raw ISO 32000 positioned glyph runs and vector strokes into
//! structured `EditableParagraphBlock`s, multi-column reading streams, and
//! `ReconstructedTableGrid`s (both ruled vector tables and borderless whitespace-gutter tables).
//!
//! Powers:
//! 1. "Edit PDF Like Word" inline paragraph reflow & editing
//! 2. Native AST conversion: PDF -> `.docx` (Writer), `.xlsx` (Spreadsheets), `.pptx` (Slides)

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Axis-aligned bounding box in normalized page points (origin `(0, 0)` at top-left,
/// `x` increases rightwards, `y` increases downwards).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PdfRect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl PdfRect {
    #[inline]
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        (self.x1 - self.x0).max(0.0)
    }

    #[inline]
    pub fn height(&self) -> f32 {
        (self.y1 - self.y0).max(0.0)
    }

    #[inline]
    pub fn center_x(&self) -> f32 {
        (self.x0 + self.x1) * 0.5
    }

    #[inline]
    pub fn center_y(&self) -> f32 {
        (self.y0 + self.y1) * 0.5
    }

    #[inline]
    pub fn union(&self, other: &Self) -> Self {
        Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    #[inline]
    pub fn contains_point(&self, x: f32, y: f32, tolerance: f32) -> bool {
        x >= self.x0 - tolerance
            && x <= self.x1 + tolerance
            && y >= self.y0 - tolerance
            && y <= self.y1 + tolerance
    }

    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.x0 < other.x1 && self.x1 > other.x0 && self.y0 < other.y1 && self.y1 > other.y0
    }

    #[inline]
    pub fn horizontal_overlap_ratio(&self, other: &Self) -> f32 {
        let overlap = (self.x1.min(other.x1) - self.x0.max(other.x0)).max(0.0);
        let min_w = self.width().min(other.width()).max(1.0);
        overlap / min_w
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for RgbaColor {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }
}

/// Positioned glyph run extracted by the ISO 32000 content-stream interpreter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionedGlyphRun {
    pub id: u64,
    pub text: String,
    pub bbox: PdfRect,
    pub baseline_y: f32,
    pub font_name: String,
    pub font_size_pt: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub color: RgbaColor,
    /// Byte range inside the decompressed page content stream for direct inline editing/redaction.
    pub stream_byte_range: Option<(usize, usize)>,
}

/// 2D line segment extracted from PDF vector paths (`m`/`l`/`re` operators).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VectorStrokeSegment {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub stroke_width_pt: f32,
    pub color: RgbaColor,
}

/// Filled rectangle extracted from PDF vector paths (used for table cell shading or thin rule lines).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VectorFilledRect {
    pub rect: PdfRect,
    pub fill_color: RgbaColor,
}

/// Raw interpreted scene for a single PDF page.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageScene {
    pub page_index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub glyph_runs: Vec<PositionedGlyphRun>,
    pub stroke_segments: Vec<VectorStrokeSegment>,
    pub filled_rects: Vec<VectorFilledRect>,
}

// ============================================================================
// OUTPUT AST STRUCTURES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconstructedWord {
    pub text: String,
    pub bbox: PdfRect,
    pub baseline_y: f32,
    pub font_name: String,
    pub font_size_pt: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub color: RgbaColor,
    pub source_run_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconstructedTextLine {
    pub id: u64,
    pub text: String,
    pub words: Vec<ReconstructedWord>,
    pub bbox: PdfRect,
    pub baseline_y: f32,
    pub dominant_font_name: String,
    pub dominant_font_size_pt: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub dominant_color: RgbaColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticRole {
    Title,
    Heading(u8), // 1..=6
    ListItem {
        marker: String,
        ordered: bool,
        level: u8,
    },
    Caption,
    BodyParagraph,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditableParagraphBlock {
    pub id: u64,
    pub bbox: PdfRect,
    pub lines: Vec<ReconstructedTextLine>,
    pub alignment: TextAlignment,
    pub first_line_indent_pt: f32,
    pub left_indent_pt: f32,
    pub right_indent_pt: f32,
    pub line_spacing_ratio: f32,
    pub role: SemanticRole,
    pub font_name: String,
    pub font_size_pt: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub color: RgbaColor,
}

impl EditableParagraphBlock {
    /// Returns the combined text of the paragraph, joining hyphenated line wraps cleanly.
    pub fn full_text(&self) -> String {
        let mut out = String::new();
        for (idx, line) in self.lines.iter().enumerate() {
            let trimmed = line.text.trim();
            if idx > 0 {
                if out.ends_with('-') && trimmed.chars().next().map_or(false, |c| c.is_lowercase()) {
                    out.pop(); // Strip soft wrap hyphen
                } else {
                    out.push(' ');
                }
            }
            out.push_str(trimmed);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconstructedTableCell {
    pub row_idx: usize,
    pub col_idx: usize,
    pub row_span: usize,
    pub col_span: usize,
    pub bbox: PdfRect,
    pub lines: Vec<ReconstructedTextLine>,
    pub background_color: Option<RgbaColor>,
    pub alignment: TextAlignment,
}

impl ReconstructedTableCell {
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.text.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconstructedTableGrid {
    pub id: u64,
    pub bbox: PdfRect,
    pub is_ruled: bool,
    pub col_boundaries_pt: Vec<f32>,
    pub row_boundaries_pt: Vec<f32>,
    pub cells: Vec<ReconstructedTableCell>,
}

impl ReconstructedTableGrid {
    pub fn num_rows(&self) -> usize {
        self.row_boundaries_pt.len().saturating_sub(1)
    }

    pub fn num_cols(&self) -> usize {
        self.col_boundaries_pt.len().saturating_sub(1)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutElement {
    Paragraph(EditableParagraphBlock),
    Table(ReconstructedTableGrid),
}

impl LayoutElement {
    pub fn bbox(&self) -> PdfRect {
        match self {
            LayoutElement::Paragraph(p) => p.bbox,
            LayoutElement::Table(t) => t.bbox,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutColumnRegion {
    pub id: u64,
    pub bbox: PdfRect,
    pub elements: Vec<LayoutElement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconstructedPageLayout {
    pub page_index: usize,
    pub width_pt: f32,
    pub height_pt: f32,
    pub headers: Vec<EditableParagraphBlock>,
    pub footers: Vec<EditableParagraphBlock>,
    pub regions: Vec<LayoutColumnRegion>,
}

impl ReconstructedPageLayout {
    /// Iterates all body elements in strict topological reading order (top-to-bottom, column-by-column).
    pub fn reading_order_elements(&self) -> Vec<&LayoutElement> {
        self.regions.iter().flat_map(|r| r.elements.iter()).collect()
    }
}

// ============================================================================
// RECONSTRUCTOR CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct LayoutReconstructorConfig {
    /// Fraction of page height reserved for top header band (`0.08` = top 8%).
    pub header_band_ratio: f32,
    /// Fraction of page height reserved for bottom footer band (`0.92` = bottom 8%).
    pub footer_band_ratio: f32,
    /// Minimum vertical whitespace gutter width (in pt) to split columns in XY-Cut.
    pub min_column_gutter_pt: f32,
    /// Minimum horizontal whitespace valley height (in pt) to split full-width bands in XY-Cut.
    pub min_horizontal_cut_pt: f32,
    /// DBSCAN line-to-line vertical multiplier (`1.65` x font_size).
    pub max_paragraph_line_spacing_ratio: f32,
    /// Coordinate snap tolerance (in pt) for ruled table intersections.
    pub table_grid_snap_tolerance_pt: f32,
}

impl Default for LayoutReconstructorConfig {
    fn default() -> Self {
        Self {
            header_band_ratio: 0.08,
            footer_band_ratio: 0.92,
            min_column_gutter_pt: 14.0,
            min_horizontal_cut_pt: 10.0,
            max_paragraph_line_spacing_ratio: 1.65,
            table_grid_snap_tolerance_pt: 3.0,
        }
    }
}

// ============================================================================
// MAIN RECONSTRUCTOR PIPELINE
// ============================================================================

pub struct SpatialLayoutReconstructor {
    config: LayoutReconstructorConfig,
    next_id: u64,
}

impl SpatialLayoutReconstructor {
    pub fn new(config: LayoutReconstructorConfig) -> Self {
        Self { config, next_id: 1 }
    }

    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Executes the complete 4-stage spatial reconstruction pipeline on a `PageScene`.
    pub fn reconstruct_page(&mut self, scene: &PageScene) -> ReconstructedPageLayout {
        // Stage 1: Merge positioned glyph runs into words and collinear text lines
        let all_lines = self.merge_glyphs_into_lines(&scene.glyph_runs);

        // Compute median body font size across the page for semantic heading inference
        let median_body_font_size = Self::compute_median_font_size(&all_lines);

        // Separate Header / Footer lines from Body lines
        let header_cutoff_y = scene.height_pt * self.config.header_band_ratio;
        let footer_cutoff_y = scene.height_pt * self.config.footer_band_ratio;

        let mut header_lines = Vec::new();
        let mut footer_lines = Vec::new();
        let mut body_lines = Vec::new();

        for line in all_lines {
            if line.bbox.y1 <= header_cutoff_y && Self::looks_like_running_margin(&line, scene.width_pt) {
                header_lines.push(line);
            } else if line.bbox.y0 >= footer_cutoff_y && Self::looks_like_running_margin(&line, scene.width_pt) {
                footer_lines.push(line);
            } else {
                body_lines.push(line);
            }
        }

        // Stage 2A: Ruled Table Detection (from vector strokes and thin filled rects)
        let (ruled_tables, consumed_by_ruled) =
            self.detect_ruled_tables(&scene.stroke_segments, &scene.filled_rects, &body_lines);

        let mut remaining_lines: Vec<ReconstructedTextLine> = body_lines
            .into_iter()
            .filter(|l| !consumed_by_ruled.contains(&l.id))
            .collect();

        // Stage 2B: Borderless Table Detection (from aligned multi-row whitespace gutters)
        let (borderless_tables, consumed_by_borderless) =
            self.detect_borderless_tables(&remaining_lines);

        remaining_lines.retain(|l| !consumed_by_borderless.contains(&l.id));

        let mut all_tables = ruled_tables;
        all_tables.extend(borderless_tables);

        // Stage 3: Recursive XY-Cut Page Segmentation over remaining lines + reconstructed tables
        let regions = self.segment_into_xy_cut_regions(
            remaining_lines,
            all_tables,
            median_body_font_size,
            scene.width_pt,
            scene.height_pt,
        );

        let headers = self.cluster_lines_into_paragraphs(
            &header_lines,
            PdfRect::new(0.0, 0.0, scene.width_pt, header_cutoff_y),
            median_body_font_size,
        );
        let footers = self.cluster_lines_into_paragraphs(
            &footer_lines,
            PdfRect::new(0.0, footer_cutoff_y, scene.width_pt, scene.height_pt),
            median_body_font_size,
        );

        ReconstructedPageLayout {
            page_index: scene.page_index,
            width_pt: scene.width_pt,
            height_pt: scene.height_pt,
            headers,
            footers,
            regions,
        }
    }

    // ========================================================================
    // STAGE 1: BASELINE WORD & LINE CLUSTERING
    // ========================================================================

    fn merge_glyphs_into_lines(
        &mut self,
        runs: &[PositionedGlyphRun],
    ) -> Vec<ReconstructedTextLine> {
        let mut valid_runs: Vec<&PositionedGlyphRun> = runs
            .iter()
            .filter(|r| !r.text.is_empty() && r.bbox.width() >= 0.0)
            .collect();

        if valid_runs.is_empty() {
            return Vec::new();
        }

        // Sort primarily by baseline_y, then by horizontal x0
        valid_runs.sort_by(|a, b| {
            let y_tol = (a.font_size_pt.min(b.font_size_pt) * 0.25).max(1.5);
            if (a.baseline_y - b.baseline_y).abs() <= y_tol {
                a.bbox
                    .x0
                    .partial_cmp(&b.bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            } else {
                a.baseline_y
                    .partial_cmp(&b.baseline_y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        // Group runs into baseline buckets
        let mut baseline_rows: Vec<Vec<&PositionedGlyphRun>> = Vec::new();
        for run in valid_runs {
            if let Some(last_row) = baseline_rows.last_mut() {
                let ref_run = last_row[0];
                let y_tol = (ref_run.font_size_pt.min(run.font_size_pt) * 0.25).max(1.5);
                if (ref_run.baseline_y - run.baseline_y).abs() <= y_tol {
                    last_row.push(run);
                    continue;
                }
            }
            baseline_rows.push(vec![run]);
        }

        let mut lines = Vec::new();

        for mut row in baseline_rows {
            row.sort_by(|a, b| {
                a.bbox
                    .x0
                    .partial_cmp(&b.bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Split a single baseline row into distinct collinear ReconstructedTextLine clusters
            // whenever the horizontal gap between consecutive runs exceeds `1.8 * font_size`.
            // This preserves distinct column segments and table cells on the same Y coordinate!
            let mut current_line_words: Vec<ReconstructedWord> = Vec::new();

            for run in row {
                let font_size = run.font_size_pt.max(4.0);
                let word_merge_gap = font_size * 0.22;
                let line_split_gap = font_size * 1.80;

                if let Some(prev_word) = current_line_words.last_mut() {
                    let gap = run.bbox.x0 - prev_word.bbox.x1;

                    if gap <= word_merge_gap
                        && prev_word.font_name == run.font_name
                        && (prev_word.font_size_pt - run.font_size_pt).abs() <= 1.0
                    {
                        // Same word across adjacent glyph runs
                        prev_word.text.push_str(&run.text);
                        prev_word.bbox = prev_word.bbox.union(&run.bbox);
                        prev_word.source_run_ids.push(run.id);
                        continue;
                    } else if gap > line_split_gap {
                        // Wide horizontal gutter: flush current line cluster and start a new one
                        if let Some(built) = self.build_line_from_words(std::mem::take(&mut current_line_words)) {
                            lines.push(built);
                        }
                    }
                }

                // Split run text on internal spaces if the PDF emitted multi-word strings in one Tj
                let sub_words = Self::split_run_into_words(run);
                for sw in sub_words {
                    if let Some(prev) = current_line_words.last() {
                        let gap = sw.bbox.x0 - prev.bbox.x1;
                        if gap > line_split_gap {
                            if let Some(built) =
                                self.build_line_from_words(std::mem::take(&mut current_line_words))
                            {
                                lines.push(built);
                            }
                        }
                    }
                    current_line_words.push(sw);
                }
            }

            if let Some(built) = self.build_line_from_words(current_line_words) {
                lines.push(built);
            }
        }

        lines
    }

    fn split_run_into_words(run: &PositionedGlyphRun) -> Vec<ReconstructedWord> {
        let trimmed = run.text.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }
        // If the run contains internal multiple spaces (e.g. tabular padding), split proportionally
        if !run.text.contains("  ") {
            return vec![ReconstructedWord {
                text: run.text.trim().to_string(),
                bbox: run.bbox,
                baseline_y: run.baseline_y,
                font_name: run.font_name.clone(),
                font_size_pt: run.font_size_pt,
                is_bold: run.is_bold,
                is_italic: run.is_italic,
                color: run.color,
                source_run_ids: vec![run.id],
            }];
        }

        let total_chars = run.text.chars().count().max(1) as f32;
        let char_w = run.bbox.width() / total_chars;
        let mut result = Vec::new();
        let mut char_idx = 0usize;
        let chars: Vec<char> = run.text.chars().collect();

        while char_idx < chars.len() {
            while char_idx < chars.len() && chars[char_idx].is_whitespace() {
                char_idx += 1;
            }
            if char_idx >= chars.len() {
                break;
            }
            let start = char_idx;
            while char_idx < chars.len() {
                // Break only on 2+ consecutive spaces; single space stays inside phrase unless split
                if chars[char_idx].is_whitespace() {
                    break;
                }
                char_idx += 1;
            }
            let end = char_idx;
            let token: String = chars[start..end].iter().collect();
            let x0 = run.bbox.x0 + (start as f32) * char_w;
            let x1 = run.bbox.x0 + (end as f32) * char_w;
            result.push(ReconstructedWord {
                text: token,
                bbox: PdfRect::new(x0, run.bbox.y0, x1, run.bbox.y1),
                baseline_y: run.baseline_y,
                font_name: run.font_name.clone(),
                font_size_pt: run.font_size_pt,
                is_bold: run.is_bold,
                is_italic: run.is_italic,
                color: run.color,
                source_run_ids: vec![run.id],
            });
        }

        result
    }

    fn build_line_from_words(
        &mut self,
        words: Vec<ReconstructedWord>,
    ) -> Option<ReconstructedTextLine> {
        if words.is_empty() {
            return None;
        }
        let mut bbox = words[0].bbox;
        let mut text = String::new();
        let mut baseline_sum = 0.0;

        for (i, w) in words.iter().enumerate() {
            if i > 0 && !text.ends_with(' ') && !w.text.starts_with(' ') {
                text.push(' ');
            }
            text.push_str(w.text.trim());
            bbox = bbox.union(&w.bbox);
            baseline_sum += w.baseline_y;
        }

        let first = &words[0];
        let id = self.alloc_id();
        Some(ReconstructedTextLine {
            id,
            text,
            bbox,
            baseline_y: baseline_sum / (words.len() as f32),
            dominant_font_name: first.font_name.clone(),
            dominant_font_size_pt: first.font_size_pt,
            is_bold: words.iter().all(|w| w.is_bold),
            is_italic: words.iter().all(|w| w.is_italic),
            dominant_color: first.color,
            words,
        })
    }

    // ========================================================================
    // STAGE 2A: RULED VECTOR TABLE RECONSTRUCTION
    // ========================================================================

    fn detect_ruled_tables(
        &mut self,
        strokes: &[VectorStrokeSegment],
        filled_rects: &[VectorFilledRect],
        lines: &[ReconstructedTextLine],
    ) -> (Vec<ReconstructedTableGrid>, HashSet<u64>) {
        #[derive(Clone, Copy, Debug)]
        struct HSeg {
            y: f32,
            x0: f32,
            x1: f32,
        }
        #[derive(Clone, Copy, Debug)]
        struct VSeg {
            x: f32,
            y0: f32,
            y1: f32,
        }

        let mut h_segs = Vec::new();
        let mut v_segs = Vec::new();

        for s in strokes {
            let dx = (s.x1 - s.x0).abs();
            let dy = (s.y1 - s.y0).abs();
            if dy <= 1.5 && dx >= 12.0 {
                h_segs.push(HSeg {
                    y: (s.y0 + s.y1) * 0.5,
                    x0: s.x0.min(s.x1),
                    x1: s.x0.max(s.x1),
                });
            } else if dx <= 1.5 && dy >= 10.0 {
                v_segs.push(VSeg {
                    x: (s.x0 + s.x1) * 0.5,
                    y0: s.y0.min(s.y1),
                    y1: s.y0.max(s.y1),
                });
            }
        }

        // Thin filled rectangles are frequently used by PDF generators to draw table rules
        for fr in filled_rects {
            let w = fr.rect.width();
            let h = fr.rect.height();
            if h <= 3.0 && w >= 12.0 {
                h_segs.push(HSeg {
                    y: fr.rect.center_y(),
                    x0: fr.rect.x0,
                    x1: fr.rect.x1,
                });
            } else if w <= 3.0 && h >= 10.0 {
                v_segs.push(VSeg {
                    x: fr.rect.center_x(),
                    y0: fr.rect.y0,
                    y1: fr.rect.y1,
                });
            }
        }

        if h_segs.len() < 2 || v_segs.len() < 2 {
            return (Vec::new(), HashSet::new());
        }

        let tol = self.config.table_grid_snap_tolerance_pt;

        // Snap unique X coordinates of vertical segments and Y coordinates of horizontal segments
        let mut raw_xs: Vec<f32> = v_segs.iter().map(|v| v.x).collect();
        let mut raw_ys: Vec<f32> = h_segs.iter().map(|h| h.y).collect();
        let col_boundaries = Self::dedup_sorted_coords(&mut raw_xs, tol);
        let row_boundaries = Self::dedup_sorted_coords(&mut raw_ys, tol);

        if col_boundaries.len() < 2 || row_boundaries.len() < 2 {
            return (Vec::new(), HashSet::new());
        }

        let num_rows = row_boundaries.len() - 1;
        let num_cols = col_boundaries.len() - 1;

        let table_bbox = PdfRect::new(
            *col_boundaries.first().unwrap(),
            *row_boundaries.first().unwrap(),
            *col_boundaries.last().unwrap(),
            *row_boundaries.last().unwrap(),
        );

        // Helper to check if a horizontal rule exists along row boundary `r` between `col c` and `c+1`
        let has_h_rule = |r_idx: usize, c_idx: usize| -> bool {
            let y = row_boundaries[r_idx];
            let x_start = col_boundaries[c_idx];
            let x_end = col_boundaries[c_idx + 1];
            let mid_x = (x_start + x_end) * 0.5;
            h_segs.iter().any(|h| {
                (h.y - y).abs() <= tol && h.x0 <= mid_x + tol && h.x1 >= mid_x - tol
            })
        };

        // Helper to check if a vertical rule exists along col boundary `c` between `row r` and `r+1`
        let has_v_rule = |r_idx: usize, c_idx: usize| -> bool {
            let x = col_boundaries[c_idx];
            let y_start = row_boundaries[r_idx];
            let y_end = row_boundaries[r_idx + 1];
            let mid_y = (y_start + y_end) * 0.5;
            v_segs.iter().any(|v| {
                (v.x - x).abs() <= tol && v.y0 <= mid_y + tol && v.y1 >= mid_y - tol
            })
        };

        // Reconstruct merged cells (`row_span`, `col_span`) by traversing the elementary grid
        let mut visited = vec![vec![false; num_cols]; num_rows];
        let mut cells = Vec::new();
        let mut consumed_line_ids = HashSet::new();

        for r in 0..num_rows {
            for c in 0..num_cols {
                if visited[r][c] {
                    continue;
                }

                // Expand horizontally while interior vertical rule is absent
                let mut col_span = 1;
                while c + col_span < num_cols && !has_v_rule(r, c + col_span) {
                    col_span += 1;
                }

                // Expand vertically while interior horizontal rule is absent across the full col_span
                let mut row_span = 1;
                'row_expand: while r + row_span < num_rows {
                    for span_c in c..(c + col_span) {
                        if has_h_rule(r + row_span, span_c) {
                            break 'row_expand;
                        }
                    }
                    row_span += 1;
                }

                for vr in r..(r + row_span) {
                    for vc in c..(c + col_span) {
                        visited[vr][vc] = true;
                    }
                }

                let cell_bbox = PdfRect::new(
                    col_boundaries[c],
                    row_boundaries[r],
                    col_boundaries[c + col_span],
                    row_boundaries[r + row_span],
                );

                // Collect text lines whose center falls inside `cell_bbox`
                let mut cell_lines: Vec<ReconstructedTextLine> = lines
                    .iter()
                    .filter(|l| {
                        !consumed_line_ids.contains(&l.id)
                            && cell_bbox.contains_point(l.bbox.center_x(), l.bbox.center_y(), 2.0)
                    })
                    .cloned()
                    .collect();

                cell_lines.sort_by(|a, b| {
                    a.baseline_y
                        .partial_cmp(&b.baseline_y)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                for cl in &cell_lines {
                    consumed_line_ids.insert(cl.id);
                }

                // Detect cell background shading from `filled_rects`
                let bg_color = filled_rects
                    .iter()
                    .find(|fr| {
                        fr.rect.width() > 6.0
                            && fr.rect.height() > 6.0
                            && fr.rect.contains_point(cell_bbox.center_x(), cell_bbox.center_y(), 1.0)
                    })
                    .map(|fr| fr.fill_color);

                let alignment = Self::infer_cell_alignment(&cell_lines, cell_bbox);

                cells.push(ReconstructedTableCell {
                    row_idx: r,
                    col_idx: c,
                    row_span,
                    col_span,
                    bbox: cell_bbox,
                    lines: cell_lines,
                    background_color: bg_color,
                    alignment,
                });
            }
        }

        let table = ReconstructedTableGrid {
            id: self.alloc_id(),
            bbox: table_bbox,
            is_ruled: true,
            col_boundaries_pt: col_boundaries,
            row_boundaries_pt: row_boundaries,
            cells,
        };

        (vec![table], consumed_line_ids)
    }

    // ========================================================================
    // STAGE 2B: BORDERLESS WHITESPACE-GUTTER TABLE RECONSTRUCTION
    // ========================================================================

    fn detect_borderless_tables(
        &mut self,
        lines: &[ReconstructedTextLine],
    ) -> (Vec<ReconstructedTableGrid>, HashSet<u64>) {
        if lines.len() < 6 {
            return (Vec::new(), HashSet::new());
        }

        // Group lines into shared horizontal visual rows by baseline Y
        let mut sorted_lines: Vec<&ReconstructedTextLine> = lines.iter().collect();
        sorted_lines.sort_by(|a, b| {
            a.baseline_y
                .partial_cmp(&b.baseline_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut visual_rows: Vec<Vec<&ReconstructedTextLine>> = Vec::new();
        for line in sorted_lines {
            if let Some(last_row) = visual_rows.last_mut() {
                let tol = (last_row[0].dominant_font_size_pt * 0.35).max(2.5);
                if (last_row[0].baseline_y - line.baseline_y).abs() <= tol {
                    last_row.push(line);
                    continue;
                }
            }
            visual_rows.push(vec![line]);
        }

        for row in &mut visual_rows {
            row.sort_by(|a, b| {
                a.bbox
                    .x0
                    .partial_cmp(&b.bbox.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        // Scan for contiguous bands of >= 3 visual rows where each row has >= 2 distinct horizontal cells
        // and vertical gap between rows is consistent with tabular spacing.
        let mut tables = Vec::new();
        let mut consumed = HashSet::new();
        let mut idx = 0;

        while idx < visual_rows.len() {
            if visual_rows[idx].len() < 2 {
                idx += 1;
                continue;
            }

            let start_idx = idx;
            let mut end_idx = idx + 1;
            while end_idx < visual_rows.len() && visual_rows[end_idx].len() >= 2 {
                let prev_y = visual_rows[end_idx - 1][0].baseline_y;
                let curr_y = visual_rows[end_idx][0].baseline_y;
                let font_sz = visual_rows[end_idx][0].dominant_font_size_pt.max(8.0);
                if (curr_y - prev_y) > font_sz * 3.2 {
                    break;
                }
                end_idx += 1;
            }

            let row_count = end_idx - start_idx;
            if row_count >= 3 {
                let candidate_rows = &visual_rows[start_idx..end_idx];
                if let Some(table) = self.try_build_borderless_grid(candidate_rows, &mut consumed) {
                    tables.push(table);
                }
            }

            idx = end_idx;
        }

        (tables, consumed)
    }

    fn try_build_borderless_grid(
        &mut self,
        candidate_rows: &[Vec<&ReconstructedTextLine>],
        consumed: &mut HashSet<u64>,
    ) -> Option<ReconstructedTableGrid> {
        // Project X intervals of all cells across candidate_rows to discover consistent column tracks
        let mut intervals: Vec<(f32, f32)> = Vec::new();
        let mut table_bbox: Option<PdfRect> = None;

        for row in candidate_rows {
            for cell_line in row {
                intervals.push((cell_line.bbox.x0, cell_line.bbox.x1));
                table_bbox = Some(match table_bbox {
                    Some(b) => b.union(&cell_line.bbox),
                    None => cell_line.bbox,
                });
            }
        }

        let table_bbox = table_bbox?;
        intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Merge overlapping horizontal intervals into column tracks
        let mut col_tracks: Vec<(f32, f32)> = Vec::new();
        for (x0, x1) in intervals {
            if let Some(last) = col_tracks.last_mut() {
                if x0 <= last.1 + 6.0 {
                    last.1 = last.1.max(x1);
                    continue;
                }
            }
            col_tracks.push((x0, x1));
        }

        if col_tracks.len() < 2 {
            return None;
        }

        // Build column boundary coordinates at midpoints of vertical gutters
        let mut col_boundaries = Vec::with_capacity(col_tracks.len() + 1);
        col_boundaries.push(col_tracks[0].0);
        for i in 0..(col_tracks.len() - 1) {
            let gutter_mid = (col_tracks[i].1 + col_tracks[i + 1].0) * 0.5;
            col_boundaries.push(gutter_mid);
        }
        col_boundaries.push(col_tracks.last().unwrap().1);

        // Build row boundary coordinates
        let mut row_boundaries = Vec::with_capacity(candidate_rows.len() + 1);
        let first_top = candidate_rows[0]
            .iter()
            .map(|l| l.bbox.y0)
            .fold(f32::INFINITY, f32::min);
        row_boundaries.push(first_top);

        for r in 0..(candidate_rows.len() - 1) {
            let r_bot = candidate_rows[r]
                .iter()
                .map(|l| l.bbox.y1)
                .fold(f32::NEG_INFINITY, f32::max);
            let next_top = candidate_rows[r + 1]
                .iter()
                .map(|l| l.bbox.y0)
                .fold(f32::INFINITY, f32::min);
            row_boundaries.push((r_bot + next_top) * 0.5);
        }

        let last_bot = candidate_rows
            .last()
            .unwrap()
            .iter()
            .map(|l| l.bbox.y1)
            .fold(f32::NEG_INFINITY, f32::max);
        row_boundaries.push(last_bot);

        let num_cols = col_tracks.len();
        let mut cells = Vec::new();

        for (r_idx, row) in candidate_rows.iter().enumerate() {
            for c_idx in 0..num_cols {
                let cell_bbox = PdfRect::new(
                    col_boundaries[c_idx],
                    row_boundaries[r_idx],
                    col_boundaries[c_idx + 1],
                    row_boundaries[r_idx + 1],
                );

                let cell_lines: Vec<ReconstructedTextLine> = row
                    .iter()
                    .filter(|l| {
                        let cx = l.bbox.center_x();
                        cx >= cell_bbox.x0 && cx <= cell_bbox.x1
                    })
                    .map(|l| (*l).clone())
                    .collect();

                for cl in &cell_lines {
                    consumed.insert(cl.id);
                }

                let alignment = Self::infer_cell_alignment(&cell_lines, cell_bbox);

                cells.push(ReconstructedTableCell {
                    row_idx: r_idx,
                    col_idx: c_idx,
                    row_span: 1,
                    col_span: 1,
                    bbox: cell_bbox,
                    lines: cell_lines,
                    background_color: None,
                    alignment,
                });
            }
        }

        Some(ReconstructedTableGrid {
            id: self.alloc_id(),
            bbox: table_bbox,
            is_ruled: false,
            col_boundaries_pt: col_boundaries,
            row_boundaries_pt: row_boundaries,
            cells,
        })
    }

    // ========================================================================
    // STAGE 3: RECURSIVE PROJECTION-PROFILE XY-CUT SEGMENTATION
    // ========================================================================

    fn segment_into_xy_cut_regions(
        &mut self,
        lines: Vec<ReconstructedTextLine>,
        tables: Vec<ReconstructedTableGrid>,
        median_body_font_size: f32,
        _page_w: f32,
        _page_h: f32,
    ) -> Vec<LayoutColumnRegion> {
        #[derive(Clone)]
        enum RawBlock {
            Line(ReconstructedTextLine),
            Table(ReconstructedTableGrid),
        }

        impl RawBlock {
            fn bbox(&self) -> PdfRect {
                match self {
                    RawBlock::Line(l) => l.bbox,
                    RawBlock::Table(t) => t.bbox,
                }
            }
        }

        let mut blocks: Vec<RawBlock> = lines.into_iter().map(RawBlock::Line).collect();
        blocks.extend(tables.into_iter().map(RawBlock::Table));

        if blocks.is_empty() {
            return Vec::new();
        }

        fn recursive_xy_cut(
            mut blocks: Vec<RawBlock>,
            min_col_gutter: f32,
            min_h_cut: f32,
            prefer_vertical: bool,
        ) -> Vec<Vec<RawBlock>> {
            if blocks.len() <= 1 {
                return vec![blocks];
            }

            let bbox = blocks
                .iter()
                .fold(blocks[0].bbox(), |acc, b| acc.union(&b.bbox()));

            let try_v_split = |items: &[RawBlock]| -> Option<f32> {
                let mut intervals: Vec<(f32, f32)> =
                    items.iter().map(|b| (b.bbox().x0, b.bbox().x1)).collect();
                intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                let mut max_end = intervals[0].1;
                let mut best_cut: Option<(f32, f32)> = None; // (gap_width, cut_x)
                for &(x0, x1) in &intervals[1..] {
                    let gap = x0 - max_end;
                    if gap >= min_col_gutter {
                        if best_cut.map_or(true, |(best_g, _)| gap > best_g) {
                            best_cut = Some((gap, (max_end + x0) * 0.5));
                        }
                    }
                    max_end = max_end.max(x1);
                }
                best_cut.map(|(_, cut_x)| cut_x)
            };

            let try_h_split = |items: &[RawBlock]| -> Option<f32> {
                // Only perform a horizontal XY-cut when separating a full-width element
                // (like a wide title banner or wide table) from multi-column blocks below/above it.
                let mut intervals: Vec<(f32, f32, f32)> = items
                    .iter()
                    .map(|b| (b.bbox().y0, b.bbox().y1, b.bbox().width()))
                    .collect();
                intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                let region_w = bbox.width().max(1.0);
                let has_wide = intervals.iter().any(|&(_, _, w)| w > region_w * 0.62);
                let has_narrow = intervals.iter().any(|&(_, _, w)| w <= region_w * 0.52);
                if !(has_wide && has_narrow) {
                    return None;
                }

                let mut max_end = intervals[0].1;
                for &(y0, y1, _) in &intervals[1..] {
                    let gap = y0 - max_end;
                    if gap >= min_h_cut {
                        return Some((max_end + y0) * 0.5);
                    }
                    max_end = max_end.max(y1);
                }
                None
            };

            if prefer_vertical {
                if let Some(cut_x) = try_v_split(&blocks) {
                    let (left, right): (Vec<_>, Vec<_>) =
                        blocks.into_iter().partition(|b| b.bbox().center_x() < cut_x);
                    if !left.is_empty() && !right.is_empty() {
                        let mut out = recursive_xy_cut(left, min_col_gutter, min_h_cut, false);
                        out.extend(recursive_xy_cut(right, min_col_gutter, min_h_cut, false));
                        return out;
                    }
                    blocks = if left.is_empty() { right } else { left };
                }
                if let Some(cut_y) = try_h_split(&blocks) {
                    let (top, bot): (Vec<_>, Vec<_>) =
                        blocks.into_iter().partition(|b| b.bbox().center_y() < cut_y);
                    if !top.is_empty() && !bot.is_empty() {
                        let mut out = recursive_xy_cut(top, min_col_gutter, min_h_cut, true);
                        out.extend(recursive_xy_cut(bot, min_col_gutter, min_h_cut, true));
                        return out;
                    }
                    blocks = if top.is_empty() { bot } else { top };
                }
            } else {
                if let Some(cut_y) = try_h_split(&blocks) {
                    let (top, bot): (Vec<_>, Vec<_>) =
                        blocks.into_iter().partition(|b| b.bbox().center_y() < cut_y);
                    if !top.is_empty() && !bot.is_empty() {
                        let mut out = recursive_xy_cut(top, min_col_gutter, min_h_cut, true);
                        out.extend(recursive_xy_cut(bot, min_col_gutter, min_h_cut, true));
                        return out;
                    }
                    blocks = if top.is_empty() { bot } else { top };
                }
                if let Some(cut_x) = try_v_split(&blocks) {
                    let (left, right): (Vec<_>, Vec<_>) =
                        blocks.into_iter().partition(|b| b.bbox().center_x() < cut_x);
                    if !left.is_empty() && !right.is_empty() {
                        let mut out = recursive_xy_cut(left, min_col_gutter, min_h_cut, false);
                        out.extend(recursive_xy_cut(right, min_col_gutter, min_h_cut, false));
                        return out;
                    }
                    blocks = if left.is_empty() { right } else { left };
                }
            }

            vec![blocks]
        }

        let leaf_groups = recursive_xy_cut(
            blocks,
            self.config.min_column_gutter_pt,
            self.config.min_horizontal_cut_pt,
            false,
        );

        let mut regions = Vec::new();
        for group in leaf_groups {
            if group.is_empty() {
                continue;
            }
            let region_bbox = group
                .iter()
                .fold(group[0].bbox(), |acc, b| acc.union(&b.bbox()));

            let mut region_lines = Vec::new();
            let mut region_tables = Vec::new();
            for item in group {
                match item {
                    RawBlock::Line(l) => region_lines.push(l),
                    RawBlock::Table(t) => region_tables.push(t),
                }
            }

            let paragraphs = self.cluster_lines_into_paragraphs(
                &region_lines,
                region_bbox,
                median_body_font_size,
            );

            let mut elements: Vec<LayoutElement> =
                paragraphs.into_iter().map(LayoutElement::Paragraph).collect();
            elements.extend(region_tables.into_iter().map(LayoutElement::Table));

            // Sort elements within each leaf column strictly top-to-bottom
            elements.sort_by(|a, b| {
                a.bbox()
                    .y0
                    .partial_cmp(&b.bbox().y0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            regions.push(LayoutColumnRegion {
                id: self.alloc_id(),
                bbox: region_bbox,
                elements,
            });
        }

        regions
    }

    // ========================================================================
    // STAGE 4: DBSCAN / DENSITY & TYPOGRAPHY PARAGRAPH CLUSTERING
    // ========================================================================

    fn cluster_lines_into_paragraphs(
        &mut self,
        lines: &[ReconstructedTextLine],
        region_bbox: PdfRect,
        median_body_font_size: f32,
    ) -> Vec<EditableParagraphBlock> {
        if lines.is_empty() {
            return Vec::new();
        }

        let mut sorted: Vec<ReconstructedTextLine> = lines.to_vec();
        sorted.sort_by(|a, b| {
            a.baseline_y
                .partial_cmp(&b.baseline_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut clusters: Vec<Vec<ReconstructedTextLine>> = Vec::new();

        for line in sorted {
            let starts_new_list_item = Self::detect_list_marker(&line.text).is_some();

            let should_merge = if let Some(curr_cluster) = clusters.last() {
                if starts_new_list_item {
                    false
                } else {
                    let prev_line = curr_cluster.last().unwrap();
                    self.are_lines_density_connected(prev_line, &line, region_bbox)
                }
            } else {
                false
            };

            if should_merge {
                clusters.last_mut().unwrap().push(line);
            } else {
                clusters.push(vec![line]);
            }
        }

        let mut paragraphs = Vec::with_capacity(clusters.len());
        for cluster in clusters {
            paragraphs.push(self.build_paragraph_block(
                cluster,
                region_bbox,
                median_body_font_size,
            ));
        }

        paragraphs
    }

    /// DBSCAN neighborhood predicate checking vertical baseline proximity, font compatibility,
    /// and horizontal column overlap between consecutive lines.
    fn are_lines_density_connected(
        &self,
        prev: &ReconstructedTextLine,
        curr: &ReconstructedTextLine,
        region_bbox: PdfRect,
    ) -> bool {
        // Font size difference > 1.5pt indicates a heading-to-body or caption transition
        if (prev.dominant_font_size_pt - curr.dominant_font_size_pt).abs() > 1.5 {
            return false;
        }

        // Boldness transition on short single-line headings
        if prev.is_bold != curr.is_bold && prev.bbox.width() < region_bbox.width() * 0.75 {
            return false;
        }

        // Vertical baseline spacing epsilon check
        let dy = curr.baseline_y - prev.baseline_y;
        let max_dy = prev.dominant_font_size_pt * self.config.max_paragraph_line_spacing_ratio;
        if dy <= 0.0 || dy > max_dy {
            return false;
        }

        // Horizontal overlap check
        if prev.bbox.horizontal_overlap_ratio(&curr.bbox) < 0.25 {
            return false;
        }

        // If the previous line stopped far short of the right margin (less than 68% of column width),
        // it is likely the terminal line of a paragraph.
        let col_right = region_bbox.x1;
        let prev_fill_ratio =
            (prev.bbox.x1 - region_bbox.x0) / (col_right - region_bbox.x0).max(1.0);
        if prev_fill_ratio < 0.68 {
            return false;
        }

        // If `curr` has a significant positive first-line indent compared to `prev`, `curr` starts a new paragraph
        if curr.bbox.x0 - prev.bbox.x0 >= prev.dominant_font_size_pt * 1.2 {
            return false;
        }

        true
    }

    fn build_paragraph_block(
        &mut self,
        lines: Vec<ReconstructedTextLine>,
        region_bbox: PdfRect,
        median_body_font_size: f32,
    ) -> EditableParagraphBlock {
        let bbox = lines
            .iter()
            .fold(lines[0].bbox, |acc, l| acc.union(&l.bbox));

        let font_name = lines[0].dominant_font_name.clone();
        let font_size = lines[0].dominant_font_size_pt;
        let is_bold = lines[0].is_bold;
        let is_italic = lines[0].is_italic;
        let color = lines[0].dominant_color;

        // Compute alignment & indents across the paragraph's lines
        let (alignment, first_line_indent_pt, left_indent_pt, right_indent_pt) =
            Self::infer_paragraph_alignment_and_indents(&lines, region_bbox);

        // Compute average line spacing ratio
        let line_spacing_ratio = if lines.len() >= 2 {
            let total_dy = lines.last().unwrap().baseline_y - lines[0].baseline_y;
            let avg_dy = total_dy / ((lines.len() - 1) as f32);
            (avg_dy / font_size.max(1.0)).clamp(0.8, 3.0)
        } else {
            1.15
        };

        // Infer SemanticRole (Title, Heading 1..6, ListItem, Caption, BodyParagraph)
        let role = Self::infer_semantic_role(&lines, font_size, median_body_font_size);

        EditableParagraphBlock {
            id: self.alloc_id(),
            bbox,
            lines,
            alignment,
            first_line_indent_pt,
            left_indent_pt,
            right_indent_pt,
            line_spacing_ratio,
            role,
            font_name,
            font_size_pt: font_size,
            is_bold,
            is_italic,
            color,
        }
    }

    fn infer_paragraph_alignment_and_indents(
        lines: &[ReconstructedTextLine],
        region_bbox: PdfRect,
    ) -> (TextAlignment, f32, f32, f32) {
        if lines.len() == 1 {
            let l = &lines[0];
            let left_gap = (l.bbox.x0 - region_bbox.x0).max(0.0);
            let right_gap = (region_bbox.x1 - l.bbox.x1).max(0.0);
            if (left_gap - right_gap).abs() <= 8.0 && left_gap > 12.0 {
                return (TextAlignment::Center, 0.0, 0.0, 0.0);
            } else if right_gap <= 6.0 && left_gap > 20.0 {
                return (TextAlignment::Right, 0.0, left_gap, 0.0);
            }
            return (TextAlignment::Left, 0.0, left_gap, right_gap);
        }

        // Multi-line analysis: examine interior lines (all lines except the last line)
        let interior = &lines[..lines.len() - 1];
        let body_left = lines[1..]
            .iter()
            .map(|l| l.bbox.x0)
            .fold(f32::INFINITY, f32::min);
        let first_indent = lines[0].bbox.x0 - body_left;

        let left_aligned = interior
            .iter()
            .skip(1)
            .all(|l| (l.bbox.x0 - body_left).abs() <= 3.0);
        let max_right = interior
            .iter()
            .map(|l| l.bbox.x1)
            .fold(f32::NEG_INFINITY, f32::max);
        let right_aligned = interior.iter().all(|l| (max_right - l.bbox.x1).abs() <= 3.5);

        let centered = lines.iter().all(|l| {
            let lg = l.bbox.x0 - region_bbox.x0;
            let rg = region_bbox.x1 - l.bbox.x1;
            (lg - rg).abs() <= 8.0
        });

        let alignment = if left_aligned && right_aligned && (max_right - body_left) > region_bbox.width() * 0.70 {
            TextAlignment::Justify
        } else if centered {
            TextAlignment::Center
        } else if right_aligned && !left_aligned {
            TextAlignment::Right
        } else {
            TextAlignment::Left
        };

        let left_indent = (body_left - region_bbox.x0).max(0.0);
        let right_indent = (region_bbox.x1 - max_right).max(0.0);
        (alignment, first_indent, left_indent, right_indent)
    }

    fn infer_semantic_role(
        lines: &[ReconstructedTextLine],
        font_size: f32,
        median_body_size: f32,
    ) -> SemanticRole {
        let first_text = lines[0].text.trim();

        if let Some((marker, ordered)) = Self::detect_list_marker(first_text) {
            return SemanticRole::ListItem {
                marker,
                ordered,
                level: 0,
            };
        }

        let ratio = font_size / median_body_size.max(8.0);
        if ratio >= 1.85 {
            return SemanticRole::Title;
        } else if ratio >= 1.50 {
            return SemanticRole::Heading(1);
        } else if ratio >= 1.25 {
            return SemanticRole::Heading(2);
        } else if (ratio >= 1.10 && lines[0].is_bold) || (lines.len() == 1 && lines[0].is_bold) {
            return SemanticRole::Heading(3);
        }

        if (first_text.starts_with("Figure ") || first_text.starts_with("Table "))
            && ratio <= 0.95
        {
            return SemanticRole::Caption;
        }

        SemanticRole::BodyParagraph
    }

    fn detect_list_marker(text: &str) -> Option<(String, bool)> {
        let trimmed = text.trim_start();
        for bullet in ["â€¢", "â—", "â—‹", "â–ª", "â–«", "â€“", "- "] {
            if trimmed.starts_with(bullet) {
                return Some((bullet.trim().to_string(), false));
            }
        }

        // Check ordered patterns like "1.", "1)", "(1)", "a.", "i."
        let first_token = trimmed.split_whitespace().next()?;
        if first_token.len() <= 5
            && (first_token.ends_with('.') || first_token.ends_with(')'))
            && first_token
                .trim_matches(|c| c == '(' || c == ')' || c == '.')
                .chars()
                .all(|c| c.is_ascii_alphanumeric())
        {
            return Some((first_token.to_string(), true));
        }

        None
    }

    fn infer_cell_alignment(lines: &[ReconstructedTextLine], cell_bbox: PdfRect) -> TextAlignment {
        if lines.is_empty() {
            return TextAlignment::Left;
        }
        let l = &lines[0];
        let left_pad = (l.bbox.x0 - cell_bbox.x0).max(0.0);
        let right_pad = (cell_bbox.x1 - l.bbox.x1).max(0.0);

        if (left_pad - right_pad).abs() <= 4.0 && left_pad > 6.0 {
            TextAlignment::Center
        } else if right_pad < left_pad * 0.5 && left_pad > 8.0 {
            TextAlignment::Right
        } else {
            TextAlignment::Left
        }
    }

    fn compute_median_font_size(lines: &[ReconstructedTextLine]) -> f32 {
        if lines.is_empty() {
            return 11.0;
        }
        let mut sizes: Vec<f32> = lines.iter().map(|l| l.dominant_font_size_pt).collect();
        sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sizes[sizes.len() / 2]
    }

    fn looks_like_running_margin(line: &ReconstructedTextLine, page_w: f32) -> bool {
        line.dominant_font_size_pt <= 10.5 && line.bbox.width() <= page_w * 0.85
    }

    fn dedup_sorted_coords(coords: &mut Vec<f32>, tol: f32) -> Vec<f32> {
        coords.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut out: Vec<f32> = Vec::new();
        for &c in coords.iter() {
            if let Some(last) = out.last_mut() {
                if (c - *last).abs() <= tol {
                    *last = (*last + c) * 0.5;
                    continue;
                }
            }
            out.push(c);
        }
        out
    }
}

// ============================================================================
// UNIT TESTS (GATE 5 VERIFICATION SUITE)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_run(
        id: u64,
        text: &str,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        font_size: f32,
        bold: bool,
    ) -> PositionedGlyphRun {
        PositionedGlyphRun {
            id,
            text: text.to_string(),
            bbox: PdfRect::new(x0, y0, x1, y1),
            baseline_y: y1 - 2.0,
            font_name: "Inter".to_string(),
            font_size_pt: font_size,
            is_bold: bold,
            is_italic: false,
            color: RgbaColor::default(),
            stream_byte_range: None,
        }
    }

    #[test]
    fn test_two_column_xy_cut_reading_order() {
        // Page: 612 x 792 pt (US Letter)
        // Full-width Title at top (y = 80..104), then 2 columns:
        // Left column: x = 50..280, Right column: x = 320..550 (40pt gutter)
        let scene = PageScene {
            page_index: 0,
            width_pt: 612.0,
            height_pt: 792.0,
            glyph_runs: vec![
                // Running Header
                make_run(1, "OpenWPS Technical Whitepaper", 50.0, 30.0, 220.0, 40.0, 9.0, false),
                // Full-width Title
                make_run(
                    2,
                    "Spatial PDF Layout Reconstruction Architecture",
                    80.0,
                    80.0,
                    530.0,
                    104.0,
                    22.0,
                    true,
                ),
                // Left Column Paragraph (2 lines)
                make_run(3, "Left column line one explains how", 50.0, 140.0, 275.0, 152.0, 11.0, false),
                make_run(4, "projection profiles segment pages.", 50.0, 154.0, 240.0, 166.0, 11.0, false),
                // Right Column Paragraph (at same Y as Left Column!)
                make_run(5, "Right column line one must appear", 320.0, 140.0, 545.0, 152.0, 11.0, false),
                make_run(6, "after the entire left column finishes.", 320.0, 154.0, 520.0, 166.0, 11.0, false),
            ],
            stroke_segments: vec![],
            filled_rects: vec![],
        };

        let mut reconstructor = SpatialLayoutReconstructor::new(LayoutReconstructorConfig::default());
        let layout = reconstructor.reconstruct_page(&scene);

        assert_eq!(layout.headers.len(), 1);
        assert_eq!(layout.headers[0].full_text(), "OpenWPS Technical Whitepaper");

        let elements = layout.reading_order_elements();
        assert_eq!(elements.len(), 3, "Expected Title + Left Col Para + Right Col Para");

        let texts: Vec<String> = elements
            .iter()
            .map(|el| match el {
                LayoutElement::Paragraph(p) => p.full_text(),
                LayoutElement::Table(_) => panic!("Unexpected table"),
            })
            .collect();

        assert_eq!(texts[0], "Spatial PDF Layout Reconstruction Architecture");
        assert_eq!(
            texts[1],
            "Left column line one explains how projection profiles segment pages."
        );
        assert_eq!(
            texts[2],
            "Right column line one must appear after the entire left column finishes."
        );
    }

    #[test]
    fn test_ruled_table_with_merged_header_cell() {
        // Construct a 2-row x 2-col ruled table where Row 0 merges Col 0 and Col 1 (col_span = 2)
        // X boundaries: 100, 250, 400
        // Y boundaries: 200, 230, 260
        let strokes = vec![
            // Horizontal rules (top, middle, bottom)
            VectorStrokeSegment { x0: 100.0, y0: 200.0, x1: 400.0, y1: 200.0, stroke_width_pt: 1.0, color: RgbaColor::default() },
            VectorStrokeSegment { x0: 100.0, y0: 230.0, x1: 400.0, y1: 230.0, stroke_width_pt: 1.0, color: RgbaColor::default() },
            VectorStrokeSegment { x0: 100.0, y0: 260.0, x1: 400.0, y1: 260.0, stroke_width_pt: 1.0, color: RgbaColor::default() },
            // Outer left & right vertical rules
            VectorStrokeSegment { x0: 100.0, y0: 200.0, x1: 100.0, y1: 260.0, stroke_width_pt: 1.0, color: RgbaColor::default() },
            VectorStrokeSegment { x0: 400.0, y0: 200.0, x1: 400.0, y1: 260.0, stroke_width_pt: 1.0, color: RgbaColor::default() },
            // Center vertical rule ONLY in Row 1 (y = 230..260), leaving Row 0 merged!
            VectorStrokeSegment { x0: 250.0, y0: 230.0, x1: 250.0, y1: 260.0, stroke_width_pt: 1.0, color: RgbaColor::default() },
        ];

        let glyph_runs = vec![
            make_run(1, "Q4 Revenue Summary", 180.0, 210.0, 320.0, 222.0, 11.0, true),
            make_run(2, "North America", 110.0, 240.0, 200.0, 252.0, 10.0, false),
            make_run(3, "$1,420,000", 270.0, 240.0, 380.0, 252.0, 10.0, false),
        ];

        let scene = PageScene {
            page_index: 0,
            width_pt: 612.0,
            height_pt: 792.0,
            glyph_runs,
            stroke_segments: strokes,
            filled_rects: vec![],
        };

        let mut reconstructor = SpatialLayoutReconstructor::new(LayoutReconstructorConfig::default());
        let layout = reconstructor.reconstruct_page(&scene);
        let elements = layout.reading_order_elements();
        assert_eq!(elements.len(), 1);

        match elements[0] {
            LayoutElement::Table(table) => {
                assert!(table.is_ruled);
                assert_eq!(table.num_rows(), 2);
                assert_eq!(table.num_cols(), 2);
                assert_eq!(table.cells.len(), 3, "1 merged header cell + 2 data cells");
                assert_eq!(table.cells[0].row_idx, 0);
                assert_eq!(table.cells[0].col_span, 2);
                assert_eq!(table.cells[0].text(), "Q4 Revenue Summary");
                assert_eq!(table.cells[1].text(), "North America");
                assert_eq!(table.cells[2].text(), "$1,420,000");
            }
            _ => panic!("Expected ReconstructedTableGrid"),
        }
    }

    #[test]
    fn test_borderless_financial_table_detection() {
        // 3 rows x 3 columns of aligned financial data with no vector strokes
        let glyph_runs = vec![
            // Row 0
            make_run(1, "Item", 60.0, 300.0, 100.0, 312.0, 10.0, true),
            make_run(2, "Units", 240.0, 300.0, 280.0, 312.0, 10.0, true),
            make_run(3, "Total", 400.0, 300.0, 450.0, 312.0, 10.0, true),
            // Row 1
            make_run(4, "Enterprise Licenses", 60.0, 320.0, 165.0, 332.0, 10.0, false),
            make_run(5, "1,250", 240.0, 320.0, 275.0, 332.0, 10.0, false),
            make_run(6, "$375,000", 400.0, 320.0, 460.0, 332.0, 10.0, false),
            // Row 2
            make_run(7, "Cloud Storage Addon", 60.0, 340.0, 170.0, 352.0, 10.0, false),
            make_run(8, "840", 240.0, 340.0, 268.0, 352.0, 10.0, false),
            make_run(9, "$84,000", 400.0, 340.0, 452.0, 352.0, 10.0, false),
        ];

        let scene = PageScene {
            page_index: 0,
            width_pt: 612.0,
            height_pt: 792.0,
            glyph_runs,
            stroke_segments: vec![],
            filled_rects: vec![],
        };

        let mut reconstructor = SpatialLayoutReconstructor::new(LayoutReconstructorConfig::default());
        let layout = reconstructor.reconstruct_page(&scene);
        let elements = layout.reading_order_elements();
        assert_eq!(elements.len(), 1);

        match elements[0] {
            LayoutElement::Table(table) => {
                assert!(!table.is_ruled);
                assert_eq!(table.num_rows(), 3);
                assert_eq!(table.num_cols(), 3);
                assert_eq!(table.cells[4].text(), "1,250");
                assert_eq!(table.cells[8].text(), "$84,000");
            }
            _ => panic!("Expected borderless ReconstructedTableGrid"),
        }
    }
}
