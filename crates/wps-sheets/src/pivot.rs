use crate::cell::{CellCoordinate, CellValue};
use crate::sparse_matrix::SparseMatrix;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PivotAggregation {
    Sum,
    Count,
    Average,
    Min,
    Max,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotConfig {
    pub row_field_col: u16,
    pub val_field_col: u16,
    pub aggregation: PivotAggregation,
}

pub struct PivotCube;

impl PivotCube {
    pub fn build_pivot(
        matrix: &SparseMatrix,
        max_rows: u32,
        config: &PivotConfig,
    ) -> Vec<Vec<CellValue>> {
        let mut grouped: BTreeMap<String, Vec<f64>> = BTreeMap::new();

        for r in 1..=max_rows {
            let row_key = matrix
                .get_cell(CellCoordinate::new(r, config.row_field_col))
                .map(|c| c.value.as_string())
                .unwrap_or_default();

            if let Some(val) = matrix
                .get_cell(CellCoordinate::new(r, config.val_field_col))
                .and_then(|c| c.value.as_number())
            {
                grouped.entry(row_key).or_default().push(val);
            }
        }

        let mut output = Vec::new();
        // Header
        output.push(vec![
            CellValue::Text("Row Labels".into()),
            CellValue::Text(format!("{:?} of Value", config.aggregation)),
        ]);

        let mut grand_total_vec = Vec::new();

        for (key, values) in grouped {
            grand_total_vec.extend(values.clone());
            let agg_val = match config.aggregation {
                PivotAggregation::Sum => values.iter().sum::<f64>(),
                PivotAggregation::Count => values.len() as f64,
                PivotAggregation::Average => {
                    if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 }
                }
                PivotAggregation::Min => values.into_iter().fold(f64::INFINITY, f64::min),
                PivotAggregation::Max => values.into_iter().fold(f64::NEG_INFINITY, f64::max),
            };
            output.push(vec![CellValue::Text(key), CellValue::Number(agg_val)]);
        }

        // Grand Total Row
        let grand_total = match config.aggregation {
            PivotAggregation::Sum => grand_total_vec.iter().sum::<f64>(),
            PivotAggregation::Count => grand_total_vec.len() as f64,
            PivotAggregation::Average => {
                if grand_total_vec.is_empty() { 0.0 } else { grand_total_vec.iter().sum::<f64>() / grand_total_vec.len() as f64 }
            }
            _ => 0.0,
        };
        output.push(vec![CellValue::Text("Grand Total".into()), CellValue::Number(grand_total)]);

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pivot_cube() {
        let mut matrix = SparseMatrix::new();
        // Header
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Category".into()));
        matrix.set_value(CellCoordinate::new(0, 1), CellValue::Text("Sales".into()));

        // Data
        matrix.set_value(CellCoordinate::new(1, 0), CellValue::Text("Hardware".into()));
        matrix.set_value(CellCoordinate::new(1, 1), CellValue::Number(100.0));
        matrix.set_value(CellCoordinate::new(2, 0), CellValue::Text("Hardware".into()));
        matrix.set_value(CellCoordinate::new(2, 1), CellValue::Number(150.0));
        matrix.set_value(CellCoordinate::new(3, 0), CellValue::Text("Software".into()));
        matrix.set_value(CellCoordinate::new(3, 1), CellValue::Number(300.0));

        let config = PivotConfig {
            row_field_col: 0,
            val_field_col: 1,
            aggregation: PivotAggregation::Sum,
        };

        let pivot = PivotCube::build_pivot(&matrix, 3, &config);
        assert_eq!(pivot.len(), 4); // Header + Hardware + Software + Grand Total
        assert_eq!(pivot[1][1], CellValue::Number(250.0)); // Hardware total
        assert_eq!(pivot[2][1], CellValue::Number(300.0)); // Software total
        assert_eq!(pivot[3][1], CellValue::Number(550.0)); // Grand total
    }
}
