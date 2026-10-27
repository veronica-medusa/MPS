use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_ROWS: u32 = 1_048_576;
pub const MAX_COLS: u16 = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellCoordinate {
    pub row: u32, // 0-indexed (0 to 1,048,575)
    pub col: u16, // 0-indexed (0 to 16,383)
}

impl CellCoordinate {
    pub const fn new(row: u32, col: u16) -> Self {
        Self { row, col }
    }

    /// Parses standard Excel coordinate string like "A1", "$B$10", "XFD1048576"
    pub fn parse(s: &str) -> Option<Self> {
        let clean = s.trim().replace('$', "");
        let col_letters: String = clean.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
        let row_digits: String = clean.chars().skip(col_letters.len()).collect();

        if col_letters.is_empty() || row_digits.is_empty() {
            return None;
        }

        let mut col: u16 = 0;
        for ch in col_letters.to_ascii_uppercase().chars() {
            col = col.checked_mul(26)?.checked_add((ch as u16) - b'A' as u16 + 1)?;
        }
        col = col.checked_sub(1)?;

        let row = row_digits.parse::<u32>().ok()?.checked_sub(1)?;

        if row < MAX_ROWS && col < MAX_COLS {
            Some(Self { row, col })
        } else {
            None
        }
    }

    pub fn to_a1_string(&self) -> String {
        let mut col_str = String::new();
        let mut temp_col = self.col as u32 + 1;
        while temp_col > 0 {
            let rem = ((temp_col - 1) % 26) as u8;
            col_str.insert(0, (b'A' + rem) as char);
            temp_col = (temp_col - 1) / 26;
        }
        format!("{}{}", col_str, self.row + 1)
    }
}

impl fmt::Display for CellCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_a1_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellRange {
    pub start: CellCoordinate,
    pub end: CellCoordinate,
}

impl CellRange {
    pub fn new(start: CellCoordinate, end: CellCoordinate) -> Self {
        let min_row = start.row.min(end.row);
        let max_row = start.row.max(end.row);
        let min_col = start.col.min(end.col);
        let max_col = start.col.max(end.col);
        Self {
            start: CellCoordinate::new(min_row, min_col),
            end: CellCoordinate::new(max_row, max_col),
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 1 {
            let coord = CellCoordinate::parse(parts[0])?;
            Some(Self::new(coord, coord))
        } else if parts.len() == 2 {
            let start = CellCoordinate::parse(parts[0])?;
            let end = CellCoordinate::parse(parts[1])?;
            Some(Self::new(start, end))
        } else {
            None
        }
    }

    pub fn contains(&self, coord: CellCoordinate) -> bool {
        coord.row >= self.start.row
            && coord.row <= self.end.row
            && coord.col >= self.start.col
            && coord.col <= self.end.col
    }

    pub fn iter_coords(&self) -> impl Iterator<Item = CellCoordinate> {
        let start_row = self.start.row;
        let end_row = self.end.row;
        let start_col = self.start.col;
        let end_col = self.end.col;

        (start_row..=end_row).flat_map(move |r| (start_col..=end_col).map(move |c| CellCoordinate::new(r, c)))
    }

    pub fn row_count(&self) -> u32 {
        self.end.row - self.start.row + 1
    }

    pub fn col_count(&self) -> u16 {
        self.end.col - self.start.col + 1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellError {
    Null,       // #NULL!
    DivZero,    // #DIV/0!
    Value,      // #VALUE!
    Ref,        // #REF!
    Name,       // #NAME?
    Num,        // #NUM!
    NA,         // #N/A
    Spill,      // #SPILL!
    Circular,   // #CIRCULAR!
}

impl CellError {
    pub fn as_str(&self) -> &'static str {
        match self {
            CellError::Null => "#NULL!",
            CellError::DivZero => "#DIV/0!",
            CellError::Value => "#VALUE!",
            CellError::Ref => "#REF!",
            CellError::Name => "#NAME?",
            CellError::Num => "#NUM!",
            CellError::NA => "#N/A",
            CellError::Spill => "#SPILL!",
            CellError::Circular => "#CIRCULAR!",
        }
    }
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum CellValue {
    #[default]
    Empty,
    Number(f64),
    Text(String),
    Boolean(bool),
    Error(CellError),
    Array(Vec<Vec<CellValue>>),
}

impl CellValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            CellValue::Text(t) => t.trim().parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    format!("{}", n)
                }
            }
            CellValue::Text(t) => t.clone(),
            CellValue::Boolean(b) => if *b { "TRUE".into() } else { "FALSE".into() },
            CellValue::Error(e) => e.as_str().into(),
            CellValue::Array(arr) => {
                if let Some(first_row) = arr.first() {
                    if let Some(first_val) = first_row.first() {
                        return first_val.as_string();
                    }
                }
                "#VALUE!".into()
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, CellValue::Error(_))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Cell {
    pub formula: Option<String>,
    pub value: CellValue,
    pub format_index: usize,
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_coordinates() {
        let coord = CellCoordinate::parse("A1").unwrap();
        assert_eq!(coord.row, 0);
        assert_eq!(coord.col, 0);
        assert_eq!(coord.to_a1_string(), "A1");

        let b2 = CellCoordinate::parse("$B$2").unwrap();
        assert_eq!(b2.row, 1);
        assert_eq!(b2.col, 1);
        assert_eq!(b2.to_a1_string(), "B2");

        let xfd = CellCoordinate::parse("XFD1048576").unwrap();
        assert_eq!(xfd.row, 1_048_575);
        assert_eq!(xfd.col, 16_383);
        assert_eq!(xfd.to_a1_string(), "XFD1048576");
    }

    #[test]
    fn test_cell_range() {
        let range = CellRange::parse("A1:C3").unwrap();
        assert_eq!(range.row_count(), 3);
        assert_eq!(range.col_count(), 3);
        assert!(range.contains(CellCoordinate::new(1, 1))); // B2
        assert!(!range.contains(CellCoordinate::new(3, 3))); // D4
    }
}
