use serde::{Deserialize, Serialize};
use wps_sheets::cell::{CellCoordinate, CellRange, CellValue};
use wps_sheets::sparse_matrix::SparseMatrix;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PySeries {
    pub name: String,
    pub values: Vec<CellValue>,
}

impl PySeries {
    pub fn new(name: &str, values: Vec<CellValue>) -> Self {
        Self {
            name: name.to_string(),
            values,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn numeric_values(&self) -> Vec<f64> {
        self.values.iter().filter_map(|v| match v {
            CellValue::Number(n) => Some(*n),
            CellValue::Text(s) => s.trim().parse::<f64>().ok(),
            _ => None,
        }).collect()
    }

    pub fn mean(&self) -> Option<f64> {
        let nums = self.numeric_values();
        if nums.is_empty() {
            None
        } else {
            Some(nums.iter().sum::<f64>() / nums.len() as f64)
        }
    }

    pub fn std_dev(&self) -> Option<f64> {
        let nums = self.numeric_values();
        if nums.len() < 2 {
            return None;
        }
        let mean = self.mean()?;
        let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn min(&self) -> Option<f64> {
        self.numeric_values().into_iter().reduce(f64::min)
    }

    pub fn max(&self) -> Option<f64> {
        self.numeric_values().into_iter().reduce(f64::max)
    }

    pub fn percentile(&self, p: f64) -> Option<f64> {
        let mut nums = self.numeric_values();
        if nums.is_empty() {
            return None;
        }
        nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((nums.len() - 1) as f64 * p).round() as usize;
        Some(nums[idx.min(nums.len() - 1)])
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PyDataFrame {
    pub columns: Vec<PySeries>,
}

impl PyDataFrame {
    pub fn new(columns: Vec<PySeries>) -> Self {
        Self { columns }
    }

    pub fn from_sheet_range(matrix: &SparseMatrix, range: CellRange) -> Self {
        let mut columns = Vec::new();

        for col_idx in range.start.col..=range.end.col {
            // First row in range is treated as header if it's text
            let header_coord = CellCoordinate::new(range.start.row, col_idx);
            let col_name = matrix.get_cell(header_coord)
                .map(|c| c.value.as_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("Col_{}", col_idx + 1));

            let mut values = Vec::new();
            for row_idx in (range.start.row + 1)..=range.end.row {
                let coord = CellCoordinate::new(row_idx, col_idx);
                let val = matrix.get_cell(coord)
                    .map(|c| c.value.clone())
                    .unwrap_or(CellValue::Empty);
                values.push(val);
            }

            columns.push(PySeries::new(&col_name, values));
        }

        Self { columns }
    }

    pub fn num_rows(&self) -> usize {
        self.columns.first().map(|c| c.len()).unwrap_or(0)
    }

    pub fn num_cols(&self) -> usize {
        self.columns.len()
    }

    pub fn get_column(&self, name: &str) -> Option<&PySeries> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Computes summary statistics matching pandas DataFrame.describe()
    pub fn describe(&self) -> Self {
        let metrics = ["count", "mean", "std", "min", "25%", "50%", "75%", "max"];
        let metric_series = PySeries::new("metric", metrics.iter().map(|m| CellValue::Text(m.to_string())).collect());

        let mut desc_columns = vec![metric_series];

        for col in &self.columns {
            let nums = col.numeric_values();
            let count = nums.len() as f64;
            let mean = col.mean().unwrap_or(0.0);
            let std = col.std_dev().unwrap_or(0.0);
            let min = col.min().unwrap_or(0.0);
            let p25 = col.percentile(0.25).unwrap_or(0.0);
            let p50 = col.percentile(0.50).unwrap_or(0.0);
            let p75 = col.percentile(0.75).unwrap_or(0.0);
            let max = col.max().unwrap_or(0.0);

            let vals = vec![
                CellValue::Number(count),
                CellValue::Number(mean),
                CellValue::Number(std),
                CellValue::Number(min),
                CellValue::Number(p25),
                CellValue::Number(p50),
                CellValue::Number(p75),
                CellValue::Number(max),
            ];

            desc_columns.push(PySeries::new(&col.name, vals));
        }

        Self { columns: desc_columns }
    }

    /// Converts DataFrame to a 2D array matrix of CellValues for spilling into the spreadsheet grid
    pub fn to_2d_array(&self) -> Vec<Vec<CellValue>> {
        let mut rows = Vec::new();

        // 1. Header row
        let header = self.columns.iter().map(|c| CellValue::Text(c.name.clone())).collect();
        rows.push(header);

        // 2. Data rows
        let n_rows = self.num_rows();
        for r in 0..n_rows {
            let mut row = Vec::new();
            for col in &self.columns {
                let val = col.values.get(r).cloned().unwrap_or(CellValue::Empty);
                row.push(val);
            }
            rows.push(row);
        }

        rows
    }
}
