use crate::cell::*;
use std::collections::{BTreeMap, HashMap};

pub const CHUNK_SIZE: usize = 64;

#[derive(Debug, Clone)]
pub struct SparseChunk {
    cells: HashMap<(u8, u8), Cell>,
}

impl Default for SparseChunk {
    fn default() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }
}

impl SparseChunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, local_row: u8, local_col: u8) -> Option<&Cell> {
        self.cells.get(&(local_row, local_col))
    }

    pub fn get_mut(&mut self, local_row: u8, local_col: u8) -> Option<&mut Cell> {
        self.cells.get_mut(&(local_row, local_col))
    }

    pub fn set(&mut self, local_row: u8, local_col: u8, cell: Cell) {
        if cell.value.is_empty() && cell.formula.is_none() && cell.comment.is_none() && cell.format_index == 0 {
            self.cells.remove(&(local_row, local_col));
        } else {
            self.cells.insert((local_row, local_col), cell);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SparseMatrix {
    chunks: BTreeMap<(u32, u16), Box<SparseChunk>>,
    pub row_heights: HashMap<u32, f64>,
    pub col_widths: HashMap<u16, f64>,
    pub merged_ranges: Vec<CellRange>,
    pub frozen_rows: u32,
    pub frozen_cols: u16,
}

impl SparseMatrix {
    pub fn new() -> Self {
        Self::default()
    }

    fn chunk_coord(coord: CellCoordinate) -> ((u32, u16), (u8, u8)) {
        let chunk_row = coord.row / CHUNK_SIZE as u32;
        let chunk_col = coord.col / CHUNK_SIZE as u16;
        let local_row = (coord.row % CHUNK_SIZE as u32) as u8;
        let local_col = (coord.col % CHUNK_SIZE as u16) as u8;
        ((chunk_row, chunk_col), (local_row, local_col))
    }

    pub fn get_cell(&self, coord: CellCoordinate) -> Option<&Cell> {
        let (chunk_idx, local) = Self::chunk_coord(coord);
        self.chunks.get(&chunk_idx).and_then(|c| c.get(local.0, local.1))
    }

    pub fn get_cell_mut(&mut self, coord: CellCoordinate) -> Option<&mut Cell> {
        let (chunk_idx, local) = Self::chunk_coord(coord);
        self.chunks.get_mut(&chunk_idx).and_then(|c| c.get_mut(local.0, local.1))
    }

    pub fn set_cell(&mut self, coord: CellCoordinate, cell: Cell) {
        let (chunk_idx, local) = Self::chunk_coord(coord);
        let chunk = self.chunks.entry(chunk_idx).or_insert_with(|| Box::new(SparseChunk::new()));
        chunk.set(local.0, local.1, cell);
        if chunk.is_empty() {
            self.chunks.remove(&chunk_idx);
        }
    }

    pub fn set_value(&mut self, coord: CellCoordinate, value: CellValue) {
        let mut cell = self.get_cell(coord).cloned().unwrap_or_default();
        cell.value = value;
        self.set_cell(coord, cell);
    }

    pub fn set_formula(&mut self, coord: CellCoordinate, formula: Option<String>) {
        let mut cell = self.get_cell(coord).cloned().unwrap_or_default();
        cell.formula = formula;
        self.set_cell(coord, cell);
    }

    pub fn non_empty_cell_count(&self) -> usize {
        self.chunks.values().map(|c| c.cells.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_matrix_at_scale() {
        let mut matrix = SparseMatrix::new();
        let c1 = CellCoordinate::new(0, 0); // A1
        let c2 = CellCoordinate::new(1_000_000, 16_000); // Far corner

        matrix.set_value(c1, CellValue::Number(42.0));
        matrix.set_value(c2, CellValue::Text("Far Cell".into()));

        assert_eq!(matrix.get_cell(c1).unwrap().value, CellValue::Number(42.0));
        assert_eq!(matrix.get_cell(c2).unwrap().value, CellValue::Text("Far Cell".into()));
        assert_eq!(matrix.non_empty_cell_count(), 2);
    }
}
