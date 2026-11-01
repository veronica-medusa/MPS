//! Cross-Engine PDF-to-Office AST Converter (`wps-pdf`)
//!
//! Transforms `ReconstructedPageLayout` (produced by `layout_reconstructor.rs`)
//! directly into native AST structures for:
//! 1. `wps-writer` (`.docx` Word Processor AST in Twips)
//! 2. `wps-sheets` (`.xlsx` Spreadsheet Workbook AST with typed numeric/currency/date cells)
//! 3. `wps-slides` (`.pptx` Presentation Scene Graph AST in EMUs)

use crate::layout_reconstructor::{
    EditableParagraphBlock, LayoutElement, ReconstructedPageLayout,
    ReconstructedTableGrid, ReconstructedTextLine, RgbaColor, SemanticRole, TextAlignment,
};
use crate::parser::{PdfPage, PdfTextElement};
use crate::spatial::{BlockKind, SpatialReconstructor};
use crate::PdfDocument;
use serde::{Deserialize, Serialize};
use wps_core::geometry::Rect;
use wps_core::units::Twips;
use wps_sheets::{CellCoordinate, CellValue, SparseMatrix, Workbook, Worksheet};
use wps_slides::scene_graph::{PresentationDeck, Slide};
use wps_slides::shape::{ParametricShape, ShapeFill, ShapeGeometryType};

/// 1 PDF point (1/72 inch) = 20 Twips (1/1440 inch)
pub const TWIPS_PER_PT: f32 = 20.0;
/// 1 PDF point (1/72 inch) = 12,700 English Metric Units (1/914,400 inch)
pub const EMUS_PER_PT: f64 = 12_700.0;

#[inline]
pub fn pt_to_twips(pt: f32) -> u32 {
    (pt * TWIPS_PER_PT).round().max(0.0) as u32
}

#[inline]
pub fn pt_to_signed_twips(pt: f32) -> i32 {
    (pt * TWIPS_PER_PT).round() as i32
}

#[inline]
pub fn pt_to_emu(pt: f32) -> i64 {
    ((pt as f64) * EMUS_PER_PT).round() as i64
}

// ============================================================================
// 1. TARGET AST STRUCTURES FOR WPS WRITER (`.DOCX`)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterMarginsTwips {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
    pub header: u32,
    pub footer: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterTextRunAst {
    pub text: String,
    pub font_family: String,
    /// Font size in ECMA-376 half-points (`w:sz`, e.g., `24` = `12pt`).
    pub font_size_half_pts: u16,
    pub is_bold: bool,
    pub is_italic: bool,
    pub color: RgbaColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterListBindingAst {
    pub marker: String,
    pub ordered: bool,
    pub level: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterParagraphAst {
    pub style_id: String, // "Title", "Heading 1".."Heading 6", "List Paragraph", "Caption", "Normal"
    pub alignment: TextAlignment,
    pub first_line_indent_twips: i32,
    pub left_indent_twips: i32,
    pub right_indent_twips: i32,
    pub space_before_twips: u32,
    pub space_after_twips: u32,
    /// Line spacing in 240ths of a line (`240` = 1.0x single spacing).
    pub line_spacing_240ths: u32,
    pub list_binding: Option<WriterListBindingAst>,
    pub runs: Vec<WriterTextRunAst>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriterVMergeState {
    None,
    Restart,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterTableCellAst {
    pub row_idx: usize,
    pub col_idx: usize,
    /// Horizontal span (`w:gridSpan`).
    pub grid_span: u16,
    /// Vertical merge state (`w:vMerge`).
    pub v_merge: WriterVMergeState,
    pub width_twips: u32,
    pub background_shading: Option<RgbaColor>,
    pub has_borders: bool,
    pub paragraphs: Vec<WriterParagraphAst>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterTableRowAst {
    pub height_twips: u32,
    pub is_header_repeat: bool,
    pub cells: Vec<WriterTableCellAst>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterTableAst {
    pub id: u64,
    pub is_ruled: bool,
    pub grid_col_widths_twips: Vec<u32>,
    pub rows: Vec<WriterTableRowAst>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriterBlockAst {
    Paragraph(WriterParagraphAst),
    Table(WriterTableAst),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterSectionAst {
    pub page_index: usize,
    pub page_width_twips: u32,
    pub page_height_twips: u32,
    pub margins_twips: WriterMarginsTwips,
    pub column_count: u8,
    pub column_space_twips: u32,
    pub headers: Vec<WriterParagraphAst>,
    pub footers: Vec<WriterParagraphAst>,
    pub blocks: Vec<WriterBlockAst>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertedWriterDocument {
    pub sections: Vec<WriterSectionAst>,
}

// ============================================================================
// 2. TARGET AST STRUCTURES FOR WPS SPREADSHEETS (`.XLSX`)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfToSheetsMode {
    /// Each reconstructed table becomes its own dedicated worksheet (`Table 1`, `Table 2`, ...).
    OneSheetPerTable,
    /// All tables and structured lines on a PDF page are stacked onto one worksheet per page.
    OneSheetPerPage,
}

pub type PdfToExcelMode = PdfToSheetsMode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SheetTypedValue {
    Empty,
    Number(f64),
    Boolean(bool),
    DateIso(String),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvertedSheetCell {
    pub row: u32,
    pub col: u16,
    pub value: SheetTypedValue,
    pub number_format: String,
    pub is_bold: bool,
    pub alignment: TextAlignment,
    pub background_color: Option<RgbaColor>,
    pub has_border: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheetMergedRange {
    pub start_row: u32,
    pub start_col: u16,
    pub end_row: u32,
    pub end_col: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvertedWorksheetAst {
    pub name: String,
    pub col_widths_px: Vec<u16>,
    pub cells: Vec<ConvertedSheetCell>,
    pub merged_ranges: Vec<SheetMergedRange>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConvertedWorkbookAst {
    pub sheets: Vec<ConvertedWorksheetAst>,
}

// ============================================================================
// 3. TARGET AST STRUCTURES FOR WPS PRESENTATION (`.PPTX`)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvertedSlideNodeAst {
    TextBox {
        node_id: u64,
        x_emu: i64,
        y_emu: i64,
        width_emu: i64,
        height_emu: i64,
        role_label: String,
        text: String,
        font_name: String,
        font_size_pt: u16,
        is_bold: bool,
        color: RgbaColor,
    },
    Table {
        node_id: u64,
        x_emu: i64,
        y_emu: i64,
        width_emu: i64,
        height_emu: i64,
        num_rows: usize,
        num_cols: usize,
        cell_matrix: Vec<Vec<String>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertedSlideAst {
    pub slide_index: usize,
    pub nodes: Vec<ConvertedSlideNodeAst>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvertedPresentationAst {
    pub width_emu: i64,
    pub height_emu: i64,
    pub slides: Vec<ConvertedSlideAst>,
}

// ============================================================================
// 4. CORE CONVERSION IMPLEMENTATION (`PdfToOfficeConverter`)
// ============================================================================

pub struct PdfToOfficeConverter;

impl PdfToOfficeConverter {
    // ========================================================================
    // 4A. PDF -> WPS WRITER (`.DOCX`) CONVERTER
    // ========================================================================

    /// Converts a slice of `ReconstructedPageLayout` pages into a native `ConvertedWriterDocument`.
    pub fn convert_pdf_to_writer(pages: &[ReconstructedPageLayout]) -> ConvertedWriterDocument {
        let mut sections = Vec::with_capacity(pages.len());

        for page in pages {
            let page_w_twips = pt_to_twips(page.width_pt.max(72.0));
            let page_h_twips = pt_to_twips(page.height_pt.max(72.0));

            let margins_twips = Self::infer_page_margins_twips(page);
            let (column_count, column_space_twips) = Self::infer_section_columns(page);

            let headers: Vec<WriterParagraphAst> = page
                .headers
                .iter()
                .map(|p| Self::convert_paragraph_to_writer_ast(p, "Header"))
                .collect();

            let footers: Vec<WriterParagraphAst> = page
                .footers
                .iter()
                .map(|p| Self::convert_paragraph_to_writer_ast(p, "Footer"))
                .collect();

            let mut blocks = Vec::new();
            for element in page.reading_order_elements() {
                match element {
                    LayoutElement::Paragraph(p) => {
                        let style_id = Self::map_role_to_writer_style(&p.role);
                        blocks.push(WriterBlockAst::Paragraph(
                            Self::convert_paragraph_to_writer_ast(p, style_id),
                        ));
                    }
                    LayoutElement::Table(t) => {
                        blocks.push(WriterBlockAst::Table(Self::convert_table_to_writer_ast(t)));
                    }
                }
            }

            sections.push(WriterSectionAst {
                page_index: page.page_index,
                page_width_twips: page_w_twips,
                page_height_twips: page_h_twips,
                margins_twips,
                column_count,
                column_space_twips,
                headers,
                footers,
                blocks,
            });
        }

        ConvertedWriterDocument { sections }
    }

    fn map_role_to_writer_style(role: &SemanticRole) -> &'static str {
        match role {
            SemanticRole::Title => "Title",
            SemanticRole::Heading(1) => "Heading 1",
            SemanticRole::Heading(2) => "Heading 2",
            SemanticRole::Heading(3) => "Heading 3",
            SemanticRole::Heading(4) => "Heading 4",
            SemanticRole::Heading(5) => "Heading 5",
            SemanticRole::Heading(_) => "Heading 6",
            SemanticRole::ListItem { .. } => "List Paragraph",
            SemanticRole::Caption => "Caption",
            SemanticRole::BodyParagraph => "Normal",
        }
    }

    fn convert_paragraph_to_writer_ast(
        para: &EditableParagraphBlock,
        style_id: &str,
    ) -> WriterParagraphAst {
        let list_binding = match &para.role {
            SemanticRole::ListItem {
                marker,
                ordered,
                level,
            } => Some(WriterListBindingAst {
                marker: marker.clone(),
                ordered: *ordered,
                level: *level,
            }),
            _ => None,
        };

        // Build character runs from word spans, coalescing adjacent words that share identical formatting
        let mut runs: Vec<WriterTextRunAst> = Vec::new();
        for (line_idx, line) in para.lines.iter().enumerate() {
            for (word_idx, word) in line.words.iter().enumerate() {
                let mut token = word.text.clone();
                if line_idx > 0 && word_idx == 0 {
                    // Strip soft hyphen on line wrap or prepend space between wrapped lines
                    if let Some(prev_run) = runs.last_mut() {
                        if prev_run.text.ends_with('-')
                            && token.chars().next().is_some_and(|c| c.is_lowercase())
                        {
                            prev_run.text.pop();
                        } else if !prev_run.text.ends_with(' ') {
                            prev_run.text.push(' ');
                        }
                    }
                } else if word_idx > 0 {
                    if let Some(prev_run) = runs.last_mut() {
                        if !prev_run.text.ends_with(' ') {
                            prev_run.text.push(' ');
                        }
                    }
                }

                // Strip leading bullet/number marker from the first word of a ListItem paragraph
                if line_idx == 0 && word_idx == 0 {
                    if let Some(lb) = &list_binding {
                        if let Some(stripped) = token.strip_prefix(&lb.marker) {
                            token = stripped.trim_start().to_string();
                        }
                    }
                }

                if token.is_empty() {
                    continue;
                }

                let size_half_pts = ((word.font_size_pt * 2.0).round().max(8.0)) as u16;
                if let Some(last) = runs.last_mut() {
                    if last.font_family == word.font_name
                        && last.font_size_half_pts == size_half_pts
                        && last.is_bold == word.is_bold
                        && last.is_italic == word.is_italic
                        && last.color == word.color
                    {
                        last.text.push_str(&token);
                        continue;
                    }
                }

                runs.push(WriterTextRunAst {
                    text: token,
                    font_family: word.font_name.clone(),
                    font_size_half_pts: size_half_pts,
                    is_bold: word.is_bold,
                    is_italic: word.is_italic,
                    color: word.color,
                });
            }
        }

        if runs.is_empty() {
            runs.push(WriterTextRunAst {
                text: para.full_text(),
                font_family: para.font_name.clone(),
                font_size_half_pts: (para.font_size_pt * 2.0).round() as u16,
                is_bold: para.is_bold,
                is_italic: para.is_italic,
                color: para.color,
            });
        }

        let space_after_twips = match para.role {
            SemanticRole::Title => 240,
            SemanticRole::Heading(_) => 160,
            _ => 120,
        };

        WriterParagraphAst {
            style_id: style_id.to_string(),
            alignment: para.alignment,
            first_line_indent_twips: pt_to_signed_twips(para.first_line_indent_pt),
            left_indent_twips: pt_to_signed_twips(para.left_indent_pt),
            right_indent_twips: pt_to_signed_twips(para.right_indent_pt),
            space_before_twips: 60,
            space_after_twips,
            line_spacing_240ths: ((para.line_spacing_ratio * 240.0).round() as u32).clamp(200, 720),
            list_binding,
            runs,
        }
    }

    /// Converts a `ReconstructedTableGrid` into a `WriterTableAst`, expanding horizontal
    /// (`w:gridSpan`) and vertical (`w:vMerge` `Restart` / `Continue`) merges into a valid
    /// ECMA-376 row/cell structure.
    fn convert_table_to_writer_ast(table: &ReconstructedTableGrid) -> WriterTableAst {
        let num_rows = table.num_rows();
        let num_cols = table.num_cols();

        let grid_col_widths_twips: Vec<u32> = table
            .col_boundaries_pt
            .windows(2)
            .map(|w| pt_to_twips((w[1] - w[0]).max(10.0)))
            .collect();

        // Build a 2D lookup of cells by `(row, col)` to emit ECMA-376 `vMerge: Continue` cells
        #[derive(Clone)]
        enum SlotState {
            Empty,
            Origin(usize), // Index into `table.cells`
            SpannedHoriz,
            SpannedVert {
                origin_cell_idx: usize,
                col_span: u16,
                width_twips: u32,
            },
        }

        let mut matrix = vec![vec![SlotState::Empty; num_cols]; num_rows];

        for (cell_idx, cell) in table.cells.iter().enumerate() {
            let r0 = cell.row_idx.min(num_rows.saturating_sub(1));
            let c0 = cell.col_idx.min(num_cols.saturating_sub(1));
            let r_end = (r0 + cell.row_span).min(num_rows);
            let c_end = (c0 + cell.col_span).min(num_cols);
            let c_span = (c_end - c0).max(1) as u16;
            let width_twips: u32 = grid_col_widths_twips[c0..c_end].iter().sum();

            matrix[r0][c0] = SlotState::Origin(cell_idx);
            for c in (c0 + 1)..c_end {
                matrix[r0][c] = SlotState::SpannedHoriz;
            }

            for r in (r0 + 1)..r_end {
                matrix[r][c0] = SlotState::SpannedVert {
                    origin_cell_idx: cell_idx,
                    col_span: c_span,
                    width_twips,
                };
                for c in (c0 + 1)..c_end {
                    matrix[r][c] = SlotState::SpannedHoriz;
                }
            }
        }

        let mut rows_ast = Vec::with_capacity(num_rows);
        for r in 0..num_rows {
            let row_h_pt = table
                .row_boundaries_pt
                .get(r + 1)
                .zip(table.row_boundaries_pt.get(r))
                .map(|(y1, y0)| (y1 - y0).max(12.0))
                .unwrap_or(20.0);

            let mut row_cells = Vec::new();
            for c in 0..num_cols {
                match &matrix[r][c] {
                    SlotState::SpannedHoriz => continue,
                    SlotState::Origin(cell_idx) => {
                        let cell = &table.cells[*cell_idx];
                        let c_end = (c + cell.col_span).min(num_cols);
                        let width_twips: u32 = grid_col_widths_twips[c..c_end].iter().sum();
                        let v_merge = if cell.row_span > 1 {
                            WriterVMergeState::Restart
                        } else {
                            WriterVMergeState::None
                        };

                        let paragraphs = Self::convert_cell_lines_to_paragraphs(
                            &cell.lines,
                            cell.alignment,
                            r == 0,
                        );

                        // Apply Mercury Lavender (#edecfb) default header tint on ruled header rows without explicit fill
                        let background_shading = cell.background_color.or_else(|| {
                            if r == 0 && table.is_ruled {
                                Some(RgbaColor {
                                    r: 237,
                                    g: 236,
                                    b: 251,
                                    a: 255,
                                })
                            } else {
                                None
                            }
                        });

                        row_cells.push(WriterTableCellAst {
                            row_idx: r,
                            col_idx: c,
                            grid_span: cell.col_span.max(1) as u16,
                            v_merge,
                            width_twips,
                            background_shading,
                            has_borders: table.is_ruled,
                            paragraphs,
                        });
                    }
                    SlotState::SpannedVert {
                        origin_cell_idx,
                        col_span,
                        width_twips,
                    } => {
                        let origin = &table.cells[*origin_cell_idx];
                        row_cells.push(WriterTableCellAst {
                            row_idx: r,
                            col_idx: c,
                            grid_span: *col_span,
                            v_merge: WriterVMergeState::Continue,
                            width_twips: *width_twips,
                            background_shading: origin.background_color,
                            has_borders: table.is_ruled,
                            paragraphs: vec![],
                        });
                    }
                    SlotState::Empty => {
                        let width_twips = grid_col_widths_twips.get(c).copied().unwrap_or(1440);
                        row_cells.push(WriterTableCellAst {
                            row_idx: r,
                            col_idx: c,
                            grid_span: 1,
                            v_merge: WriterVMergeState::None,
                            width_twips,
                            background_shading: None,
                            has_borders: table.is_ruled,
                            paragraphs: vec![],
                        });
                    }
                }
            }

            rows_ast.push(WriterTableRowAst {
                height_twips: pt_to_twips(row_h_pt),
                is_header_repeat: r == 0,
                cells: row_cells,
            });
        }

        WriterTableAst {
            id: table.id,
            is_ruled: table.is_ruled,
            grid_col_widths_twips,
            rows: rows_ast,
        }
    }

    fn convert_cell_lines_to_paragraphs(
        lines: &[ReconstructedTextLine],
        alignment: TextAlignment,
        is_header_row: bool,
    ) -> Vec<WriterParagraphAst> {
        if lines.is_empty() {
            return vec![];
        }
        lines
            .iter()
            .map(|l| WriterParagraphAst {
                style_id: "Table Normal".to_string(),
                alignment,
                first_line_indent_twips: 0,
                left_indent_twips: 0,
                right_indent_twips: 0,
                space_before_twips: 20,
                space_after_twips: 20,
                line_spacing_240ths: 240,
                list_binding: None,
                runs: vec![WriterTextRunAst {
                    text: l.text.trim().to_string(),
                    font_family: l.dominant_font_name.clone(),
                    font_size_half_pts: (l.dominant_font_size_pt * 2.0).round() as u16,
                    is_bold: l.is_bold || is_header_row,
                    is_italic: l.is_italic,
                    color: l.dominant_color,
                }],
            })
            .collect()
    }

    fn infer_page_margins_twips(page: &ReconstructedPageLayout) -> WriterMarginsTwips {
        if page.regions.is_empty() {
            return WriterMarginsTwips {
                top: 1440,
                bottom: 1440,
                left: 1440,
                right: 1440,
                header: 720,
                footer: 720,
            };
        }

        let union_bbox = page
            .regions
            .iter()
            .fold(page.regions[0].bbox, |acc, r| acc.union(&r.bbox));

        let left_pt = union_bbox.x0.clamp(18.0, 144.0);
        let right_pt = (page.width_pt - union_bbox.x1).clamp(18.0, 144.0);
        let top_pt = union_bbox.y0.clamp(36.0, 144.0);
        let bottom_pt = (page.height_pt - union_bbox.y1).clamp(36.0, 144.0);

        WriterMarginsTwips {
            top: pt_to_twips(top_pt),
            bottom: pt_to_twips(bottom_pt),
            left: pt_to_twips(left_pt),
            right: pt_to_twips(right_pt),
            header: 720,
            footer: 720,
        }
    }

    fn infer_section_columns(page: &ReconstructedPageLayout) -> (u8, u32) {
        // Detect if two or more regions sit side-by-side in the same vertical Y band
        let mut parallel_cols = 1u8;
        let mut gutter_pt = 36.0f32;

        for i in 0..page.regions.len() {
            let mut side_by_side = 1u8;
            for j in (i + 1)..page.regions.len() {
                let r_a = &page.regions[i].bbox;
                let r_b = &page.regions[j].bbox;
                let y_overlap = (r_a.y1.min(r_b.y1) - r_a.y0.max(r_b.y0)).max(0.0);
                let x_gap = (r_b.x0 - r_a.x1).max(r_a.x0 - r_b.x1);
                if y_overlap > 20.0 && x_gap >= 10.0 {
                    side_by_side += 1;
                    gutter_pt = x_gap;
                }
            }
            parallel_cols = parallel_cols.max(side_by_side);
        }

        (parallel_cols.min(4), pt_to_twips(gutter_pt))
    }

    // ========================================================================
    // 4B. PDF -> WPS SPREADSHEETS (`.XLSX`) CONVERTER
    // ========================================================================

    /// Converts reconstructed tables (and structured key-value lines) from `pages` into a
    /// native `ConvertedWorkbookAst` with typed numbers, currencies, percentages, and merged cells.
    pub fn convert_pdf_to_sheets(
        pages: &[ReconstructedPageLayout],
        mode: PdfToSheetsMode,
    ) -> ConvertedWorkbookAst {
        let mut sheets = Vec::new();
        let mut table_counter = 1usize;

        match mode {
            PdfToSheetsMode::OneSheetPerTable => {
                for page in pages {
                    for element in page.reading_order_elements() {
                        if let LayoutElement::Table(table) = element {
                            let sheet_name =
                                format!("Page{}_Table{}", page.page_index + 1, table_counter);
                            table_counter += 1;
                            sheets.push(Self::convert_table_to_worksheet(&sheet_name, table));
                        }
                    }
                }
            }
            PdfToSheetsMode::OneSheetPerPage => {
                for page in pages {
                    let sheet_name = format!("Page_{}", page.page_index + 1);
                    let mut cells = Vec::new();
                    let mut merged_ranges = Vec::new();
                    let mut current_row = 0u32;
                    let mut max_cols = 1usize;

                    for element in page.reading_order_elements() {
                        match element {
                            LayoutElement::Paragraph(para) => {
                                let text = para.full_text();
                                if !text.is_empty() {
                                    let (typed_val, num_fmt) =
                                        Self::parse_pdf_cell_to_typed_value(&text);
                                    cells.push(ConvertedSheetCell {
                                        row: current_row,
                                        col: 0,
                                        value: typed_val,
                                        number_format: num_fmt,
                                        is_bold: para.is_bold
                                            || matches!(
                                                para.role,
                                                SemanticRole::Title | SemanticRole::Heading(_)
                                            ),
                                        alignment: para.alignment,
                                        background_color: None,
                                        has_border: false,
                                    });
                                    current_row += 1;
                                }
                            }
                            LayoutElement::Table(table) => {
                                max_cols = max_cols.max(table.num_cols());
                                let sub_sheet = Self::convert_table_to_worksheet("tmp", table);
                                for mut c in sub_sheet.cells {
                                    c.row += current_row;
                                    cells.push(c);
                                }
                                for mut m in sub_sheet.merged_ranges {
                                    m.start_row += current_row;
                                    m.end_row += current_row;
                                    merged_ranges.push(m);
                                }
                                current_row += (table.num_rows() as u32) + 1;
                            }
                        }
                    }

                    sheets.push(ConvertedWorksheetAst {
                        name: sheet_name,
                        col_widths_px: vec![140; max_cols],
                        cells,
                        merged_ranges,
                    });
                }
            }
        }

        // Ensure workbook always has at least 1 valid worksheet
        if sheets.is_empty() {
            sheets.push(ConvertedWorksheetAst {
                name: "Sheet1".to_string(),
                col_widths_px: vec![120; 4],
                cells: vec![],
                merged_ranges: vec![],
            });
        }

        ConvertedWorkbookAst { sheets }
    }

    fn convert_table_to_worksheet(
        sheet_name: &str,
        table: &ReconstructedTableGrid,
    ) -> ConvertedWorksheetAst {
        let col_widths_px: Vec<u16> = table
            .col_boundaries_pt
            .windows(2)
            .map(|w| ((w[1] - w[0]) * 1.333).round().clamp(64.0, 480.0) as u16)
            .collect();

        let mut cells = Vec::with_capacity(table.cells.len());
        let mut merged_ranges = Vec::new();

        for tc in &table.cells {
            let raw_text = tc.text();
            let (typed_val, num_fmt) = Self::parse_pdf_cell_to_typed_value(&raw_text);
            let is_bold = tc.row_idx == 0 || tc.lines.iter().any(|l| l.is_bold);

            cells.push(ConvertedSheetCell {
                row: tc.row_idx as u32,
                col: tc.col_idx as u16,
                value: typed_val,
                number_format: num_fmt,
                is_bold,
                alignment: tc.alignment,
                background_color: tc.background_color,
                has_border: table.is_ruled,
            });

            if tc.row_span > 1 || tc.col_span > 1 {
                merged_ranges.push(SheetMergedRange {
                    start_row: tc.row_idx as u32,
                    start_col: tc.col_idx as u16,
                    end_row: (tc.row_idx + tc.row_span - 1) as u32,
                    end_col: (tc.col_idx + tc.col_span - 1) as u16,
                });
            }
        }

        ConvertedWorksheetAst {
            name: sheet_name.to_string(),
            col_widths_px,
            cells,
            merged_ranges,
        }
    }

    /// Parses formatted PDF cell strings (`"$1,420,000"`, `"($450.25)"`, `"34.2%"`, `"1,250"`,
    /// `"2026-10-30"`) into strongly typed `SheetTypedValue`s and Excel/WPS number format masks.
    pub fn parse_pdf_cell_to_typed_value(raw: &str) -> (SheetTypedValue, String) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return (SheetTypedValue::Empty, "General".to_string());
        }

        // 1. Boolean check
        if trimmed.eq_ignore_ascii_case("true") {
            return (SheetTypedValue::Boolean(true), "General".to_string());
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return (SheetTypedValue::Boolean(false), "General".to_string());
        }

        // 2. Percentage check (e.g. "34.2%" -> 0.342 with "0.0%")
        if let Some(pct_body) = trimmed.strip_suffix('%') {
            let clean_pct = pct_body.trim().replace(',', "");
            if let Ok(num) = clean_pct.parse::<f64>() {
                let decimals = clean_pct
                    .split('.')
                    .nth(1)
                    .map(|d| d.len())
                    .unwrap_or(0);
                let fmt = if decimals > 0 {
                    format!("0.{}%", "0".repeat(decimals.min(4)))
                } else {
                    "0%".to_string()
                };
                return (SheetTypedValue::Number(num / 100.0), fmt);
            }
        }

        // 3. Currency & Accounting negative parentheses check (e.g. "$1,420,000" or "($45,000.50)")
        let is_paren_neg = trimmed.starts_with('(') && trimmed.ends_with(')');
        let inner = if is_paren_neg {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        let currency_symbol = inner
            .chars()
            .next()
            .filter(|&c| c == '$' || c == '€' || c == '£' || c == '₹' || c == '¥');

        if let Some(sym) = currency_symbol {
            let num_part = inner[sym.len_utf8()..].trim().replace(',', "");
            if let Ok(mut val) = num_part.parse::<f64>() {
                if is_paren_neg {
                    val = -val;
                }
                let has_decimals = num_part.contains('.');
                let fmt = if has_decimals {
                    format!("{}#,##0.00;({}#,##0.00)", sym, sym)
                } else {
                    format!("{}#,##0;({}#,##0)", sym, sym)
                };
                return (SheetTypedValue::Number(val), fmt);
            }
        }

        // 4. Standard numeric / thousands-separated number check
        let has_comma = inner.contains(',');
        let clean_num = inner.replace(',', "");
        if clean_num
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '+')
            && clean_num.chars().any(|c| c.is_ascii_digit())
        {
            if let Ok(mut val) = clean_num.parse::<f64>() {
                if is_paren_neg {
                    val = -val;
                }
                let has_decimals = clean_num.contains('.');
                let fmt = match (has_comma, has_decimals) {
                    (true, true) => "#,##0.00".to_string(),
                    (true, false) => "#,##0".to_string(),
                    (false, true) => "0.00".to_string(),
                    (false, false) => "0".to_string(),
                };
                return (SheetTypedValue::Number(val), fmt);
            }
        }

        // 5. ISO Date check (`YYYY-MM-DD`)
        if trimmed.len() == 10
            && trimmed.as_bytes()[4] == b'-'
            && trimmed.as_bytes()[7] == b'-'
            && trimmed[0..4].chars().all(|c| c.is_ascii_digit())
            && trimmed[5..7].chars().all(|c| c.is_ascii_digit())
            && trimmed[8..10].chars().all(|c| c.is_ascii_digit())
        {
            return (
                SheetTypedValue::DateIso(trimmed.to_string()),
                "yyyy-mm-dd".to_string(),
            );
        }

        (SheetTypedValue::Text(trimmed.to_string()), "@".to_string())
    }

    // ========================================================================
    // 4C. PDF -> WPS PRESENTATION (`.PPTX`) CONVERTER
    // ========================================================================

    /// Converts `pages` into a native `ConvertedPresentationAst` with EMU coordinates.
    pub fn convert_pdf_to_slides(pages: &[ReconstructedPageLayout]) -> ConvertedPresentationAst {
        let first_w = pages.first().map(|p| p.width_pt).unwrap_or(720.0);
        let first_h = pages.first().map(|p| p.height_pt).unwrap_or(540.0);

        let mut next_node_id = 1u64;
        let mut slides = Vec::with_capacity(pages.len());

        for page in pages {
            let mut nodes = Vec::new();

            for element in page.reading_order_elements() {
                match element {
                    LayoutElement::Paragraph(p) => {
                        let role_label = Self::map_role_to_writer_style(&p.role).to_string();
                        nodes.push(ConvertedSlideNodeAst::TextBox {
                            node_id: {
                                let id = next_node_id;
                                next_node_id += 1;
                                id
                            },
                            x_emu: pt_to_emu(p.bbox.x0),
                            y_emu: pt_to_emu(p.bbox.y0),
                            width_emu: pt_to_emu(p.bbox.width().max(36.0)),
                            height_emu: pt_to_emu(p.bbox.height().max(18.0)),
                            role_label,
                            text: p.full_text(),
                            font_name: p.font_name.clone(),
                            font_size_pt: p.font_size_pt.round().max(8.0) as u16,
                            is_bold: p.is_bold,
                            color: p.color,
                        });
                    }
                    LayoutElement::Table(t) => {
                        let rows = t.num_rows();
                        let cols = t.num_cols();
                        let mut cell_matrix = vec![vec![String::new(); cols]; rows];
                        for c in &t.cells {
                            if c.row_idx < rows && c.col_idx < cols {
                                cell_matrix[c.row_idx][c.col_idx] = c.text();
                            }
                        }
                        nodes.push(ConvertedSlideNodeAst::Table {
                            node_id: {
                                let id = next_node_id;
                                next_node_id += 1;
                                id
                            },
                            x_emu: pt_to_emu(t.bbox.x0),
                            y_emu: pt_to_emu(t.bbox.y0),
                            width_emu: pt_to_emu(t.bbox.width().max(72.0)),
                            height_emu: pt_to_emu(t.bbox.height().max(36.0)),
                            num_rows: rows,
                            num_cols: cols,
                            cell_matrix,
                        });
                    }
                }
            }

            slides.push(ConvertedSlideAst {
                slide_index: page.page_index,
                nodes,
            });
        }

        ConvertedPresentationAst {
            width_emu: pt_to_emu(first_w),
            height_emu: pt_to_emu(first_h),
            slides,
        }
    }
}

// ============================================================================
// 5. BACKWARD-COMPATIBLE PDF CONVERTER (For existing Workspace calls)
// ============================================================================

pub struct PdfConverter;

impl PdfConverter {
    /// Parses a raw text token into a typed spreadsheet CellValue
    pub fn parse_cell_value(text: &str) -> CellValue {
        let trimmed = text.trim();
        let cleaned = trimmed.replace(['$', '£', '€', '¥', ','], "");

        if let Some(pct_str) = trimmed.strip_suffix('%') {
            let clean_pct = pct_str.replace(',', "");
            if let Ok(n) = clean_pct.trim().parse::<f64>() {
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
                                .with_size(block.font_size),
                        );
                        para
                    }
                    BlockKind::Paragraph
                    | BlockKind::TableCell
                    | BlockKind::HeaderBlock
                    | BlockKind::FooterBlock => {
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

                section
                    .elements
                    .push(wps_writer::ast::BlockElement::Paragraph(p));
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
                                wps_writer::ast::TextRun::new(&c.text).with_size(10.0),
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

                section
                    .elements
                    .push(wps_writer::ast::BlockElement::Table(ast_table));
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
                    let mut ws = Worksheet::new(
                        &format!("sheet-{}", idx + 1),
                        &format!("Page {}", idx + 1),
                    );
                    ws.matrix = Self::to_sheets_matrix(std::slice::from_ref(page));
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
                        let font_name = if is_heading {
                            "Helvetica-Bold"
                        } else {
                            "Helvetica"
                        };
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
                        let safe_text = text
                            .replace('\\', "\\\\")
                            .replace('(', "\\(")
                            .replace(')', "\\)");
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
                if let wps_slides::scene_graph::SlideNode::AutoShape(shape) = node {
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
                if matches!(block.kind, BlockKind::Title | BlockKind::Heading(1))
                    && slide_title == format!("Slide {}", idx + 1)
                {
                    slide_title = block.plain_text();
                }

                let shape_id = format!("shape-{}-{}", idx + 1, b_idx + 1);
                let bounds = Rect::new(
                    block.bounds.x,
                    block.bounds.y,
                    block.bounds.width.max(100.0),
                    block.bounds.height.max(30.0),
                );
                let mut shape =
                    ParametricShape::new(&shape_id, ShapeGeometryType::Rectangle, bounds);
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

// ============================================================================
// 6. GATE 5 VERIFICATION UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout_reconstructor::{
        LayoutReconstructorConfig, PageScene, PdfRect, PositionedGlyphRun, SpatialLayoutReconstructor,
        VectorStrokeSegment,
    };

    fn make_glyph(
        id: u64,
        text: &str,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        size: f32,
        bold: bool,
    ) -> PositionedGlyphRun {
        PositionedGlyphRun {
            id,
            text: text.to_string(),
            bbox: PdfRect::new(x0, y0, x1, y1),
            baseline_y: y1 - 2.0,
            font_name: "Inter".to_string(),
            font_size_pt: size,
            is_bold: bold,
            is_italic: false,
            color: RgbaColor {
                r: 40,
                g: 37,
                b: 59,
                a: 255,
            }, // Mercury Ink #28253b
            stream_byte_range: None,
        }
    }

    fn build_sample_reconstructed_page() -> ReconstructedPageLayout {
        let strokes = vec![
            // 2x2 Ruled Table at y = 200..260, x = 100..400 with Row 0 merged across Col 0..1
            VectorStrokeSegment {
                x0: 100.0,
                y0: 200.0,
                x1: 400.0,
                y1: 200.0,
                stroke_width_pt: 1.0,
                color: RgbaColor::default(),
            },
            VectorStrokeSegment {
                x0: 100.0,
                y0: 230.0,
                x1: 400.0,
                y1: 230.0,
                stroke_width_pt: 1.0,
                color: RgbaColor::default(),
            },
            VectorStrokeSegment {
                x0: 100.0,
                y0: 260.0,
                x1: 400.0,
                y1: 260.0,
                stroke_width_pt: 1.0,
                color: RgbaColor::default(),
            },
            VectorStrokeSegment {
                x0: 100.0,
                y0: 200.0,
                x1: 100.0,
                y1: 260.0,
                stroke_width_pt: 1.0,
                color: RgbaColor::default(),
            },
            VectorStrokeSegment {
                x0: 400.0,
                y0: 200.0,
                x1: 400.0,
                y1: 260.0,
                stroke_width_pt: 1.0,
                color: RgbaColor::default(),
            },
            VectorStrokeSegment {
                x0: 250.0,
                y0: 230.0,
                x1: 250.0,
                y1: 260.0,
                stroke_width_pt: 1.0,
                color: RgbaColor::default(),
            },
        ];

        let glyph_runs = vec![
            make_glyph(1, "Annual Financial Report 2026", 80.0, 70.0, 420.0, 96.0, 22.0, true),
            make_glyph(
                2,
                "• Revenue grew by 34.2% across APAC.",
                80.0,
                120.0,
                340.0,
                132.0,
                11.0,
                false,
            ),
            // Table cells
            make_glyph(3, "Regional Breakdown", 180.0, 210.0, 320.0, 222.0, 11.0, true),
            make_glyph(4, "APAC Revenue", 110.0, 240.0, 210.0, 252.0, 10.0, false),
            make_glyph(5, "$1,420,000", 270.0, 240.0, 370.0, 252.0, 10.0, false),
        ];

        let scene = PageScene {
            page_index: 0,
            width_pt: 612.0,
            height_pt: 792.0,
            glyph_runs,
            stroke_segments: strokes,
            filled_rects: vec![],
        };

        let mut rec = SpatialLayoutReconstructor::new(LayoutReconstructorConfig::default());
        rec.reconstruct_page(&scene)
    }

    #[test]
    fn test_convert_pdf_to_writer_ast_twips_and_merged_table() {
        let page = build_sample_reconstructed_page();
        let writer_doc = PdfToOfficeConverter::convert_pdf_to_writer(&[page]);

        assert_eq!(writer_doc.sections.len(), 1);
        let sec = &writer_doc.sections[0];
        assert_eq!(sec.page_width_twips, 12_240); // 612pt * 20 = 12,240 twips
        assert_eq!(sec.page_height_twips, 15_840); // 792pt * 20 = 15,840 twips
        assert_eq!(sec.blocks.len(), 3);

        // Block 0: Title
        match &sec.blocks[0] {
            WriterBlockAst::Paragraph(p) => {
                assert_eq!(p.style_id, "Title");
                assert_eq!(p.runs[0].text, "Annual Financial Report 2026");
            }
            _ => panic!("Expected Title Paragraph"),
        }

        // Block 1: Bulleted List Item
        match &sec.blocks[1] {
            WriterBlockAst::Paragraph(p) => {
                assert_eq!(p.style_id, "List Paragraph");
                assert!(p.list_binding.is_some());
                assert_eq!(p.runs[0].text, "Revenue grew by 34.2% across APAC.");
            }
            _ => panic!("Expected List Paragraph"),
        }

        // Block 2: Table with grid_span = 2 on Row 0
        match &sec.blocks[2] {
            WriterBlockAst::Table(t) => {
                assert_eq!(t.grid_col_widths_twips, vec![3000, 3000]); // 150pt * 20 = 3000 twips each
                assert_eq!(t.rows.len(), 2);
                assert_eq!(t.rows[0].cells.len(), 1);
                assert_eq!(t.rows[0].cells[0].grid_span, 2);
                assert_eq!(t.rows[0].cells[0].width_twips, 6000);
                assert_eq!(
                    t.rows[0].cells[0].paragraphs[0].runs[0].text,
                    "Regional Breakdown"
                );
            }
            _ => panic!("Expected WriterTableAst"),
        }
    }

    #[test]
    fn test_convert_pdf_to_sheets_typed_numbers_and_merges() {
        let page = build_sample_reconstructed_page();
        let wb = PdfToOfficeConverter::convert_pdf_to_sheets(
            &[page],
            PdfToSheetsMode::OneSheetPerTable,
        );

        assert_eq!(wb.sheets.len(), 1);
        let sheet = &wb.sheets[0];
        assert_eq!(sheet.merged_ranges.len(), 1);
        assert_eq!(
            sheet.merged_ranges[0],
            SheetMergedRange {
                start_row: 0,
                start_col: 0,
                end_row: 0,
                end_col: 1
            }
        );

        // Verify "$1,420,000" was parsed into SheetTypedValue::Number(1_420_000.0) with currency format
        let rev_cell = sheet
            .cells
            .iter()
            .find(|c| c.row == 1 && c.col == 1)
            .expect("Expected cell at (1, 1)");
        assert_eq!(rev_cell.value, SheetTypedValue::Number(1_420_000.0));
        assert_eq!(rev_cell.number_format, "$#,##0;($#,##0)");
    }

    #[test]
    fn test_convert_pdf_to_slides_emu_geometry() {
        let page = build_sample_reconstructed_page();
        let pres = PdfToOfficeConverter::convert_pdf_to_slides(&[page]);

        assert_eq!(pres.width_emu, 7_772_400); // 612pt * 12,700 EMU/pt
        assert_eq!(pres.height_emu, 10_058_400); // 792pt * 12,700 EMU/pt
        assert_eq!(pres.slides.len(), 1);
        assert_eq!(pres.slides[0].nodes.len(), 3);
    }
}
