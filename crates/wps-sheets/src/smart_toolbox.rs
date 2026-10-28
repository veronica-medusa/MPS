use crate::cell::{CellCoordinate, CellRange, CellValue};
use crate::sparse_matrix::SparseMatrix;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum CellDiffStatus {
    Added,
    Deleted,
    ValueChanged { old_val: CellValue, new_val: CellValue },
    FormulaChanged { old_formula: String, new_formula: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SheetDiff {
    pub changed_cells: HashMap<CellCoordinate, CellDiffStatus>,
}

#[derive(Debug, Clone)]
pub struct HygieneOptions {
    pub trim_spaces: bool,
    pub remove_duplicates: bool,
    pub strip_non_printable: bool,
    pub key_columns: Vec<u16>,
}

pub struct SmartToolbox;

impl SmartToolbox {
    /// 1. Merge Without Data Loss:
    /// Combines all non-empty cell values in range into top-left cell separated by delimiter,
    /// then clears the remaining cells in the range.
    pub fn merge_without_data_loss(
        matrix: &mut SparseMatrix,
        range: CellRange,
        delimiter: &str,
    ) {
        let mut values = Vec::new();

        for coord in range.iter_coords() {
            if let Some(cell) = matrix.get_cell(coord) {
                let text = cell.value.as_string();
                if !text.trim().is_empty() {
                    values.push(text);
                }
            }
        }

        let merged_text = values.join(delimiter);

        // Clear all cells in range
        for coord in range.iter_coords() {
            matrix.set_value(coord, CellValue::Empty);
            matrix.set_formula(coord, None);
        }

        // Set merged text in top-left cell
        matrix.set_value(range.start, CellValue::Text(merged_text));
        matrix.merged_ranges.push(range);
    }

    /// 2. Split Sheet Wizard:
    /// Decomposes master sheet rows into separate named sub-matrices grouped by unique value in key_col.
    pub fn split_sheet_by_column(
        matrix: &SparseMatrix,
        max_rows: u32,
        max_cols: u16,
        key_col: u16,
    ) -> HashMap<String, SparseMatrix> {
        let mut groups: HashMap<String, Vec<u32>> = HashMap::new();

        // Row 0 is assumed header
        for r in 1..=max_rows {
            let key = matrix
                .get_cell(CellCoordinate::new(r, key_col))
                .map(|c| c.value.as_string())
                .unwrap_or_default();
            if !key.is_empty() {
                groups.entry(key).or_default().push(r);
            }
        }

        let mut result = HashMap::new();
        for (group_name, rows) in groups {
            let mut sub_matrix = SparseMatrix::new();

            // Copy header row (row 0)
            for c in 0..=max_cols {
                let coord = CellCoordinate::new(0, c);
                if let Some(cell) = matrix.get_cell(coord) {
                    sub_matrix.set_cell(coord, cell.clone());
                }
            }

            // Copy grouped rows
            for (new_r, &orig_r) in rows.iter().enumerate() {
                let dest_row = (new_r + 1) as u32;
                for c in 0..=max_cols {
                    let src_coord = CellCoordinate::new(orig_r, c);
                    if let Some(cell) = matrix.get_cell(src_coord) {
                        sub_matrix.set_cell(CellCoordinate::new(dest_row, c), cell.clone());
                    }
                }
            }

            result.insert(group_name, sub_matrix);
        }

        result
    }

    /// 3. Combine Sheets:
    /// Combines multiple matrices into a single master sheet with source tracking.
    pub fn combine_sheets(
        sheets: &[(&str, &SparseMatrix, u32, u16)],
    ) -> SparseMatrix {
        let mut master = SparseMatrix::new();
        let mut current_dest_row: u32 = 0;

        // Copy header from first sheet
        if let Some((_, first_matrix, _, max_cols)) = sheets.first() {
            for c in 0..=*max_cols {
                let coord = CellCoordinate::new(0, c);
                if let Some(cell) = first_matrix.get_cell(coord) {
                    master.set_cell(coord, cell.clone());
                }
            }
            // Add source sheet header column
            master.set_value(CellCoordinate::new(0, *max_cols + 1), CellValue::Text("Source Sheet".into()));
            current_dest_row = 1;
        }

        for (sheet_name, matrix, max_rows, max_cols) in sheets {
            for r in 1..=*max_rows {
                let mut has_data = false;
                for c in 0..=*max_cols {
                    let coord = CellCoordinate::new(r, c);
                    if let Some(cell) = matrix.get_cell(coord) {
                        if !cell.value.is_empty() {
                            has_data = true;
                            master.set_cell(CellCoordinate::new(current_dest_row, c), cell.clone());
                        }
                    }
                }
                if has_data {
                    master.set_value(CellCoordinate::new(current_dest_row, *max_cols + 1), CellValue::Text(sheet_name.to_string()));
                    current_dest_row += 1;
                }
            }
        }

        master
    }

    /// 4. Sheet Comparator:
    /// Visual diff between two matrices highlighting added, deleted, value-changed, or formula-changed cells.
    pub fn compare_sheets(
        sheet_a: &SparseMatrix,
        sheet_b: &SparseMatrix,
        max_rows: u32,
        max_cols: u16,
    ) -> SheetDiff {
        let mut changed_cells = HashMap::new();

        for r in 0..=max_rows {
            for c in 0..=max_cols {
                let coord = CellCoordinate::new(r, c);
                let cell_a = sheet_a.get_cell(coord);
                let cell_b = sheet_b.get_cell(coord);

                match (cell_a, cell_b) {
                    (None, Some(b)) if !b.value.is_empty() => {
                        changed_cells.insert(coord, CellDiffStatus::Added);
                    }
                    (Some(a), None) if !a.value.is_empty() => {
                        changed_cells.insert(coord, CellDiffStatus::Deleted);
                    }
                    (Some(a), Some(b)) => {
                        if a.formula != b.formula {
                            changed_cells.insert(
                                coord,
                                CellDiffStatus::FormulaChanged {
                                    old_formula: a.formula.clone().unwrap_or_default(),
                                    new_formula: b.formula.clone().unwrap_or_default(),
                                },
                            );
                        } else if a.value != b.value {
                            changed_cells.insert(
                                coord,
                                CellDiffStatus::ValueChanged {
                                    old_val: a.value.clone(),
                                    new_val: b.value.clone(),
                                },
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        SheetDiff { changed_cells }
    }

    /// 5. One-Click Data Hygiene:
    /// Trims spaces, strips non-printable ASCII, and deduplicates rows across key columns.
    pub fn data_hygiene(
        matrix: &mut SparseMatrix,
        max_rows: u32,
        max_cols: u16,
        options: &HygieneOptions,
    ) {
        // 1. Trim spaces and clean non-printable
        for r in 0..=max_rows {
            for c in 0..=max_cols {
                let coord = CellCoordinate::new(r, c);
                if let Some(cell) = matrix.get_cell(coord) {
                    if let CellValue::Text(text) = &cell.value {
                        let mut cleaned = text.clone();
                        if options.strip_non_printable {
                            cleaned = cleaned.chars().filter(|ch| !ch.is_control()).collect();
                        }
                        if options.trim_spaces {
                            cleaned = cleaned.trim().to_string();
                        }
                        if cleaned != *text {
                            matrix.set_value(coord, CellValue::Text(cleaned));
                        }
                    }
                }
            }
        }

        // 2. Remove duplicate rows across key columns
        if options.remove_duplicates && !options.key_columns.is_empty() {
            let mut seen_keys = HashSet::new();
            for r in 1..=max_rows {
                let mut composite_key = String::new();
                for &k_col in &options.key_columns {
                    let val = matrix
                        .get_cell(CellCoordinate::new(r, k_col))
                        .map(|c| c.value.as_string())
                        .unwrap_or_default();
                    composite_key.push_str(&val);
                    composite_key.push('\t');
                }

                if !composite_key.trim().is_empty() {
                    if !seen_keys.insert(composite_key) {
                        // Duplicate row! Clear row cells
                        for c in 0..=max_cols {
                            matrix.set_value(CellCoordinate::new(r, c), CellValue::Empty);
                        }
                    }
                }
            }
        }
    }

    /// 6. Unmerge and Fill:
    /// Unmerges the specified range and populates all former child cells with the parent cell value.
    pub fn unmerge_and_fill(matrix: &mut SparseMatrix, range: CellRange) {
        let parent_val = matrix
            .get_cell(range.start)
            .map(|c| c.value.clone())
            .unwrap_or(CellValue::Empty);

        matrix.merged_ranges.retain(|&r| r != range);

        for coord in range.iter_coords() {
            matrix.set_value(coord, parent_val.clone());
        }
    }

    /// 7. In-Place Range Arithmetic:
    /// Applies arithmetic operation (add, sub, mul, div, round) across all numbers in range.
    pub fn range_arithmetic(
        matrix: &mut SparseMatrix,
        range: CellRange,
        op: &str,
        operand: f64,
    ) {
        for coord in range.iter_coords() {
            if let Some(cell) = matrix.get_cell(coord) {
                if let CellValue::Number(n) = cell.value {
                    let new_val = match op.to_lowercase().as_str() {
                        "add" | "+" => n + operand,
                        "sub" | "-" => n - operand,
                        "mul" | "*" => n * operand,
                        "div" | "/" => if operand != 0.0 { n / operand } else { n },
                        "round" => {
                            let factor = 10f64.powi(operand as i32);
                            (n * factor).round() / factor
                        }
                        _ => n,
                    };
                    matrix.set_value(coord, CellValue::Number(new_val));
                }
            }
        }
    }

    /// 8. Batch Regex Extractor:
    /// Extracts regex pattern matches from source column into target column.
    pub fn batch_regex_extract(
        matrix: &mut SparseMatrix,
        range: CellRange,
        target_col: u16,
        pattern: &str,
    ) -> Result<usize, String> {
        let re = regex::Regex::new(pattern).map_err(|e| e.to_string())?;
        let mut count = 0;

        for r in range.start.row..=range.end.row {
            let src_coord = CellCoordinate::new(r, range.start.col);
            if let Some(cell) = matrix.get_cell(src_coord) {
                let text = cell.value.as_string();
                if let Some(mat) = re.find(&text) {
                    let dest_coord = CellCoordinate::new(r, target_col);
                    matrix.set_value(dest_coord, CellValue::Text(mat.as_str().to_string()));
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// 9. Batch Add Prefix & Suffix
    pub fn batch_add_prefix_suffix(
        matrix: &mut SparseMatrix,
        range: CellRange,
        prefix: &str,
        suffix: &str,
    ) {
        for coord in range.iter_coords() {
            if let Some(cell) = matrix.get_cell(coord) {
                if !cell.value.is_empty() {
                    let text = cell.value.as_string();
                    let updated = format!("{}{}{}", prefix, text, suffix);
                    matrix.set_value(coord, CellValue::Text(updated));
                }
            }
        }
    }

    /// 10. Batch Case Conversion
    pub fn batch_case_conversion(
        matrix: &mut SparseMatrix,
        range: CellRange,
        case_mode: &str,
    ) {
        for coord in range.iter_coords() {
            if let Some(cell) = matrix.get_cell(coord) {
                if let CellValue::Text(text) = &cell.value {
                    let converted = match case_mode.to_lowercase().as_str() {
                        "upper" => text.to_uppercase(),
                        "lower" => text.to_lowercase(),
                        "proper" => {
                            let mut c_chars: Vec<char> = Vec::new();
                            let mut cap_next = true;
                            for ch in text.chars() {
                                if ch.is_whitespace() || ch == '_' || ch == '-' {
                                    c_chars.push(ch);
                                    cap_next = true;
                                } else if cap_next {
                                    c_chars.push(ch.to_ascii_uppercase());
                                    cap_next = false;
                                } else {
                                    c_chars.push(ch.to_ascii_lowercase());
                                }
                            }
                            c_chars.into_iter().collect()
                        }
                        _ => text.clone(),
                    };
                    matrix.set_value(coord, CellValue::Text(converted));
                }
            }
        }
    }

    /// 11. Convert Formulas to Static Values
    pub fn convert_formulas_to_values(matrix: &mut SparseMatrix, range: CellRange) {
        for coord in range.iter_coords() {
            if let Some(cell) = matrix.get_cell(coord) {
                if cell.formula.is_some() {
                    let val = cell.value.clone();
                    matrix.set_cell(
                        coord,
                        crate::cell::Cell {
                            formula: None,
                            value: val,
                            format_index: cell.format_index,
                            comment: cell.comment.clone(),
                            hyperlink: cell.hyperlink.clone(),
                            spill_origin: None,
                        },
                    );
                }
            }
        }
    }

    /// 12. Smart Autofill Series Extrapolation
    pub fn autofill_series(
        matrix: &mut SparseMatrix,
        source: CellRange,
        target: CellRange,
    ) {
        // Collect numeric series from source column
        let mut source_nums = Vec::new();
        for r in source.start.row..=source.end.row {
            let coord = CellCoordinate::new(r, source.start.col);
            if let Some(cell) = matrix.get_cell(coord) {
                if let Some(num) = cell.value.as_number() {
                    source_nums.push(num);
                }
            }
        }

        if source_nums.len() >= 2 {
            // Compute linear step
            let step = (source_nums.last().unwrap() - source_nums.first().unwrap()) / (source_nums.len() - 1) as f64;
            let mut curr = *source_nums.last().unwrap();

            for r in target.start.row..=target.end.row {
                curr += step;
                matrix.set_value(CellCoordinate::new(r, target.start.col), CellValue::Number(curr));
            }
        } else if let Some(&single) = source_nums.first() {
            // Replicate single number with step 1
            let mut curr = single;
            for r in target.start.row..=target.end.row {
                curr += 1.0;
                matrix.set_value(CellCoordinate::new(r, target.start.col), CellValue::Number(curr));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_without_data_loss() {
        let mut matrix = SparseMatrix::new();
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("First".into()));
        matrix.set_value(CellCoordinate::new(0, 1), CellValue::Text("Middle".into()));
        matrix.set_value(CellCoordinate::new(0, 2), CellValue::Text("Last".into()));

        let range = CellRange::parse("A1:C1").unwrap();
        SmartToolbox::merge_without_data_loss(&mut matrix, range, " ");

        assert_eq!(
            matrix.get_cell(CellCoordinate::new(0, 0)).unwrap().value,
            CellValue::Text("First Middle Last".into())
        );
        let val_b1 = matrix.get_cell(CellCoordinate::new(0, 1)).map(|c| c.value.clone()).unwrap_or(CellValue::Empty);
        let val_c1 = matrix.get_cell(CellCoordinate::new(0, 2)).map(|c| c.value.clone()).unwrap_or(CellValue::Empty);
        assert_eq!(val_b1, CellValue::Empty);
        assert_eq!(val_c1, CellValue::Empty);
    }

    #[test]
    fn test_sheet_comparator() {
        let mut a = SparseMatrix::new();
        let mut b = SparseMatrix::new();

        let c1 = CellCoordinate::new(1, 1);
        a.set_value(c1, CellValue::Number(100.0));
        b.set_value(c1, CellValue::Number(150.0));

        let diff = SmartToolbox::compare_sheets(&a, &b, 5, 5);
        assert_eq!(diff.changed_cells.len(), 1);
        assert_eq!(
            diff.changed_cells.get(&c1),
            Some(&CellDiffStatus::ValueChanged {
                old_val: CellValue::Number(100.0),
                new_val: CellValue::Number(150.0)
            })
        );
    }
}
