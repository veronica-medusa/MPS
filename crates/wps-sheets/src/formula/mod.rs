pub mod evaluator;
pub mod functions;
pub mod parser;
pub mod token;

pub use evaluator::Evaluator;
pub use functions::call_function;
pub use parser::{BinaryOp, FormulaExpr, Parser, UnaryOp};
pub use token::{Lexer, Token};

use crate::cell::{CellCoordinate, MAX_COLS, MAX_ROWS};

/// Shifts cell references in a formula string by delta_rows and delta_cols,
/// respecting absolute reference markers ($).
pub fn shift_formula_references(formula: &str, delta_rows: i32, delta_cols: i32) -> String {
    let mut result = String::new();
    let chars: Vec<char> = formula.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Skip string literals "..."
        if chars[i] == '"' {
            result.push('"');
            i += 1;
            while i < len {
                result.push(chars[i]);
                if chars[i] == '"' {
                    i += 1;
                    if i < len && chars[i] == '"' {
                        result.push('"');
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Check if previous character was part of an identifier
        let prev_is_ident = i > 0 && (chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '.');

        if !prev_is_ident && (chars[i] == '$' || chars[i].is_ascii_alphabetic()) {
            let start = i;
            let mut curr = i;
            let mut col_abs = false;

            if chars[curr] == '$' {
                col_abs = true;
                curr += 1;
            }

            let col_start = curr;
            while curr < len && chars[curr].is_ascii_alphabetic() {
                curr += 1;
            }
            let col_letters: String = chars[col_start..curr].iter().collect();

            if !col_letters.is_empty() && curr < len {
                let mut row_abs = false;
                if chars[curr] == '$' {
                    row_abs = true;
                    curr += 1;
                }

                let row_start = curr;
                while curr < len && chars[curr].is_ascii_digit() {
                    curr += 1;
                }
                let row_digits: String = chars[row_start..curr].iter().collect();

                // Ensure next char is not alphanumeric (which would mean a longer identifier like LOG10)
                let next_is_ident = curr < len && (chars[curr].is_ascii_alphanumeric() || chars[curr] == '_' || chars[curr] == '(');

                if !row_digits.is_empty() && !next_is_ident {
                    // Valid cell reference found
                    let ref_str = format!("{}{}", col_letters, row_digits);
                    if let Some(coord) = CellCoordinate::parse(&ref_str) {
                        let new_col = if col_abs {
                            coord.col
                        } else {
                            (coord.col as i32 + delta_cols).clamp(0, MAX_COLS as i32 - 1) as u16
                        };

                        let new_row = if row_abs {
                            coord.row
                        } else {
                            (coord.row as i32 + delta_rows).clamp(0, MAX_ROWS as i32 - 1) as u32
                        };

                        let shifted_coord = CellCoordinate::new(new_row, new_col);
                        let a1 = shifted_coord.to_a1_string();
                        // Extract col letters and row digits from a1
                        let letters: String = a1.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
                        let digits: String = a1.chars().skip(letters.len()).collect();

                        if col_abs {
                            result.push('$');
                        }
                        result.push_str(&letters);
                        if row_abs {
                            result.push('$');
                        }
                        result.push_str(&digits);

                        i = curr;
                        continue;
                    }
                }
            }

            // Fallback: not a valid coordinate, push original character
            result.push(chars[start]);
            i = start + 1;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Extracts cell references from embedded Python scripts (e.g. wp.range('A1:B4') or wp.cell('C5'))
pub fn extract_python_script_dependencies(script: &str, out: &mut Vec<CellCoordinate>) {
    if let Ok(range_re) = regex::Regex::new(r#"wp\.range\s*\(\s*['"]([A-Za-z0-9:]+)['"]\s*\)"#) {
        for cap in range_re.captures_iter(script) {
            if let Some(m) = cap.get(1) {
                if let Some(rng) = crate::cell::CellRange::parse(m.as_str()) {
                    for r in rng.start.row..=rng.end.row {
                        for c in rng.start.col..=rng.end.col {
                            out.push(CellCoordinate::new(r, c));
                        }
                    }
                }
            }
        }
    }

    if let Ok(cell_re) = regex::Regex::new(r#"wp\.cell\s*\(\s*['"]([A-Za-z0-9]+)['"]\s*\)"#) {
        for cap in cell_re.captures_iter(script) {
            if let Some(m) = cap.get(1) {
                if let Some(coord) = CellCoordinate::parse(m.as_str()) {
                    out.push(coord);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_formula_references() {
        let formula = "=A1 + $B$2 + C$3 + $D4 + SUM(E5:F10) + \"A1 literal\"";
        // Shift by +1 row, +2 columns
        let shifted = shift_formula_references(formula, 1, 2);
        assert_eq!(
            shifted,
            "=C2 + $B$2 + E$3 + $D5 + SUM(G6:H11) + \"A1 literal\""
        );
    }
}
