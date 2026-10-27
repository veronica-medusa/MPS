use wps_core::color::Color;
use wps_core::geometry::Rect;
use crate::parser::{PdfPage, PdfTextElement, PathOp};

#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    Title,
    Heading(u8),
    Paragraph,
    ListItem,
    TableCell,
}

#[derive(Debug, Clone)]
pub struct TextLine {
    pub text: String,
    pub bounds: Rect,
    pub font_name: String,
    pub font_size: f64,
    pub color: Color,
    pub elements: Vec<PdfTextElement>,
}

#[derive(Debug, Clone)]
pub struct ReconstructedBlock {
    pub kind: BlockKind,
    pub bounds: Rect,
    pub lines: Vec<TextLine>,
    pub font_name: String,
    pub font_size: f64,
    pub is_bold: bool,
    pub is_italic: bool,
}

impl ReconstructedBlock {
    pub fn plain_text(&self) -> String {
        self.lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct ReconstructedCell {
    pub row: usize,
    pub col: usize,
    pub row_span: usize,
    pub col_span: usize,
    pub bounds: Rect,
    pub text: String,
    pub background: Option<Color>,
}

#[derive(Debug, Clone)]
pub struct ReconstructedTable {
    pub bounds: Rect,
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<ReconstructedCell>,
}

#[derive(Debug, Clone)]
pub struct SpatialPageLayout {
    pub page_index: usize,
    pub bounds: Rect,
    pub blocks: Vec<ReconstructedBlock>,
    pub tables: Vec<ReconstructedTable>,
}

pub struct SpatialReconstructor;

impl SpatialReconstructor {
    pub fn reconstruct_page(page: &PdfPage) -> SpatialPageLayout {
        // Step 1: Detect tables via vector line intersections
        let tables = Self::detect_tables(page);

        // Filter out text elements that fall inside detected table cells
        let mut non_table_elements = Vec::new();
        for elem in &page.text_elements {
            let center_x = elem.bounds.x + elem.bounds.width * 0.5;
            let center_y = elem.bounds.y + elem.bounds.height * 0.5;

            let in_table = tables.iter().any(|t| {
                center_x >= t.bounds.x && center_x <= t.bounds.x + t.bounds.width &&
                center_y >= t.bounds.y && center_y <= t.bounds.y + t.bounds.height
            });

            if !in_table {
                non_table_elements.push(elem.clone());
            }
        }

        // Step 2: Cluster glyph runs into text lines
        let lines = Self::cluster_elements_into_lines(&non_table_elements);

        // Step 3: Perform Recursive XY-Cut on text lines
        let blocks = Self::recursive_xy_cut(&lines, Rect::new(0.0, 0.0, page.width, page.height));

        SpatialPageLayout {
            page_index: page.page_index,
            bounds: Rect::new(0.0, 0.0, page.width, page.height),
            blocks,
            tables,
        }
    }

    /// Clusters adjacent text elements on the same horizontal baseline into cohesive lines
    fn cluster_elements_into_lines(elements: &[PdfTextElement]) -> Vec<TextLine> {
        if elements.is_empty() {
            return Vec::new();
        }

        // Sort elements primarily by Y baseline, secondarily by X
        let mut sorted = elements.to_vec();
        sorted.sort_by(|a, b| {
            let y_diff = a.baseline.partial_cmp(&b.baseline).unwrap();
            if (a.baseline - b.baseline).abs() < 3.0 {
                a.bounds.x.partial_cmp(&b.bounds.x).unwrap()
            } else {
                y_diff
            }
        });

        let mut lines: Vec<TextLine> = Vec::new();

        for elem in sorted {
            // Check if element can be appended to the current line
            let mut appended = false;
            if let Some(last_line) = lines.last_mut() {
                let baseline_diff = (elem.baseline - (last_line.bounds.y + last_line.bounds.height * 0.8)).abs();
                let horizontal_gap = elem.bounds.x - (last_line.bounds.x + last_line.bounds.width);

                if baseline_diff < 4.0 && horizontal_gap >= -5.0 && horizontal_gap < (elem.font_size * 2.0) {
                    if horizontal_gap > elem.font_size * 0.25 && !last_line.text.ends_with(' ') {
                        last_line.text.push(' ');
                    }
                    last_line.text.push_str(&elem.text);

                    // Expand bounds
                    let min_x = last_line.bounds.x.min(elem.bounds.x);
                    let min_y = last_line.bounds.y.min(elem.bounds.y);
                    let max_x = (last_line.bounds.x + last_line.bounds.width).max(elem.bounds.x + elem.bounds.width);
                    let max_y = (last_line.bounds.y + last_line.bounds.height).max(elem.bounds.y + elem.bounds.height);
                    last_line.bounds = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                    last_line.elements.push(elem.clone());
                    appended = true;
                }
            }

            if !appended {
                lines.push(TextLine {
                    text: elem.text.clone(),
                    bounds: elem.bounds,
                    font_name: elem.font_name.clone(),
                    font_size: elem.font_size,
                    color: elem.color,
                    elements: vec![elem],
                });
            }
        }

        lines
    }

    /// Recursive XY-Cut layout segmentation algorithm
    fn recursive_xy_cut(lines: &[TextLine], bounds: Rect) -> Vec<ReconstructedBlock> {
        if lines.is_empty() {
            return Vec::new();
        }

        // 1. Try Vertical Cut (Columns)
        if let Some((col_a_lines, col_b_lines, cut_x)) = Self::find_column_cut(lines, bounds) {
            let left_bounds = Rect::new(bounds.x, bounds.y, cut_x - bounds.x, bounds.height);
            let right_bounds = Rect::new(cut_x, bounds.y, bounds.x + bounds.width - cut_x, bounds.height);

            let mut blocks = Self::recursive_xy_cut(&col_a_lines, left_bounds);
            blocks.extend(Self::recursive_xy_cut(&col_b_lines, right_bounds));
            return blocks;
        }

        // 2. Perform Horizontal Segmentation into Paragraphs
        Self::segment_vertical_paragraphs(lines)
    }

    /// Finds a vertical whitespace valley separating multi-column layouts
    fn find_column_cut(lines: &[TextLine], bounds: Rect) -> Option<(Vec<TextLine>, Vec<TextLine>, f64)> {
        if lines.len() < 4 {
            return None;
        }

        let min_col_width = 100.0;
        let col_gap_threshold = 20.0;

        let left = bounds.x + min_col_width;
        let right = bounds.x + bounds.width - min_col_width;
        if right <= left {
            return None;
        }

        // Build 1D projection histogram along X axis (resolution = 2pt)
        let bins = 200;
        let bin_w = bounds.width / bins as f64;
        let mut hist = vec![0; bins];

        for line in lines {
            let start_bin = (((line.bounds.x - bounds.x) / bin_w).floor() as usize).min(bins - 1);
            let end_bin = ((((line.bounds.x + line.bounds.width) - bounds.x) / bin_w).ceil() as usize).min(bins);
            for b in start_bin..end_bin {
                hist[b] += 1;
            }
        }

        // Look for wide zero valley
        let start_idx = ((left - bounds.x) / bin_w) as usize;
        let end_idx = ((right - bounds.x) / bin_w).min(bins as f64) as usize;

        let mut longest_valley_len = 0;
        let mut longest_valley_start = 0;
        let mut cur_len = 0;
        let mut cur_start = 0;

        for i in start_idx..end_idx {
            if hist[i] == 0 {
                if cur_len == 0 {
                    cur_start = i;
                }
                cur_len += 1;
            } else {
                if cur_len > longest_valley_len {
                    longest_valley_len = cur_len;
                    longest_valley_start = cur_start;
                }
                cur_len = 0;
            }
        }
        if cur_len > longest_valley_len {
            longest_valley_len = cur_len;
            longest_valley_start = cur_start;
        }

        let valley_width = longest_valley_len as f64 * bin_w;
        if valley_width >= col_gap_threshold {
            let cut_x = bounds.x + (longest_valley_start as f64 + longest_valley_len as f64 * 0.5) * bin_w;

            let mut col_a = Vec::new();
            let mut col_b = Vec::new();

            for line in lines {
                if line.bounds.x + line.bounds.width * 0.5 < cut_x {
                    col_a.push(line.clone());
                } else {
                    col_b.push(line.clone());
                }
            }

            if !col_a.is_empty() && !col_b.is_empty() {
                return Some((col_a, col_b, cut_x));
            }
        }

        None
    }

    /// Segments lines in a single column into semantic blocks (Paragraphs, Headings, Lists)
    fn segment_vertical_paragraphs(lines: &[TextLine]) -> Vec<ReconstructedBlock> {
        let mut blocks = Vec::new();
        if lines.is_empty() {
            return blocks;
        }

        let mut sorted = lines.to_vec();
        sorted.sort_by(|a, b| a.bounds.y.partial_cmp(&b.bounds.y).unwrap());

        // Compute median font size for body text detection
        let mut font_sizes: Vec<f64> = sorted.iter().map(|l| l.font_size).collect();
        font_sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_font_size = font_sizes[(font_sizes.len() - 1) / 2];

        let mut current_block_lines: Vec<TextLine> = Vec::new();

        for line in sorted {
            let mut start_new_block = false;

            if let Some(prev) = current_block_lines.last() {
                let vertical_gap = line.bounds.y - (prev.bounds.y + prev.bounds.height);
                let line_height = prev.font_size * 1.3;

                // Thresholds: larger gap indicates paragraph break
                if vertical_gap > line_height * 0.8 {
                    start_new_block = true;
                } else if (line.font_size - prev.font_size).abs() > 2.0 {
                    // Font size change indicates heading or section change
                    start_new_block = true;
                } else if line.text.trim_start().starts_with("•") || line.text.trim_start().starts_with("-") ||
                          line.text.trim_start().chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) && line.text.contains('.') {
                    start_new_block = true;
                }
            }

            if start_new_block && !current_block_lines.is_empty() {
                blocks.push(Self::build_block(&current_block_lines, median_font_size));
                current_block_lines.clear();
            }

            current_block_lines.push(line);
        }

        if !current_block_lines.is_empty() {
            blocks.push(Self::build_block(&current_block_lines, median_font_size));
        }

        blocks
    }

    fn build_block(lines: &[TextLine], body_font_size: f64) -> ReconstructedBlock {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        let font_size = lines[0].font_size;
        let font_name = lines[0].font_name.clone();
        let is_bold = font_name.to_lowercase().contains("bold");
        let is_italic = font_name.to_lowercase().contains("italic");

        for l in lines {
            min_x = min_x.min(l.bounds.x);
            min_y = min_y.min(l.bounds.y);
            max_x = max_x.max(l.bounds.x + l.bounds.width);
            max_y = max_y.max(l.bounds.y + l.bounds.height);
        }

        let first_text = lines[0].text.trim();
        let kind = if font_size >= body_font_size * 1.8 {
            BlockKind::Title
        } else if font_size >= body_font_size * 1.4 {
            BlockKind::Heading(1)
        } else if font_size >= body_font_size * 1.15 || is_bold {
            BlockKind::Heading(2)
        } else if first_text.starts_with("•") || first_text.starts_with("-") || first_text.starts_with("*") {
            BlockKind::ListItem
        } else {
            BlockKind::Paragraph
        };

        ReconstructedBlock {
            kind,
            bounds: Rect::new(min_x, min_y, (max_x - min_x).max(10.0), (max_y - min_y).max(10.0)),
            lines: lines.to_vec(),
            font_name,
            font_size,
            is_bold,
            is_italic,
        }
    }

    /// Table detection using vector line intersection and closed-polygon analysis
    fn detect_tables(page: &PdfPage) -> Vec<ReconstructedTable> {
        let mut horizontal_lines: Vec<(f64, f64, f64)> = Vec::new(); // (y, x_min, x_max)
        let mut vertical_lines: Vec<(f64, f64, f64)> = Vec::new();   // (x, y_min, y_max)

        for path in &page.vector_paths {
            let mut cur_point = wps_core::geometry::Point::new(0.0, 0.0);
            for op in &path.operations {
                match op {
                    PathOp::MoveTo(p) => cur_point = *p,
                    PathOp::LineTo(p) => {
                        let dx = (p.x - cur_point.x).abs();
                        let dy = (p.y - cur_point.y).abs();
                        if dy < 2.0 && dx > 15.0 {
                            horizontal_lines.push((cur_point.y, cur_point.x.min(p.x), cur_point.x.max(p.x)));
                        } else if dx < 2.0 && dy > 10.0 {
                            vertical_lines.push((cur_point.x, cur_point.y.min(p.y), cur_point.y.max(p.y)));
                        }
                        cur_point = *p;
                    }
                    PathOp::Rectangle(r) => {
                        // Rectangles in tables can represent cell borders or table borders
                        if r.width > 20.0 && r.height > 10.0 {
                            horizontal_lines.push((r.y, r.x, r.x + r.width));
                            horizontal_lines.push((r.y + r.height, r.x, r.x + r.width));
                            vertical_lines.push((r.x, r.y, r.y + r.height));
                            vertical_lines.push((r.x + r.width, r.y, r.y + r.height));
                        }
                    }
                    _ => {}
                }
            }
        }

        if horizontal_lines.len() < 2 || vertical_lines.len() < 2 {
            return Vec::new();
        }

        // Deduplicate and cluster horizontal Y coordinates
        horizontal_lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut y_clusters: Vec<f64> = Vec::new();
        for (y, _, _) in &horizontal_lines {
            if let Some(last_y) = y_clusters.last() {
                if (y - last_y).abs() < 3.0 {
                    continue;
                }
            }
            y_clusters.push(*y);
        }

        // Deduplicate and cluster vertical X coordinates
        vertical_lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut x_clusters: Vec<f64> = Vec::new();
        for (x, _, _) in &vertical_lines {
            if let Some(last_x) = x_clusters.last() {
                if (x - last_x).abs() < 3.0 {
                    continue;
                }
            }
            x_clusters.push(*x);
        }

        if y_clusters.len() < 2 || x_clusters.len() < 2 {
            return Vec::new();
        }

        let num_rows = y_clusters.len() - 1;
        let num_cols = x_clusters.len() - 1;

        let table_x = x_clusters[0];
        let table_y = y_clusters[0];
        let table_w = x_clusters.last().unwrap() - table_x;
        let table_h = y_clusters.last().unwrap() - table_y;

        let mut cells = Vec::new();

        for r in 0..num_rows {
            let row_top = y_clusters[r];
            let row_bot = y_clusters[r + 1];

            for c in 0..num_cols {
                let col_left = x_clusters[c];
                let col_right = x_clusters[c + 1];

                let cell_rect = Rect::new(col_left, row_top, col_right - col_left, row_bot - row_top);

                // Collect text within this cell
                let mut cell_text_pieces = Vec::new();
                for elem in &page.text_elements {
                    let cx = elem.bounds.x + elem.bounds.width * 0.5;
                    let cy = elem.bounds.y + elem.bounds.height * 0.5;
                    if cx >= cell_rect.x && cx <= cell_rect.x + cell_rect.width &&
                       cy >= cell_rect.y && cy <= cell_rect.y + cell_rect.height {
                        cell_text_pieces.push(elem.text.as_str());
                    }
                }

                cells.push(ReconstructedCell {
                    row: r,
                    col: c,
                    row_span: 1,
                    col_span: 1,
                    bounds: cell_rect,
                    text: cell_text_pieces.join(" ").trim().to_string(),
                    background: None,
                });
            }
        }

        vec![ReconstructedTable {
            bounds: Rect::new(table_x, table_y, table_w, table_h),
            rows: num_rows,
            cols: num_cols,
            cells,
        }]
    }
}
