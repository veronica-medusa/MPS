use crate::cell::{CellCoordinate, CellValue};
use crate::sparse_matrix::SparseMatrix;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PivotAggregation {
    #[default]
    Sum,
    Count,
    Average,
    Min,
    Max,
    Product,
    StdDev,
    Var,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ShowValuesAs {
    #[default]
    Normal,
    PercentOfGrandTotal,
    PercentOfRowTotal,
    PercentOfColumnTotal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotValueDef {
    pub col: u16,
    pub name: String,
    pub aggregation: PivotAggregation,
    pub show_as: ShowValuesAs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PivotConfig {
    pub row_field_cols: Vec<u16>,
    pub col_field_cols: Vec<u16>,
    pub values: Vec<PivotValueDef>,
    pub filters: BTreeMap<u16, Vec<String>>,
    // Compatibility fields for simple 1D pivots
    pub row_field_col: u16,
    pub val_field_col: u16,
    pub aggregation: PivotAggregation,
}

impl Default for PivotConfig {
    fn default() -> Self {
        Self {
            row_field_cols: vec![0],
            col_field_cols: Vec::new(),
            values: vec![PivotValueDef {
                col: 1,
                name: "Value".to_string(),
                aggregation: PivotAggregation::Sum,
                show_as: ShowValuesAs::Normal,
            }],
            filters: BTreeMap::new(),
            row_field_col: 0,
            val_field_col: 1,
            aggregation: PivotAggregation::Sum,
        }
    }
}

pub struct PivotCube;

impl PivotCube {
    /// Computes aggregated value for a collection of numbers
    pub fn compute_aggregation(values: &[f64], agg: PivotAggregation) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        match agg {
            PivotAggregation::Sum => values.iter().sum(),
            PivotAggregation::Count => values.len() as f64,
            PivotAggregation::Average => values.iter().sum::<f64>() / values.len() as f64,
            PivotAggregation::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
            PivotAggregation::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            PivotAggregation::Product => values.iter().product(),
            PivotAggregation::Var => {
                let n = values.len();
                if n < 2 {
                    0.0
                } else {
                    let mean = values.iter().sum::<f64>() / n as f64;
                    let sum_sq_diff: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum();
                    sum_sq_diff / (n - 1) as f64
                }
            }
            PivotAggregation::StdDev => {
                let var = Self::compute_aggregation(values, PivotAggregation::Var);
                var.sqrt()
            }
        }
    }

    /// 1D Pivot table builder for backwards compatibility
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
        output.push(vec![
            CellValue::Text("Row Labels".into()),
            CellValue::Text(format!("{:?} of Value", config.aggregation)),
        ]);

        let mut grand_total_vec = Vec::new();

        for (key, values) in grouped {
            grand_total_vec.extend(values.clone());
            let agg_val = Self::compute_aggregation(&values, config.aggregation);
            output.push(vec![CellValue::Text(key), CellValue::Number(agg_val)]);
        }

        let grand_total = Self::compute_aggregation(&grand_total_vec, config.aggregation);
        output.push(vec![CellValue::Text("Grand Total".into()), CellValue::Number(grand_total)]);

        output
    }

    /// Multi-dimensional 2D Pivot Cube with row groups, column groups, ShowValuesAs, and Slicer filters
    pub fn build_pivot_cube_2d(
        matrix: &SparseMatrix,
        data_rows: u32,
        config: &PivotConfig,
    ) -> Vec<Vec<CellValue>> {
        // Step 1: Filter rows by slicer conditions
        let mut valid_rows = Vec::new();
        for r in 1..=data_rows {
            let mut matches_filter = true;
            for (&filter_col, allowed_values) in &config.filters {
                let cell_val = matrix
                    .get_cell(CellCoordinate::new(r, filter_col))
                    .map(|c| c.value.as_string())
                    .unwrap_or_default();
                if !allowed_values.is_empty() && !allowed_values.contains(&cell_val) {
                    matches_filter = false;
                    break;
                }
            }
            if matches_filter {
                valid_rows.push(r);
            }
        }

        // Step 2: Extract distinct row keys and column keys
        let row_cols = if config.row_field_cols.is_empty() {
            vec![config.row_field_col]
        } else {
            config.row_field_cols.clone()
        };

        let col_cols = &config.col_field_cols;

        let val_defs = if config.values.is_empty() {
            vec![PivotValueDef {
                col: config.val_field_col,
                name: "Value".to_string(),
                aggregation: config.aggregation,
                show_as: ShowValuesAs::Normal,
            }]
        } else {
            config.values.clone()
        };

        let mut row_keys_set = BTreeSet::new();
        let mut col_keys_set = BTreeSet::new();
        let mut cube: BTreeMap<(String, String, usize), Vec<f64>> = BTreeMap::new();
        let mut grand_values: BTreeMap<usize, Vec<f64>> = BTreeMap::new();
        let mut row_values: BTreeMap<(String, usize), Vec<f64>> = BTreeMap::new();
        let mut col_values: BTreeMap<(String, usize), Vec<f64>> = BTreeMap::new();

        for &r in &valid_rows {
            // Build composite row key e.g. "Electronics / Laptops"
            let row_key = row_cols
                .iter()
                .map(|&c| {
                    matrix
                        .get_cell(CellCoordinate::new(r, c))
                        .map(|cell| cell.value.as_string())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join(" | ");

            // Build composite col key e.g. "2026 / Q1"
            let col_key = if col_cols.is_empty() {
                "Total".to_string()
            } else {
                col_cols
                    .iter()
                    .map(|&c| {
                        matrix
                            .get_cell(CellCoordinate::new(r, c))
                            .map(|cell| cell.value.as_string())
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .join(" | ")
            };

            row_keys_set.insert(row_key.clone());
            col_keys_set.insert(col_key.clone());

            for (v_idx, v_def) in val_defs.iter().enumerate() {
                if let Some(val) = matrix
                    .get_cell(CellCoordinate::new(r, v_def.col))
                    .and_then(|c| c.value.as_number())
                {
                    cube.entry((row_key.clone(), col_key.clone(), v_idx))
                        .or_default()
                        .push(val);
                    grand_values.entry(v_idx).or_default().push(val);
                    row_values
                        .entry((row_key.clone(), v_idx))
                        .or_default()
                        .push(val);
                    col_values
                        .entry((col_key.clone(), v_idx))
                        .or_default()
                        .push(val);
                }
            }
        }

        let sorted_row_keys: Vec<String> = row_keys_set.into_iter().collect();
        let sorted_col_keys: Vec<String> = if col_keys_set.is_empty() {
            vec!["Total".to_string()]
        } else {
            col_keys_set.into_iter().collect()
        };

        // Step 3: Compute grand totals per value metric for ShowValuesAs
        let mut grand_totals: Vec<f64> = Vec::new();
        for (v_idx, v_def) in val_defs.iter().enumerate() {
            let vals = grand_values.get(&v_idx).cloned().unwrap_or_default();
            grand_totals.push(Self::compute_aggregation(&vals, v_def.aggregation));
        }

        // Step 4: Construct output 2D grid
        let mut output = Vec::new();

        // Header Row 1: Column groups
        let mut header_row = vec![CellValue::Text("Row Labels".into())];
        for col_k in &sorted_col_keys {
            for v_def in &val_defs {
                header_row.push(CellValue::Text(format!("{} - {}", col_k, v_def.name)));
            }
        }
        header_row.push(CellValue::Text("Grand Total".into()));
        output.push(header_row);

        // Data Rows
        for row_k in &sorted_row_keys {
            let mut row_data = vec![CellValue::Text(row_k.clone())];

            // Row total per value def
            let mut row_metric_totals: Vec<f64> = Vec::new();
            for (v_idx, v_def) in val_defs.iter().enumerate() {
                let r_vals = row_values.get(&(row_k.clone(), v_idx)).cloned().unwrap_or_default();
                row_metric_totals.push(Self::compute_aggregation(&r_vals, v_def.aggregation));
            }

            for col_k in &sorted_col_keys {
                for (v_idx, v_def) in val_defs.iter().enumerate() {
                    let cell_nums = cube
                        .get(&(row_k.clone(), col_k.clone(), v_idx))
                        .cloned()
                        .unwrap_or_default();
                    let raw_agg = Self::compute_aggregation(&cell_nums, v_def.aggregation);

                    let final_val = match v_def.show_as {
                        ShowValuesAs::Normal => raw_agg,
                        ShowValuesAs::PercentOfGrandTotal => {
                            let gt = grand_totals[v_idx];
                            if gt != 0.0 { (raw_agg / gt) * 100.0 } else { 0.0 }
                        }
                        ShowValuesAs::PercentOfRowTotal => {
                            let rt = row_metric_totals[v_idx];
                            if rt != 0.0 { (raw_agg / rt) * 100.0 } else { 0.0 }
                        }
                        ShowValuesAs::PercentOfColumnTotal => {
                            let c_vals = col_values.get(&(col_k.clone(), v_idx)).cloned().unwrap_or_default();
                            let ct = Self::compute_aggregation(&c_vals, v_def.aggregation);
                            if ct != 0.0 { (raw_agg / ct) * 100.0 } else { 0.0 }
                        }
                    };

                    row_data.push(CellValue::Number(final_val));
                }
            }

            // Append Row Grand Total (first value metric)
            row_data.push(CellValue::Number(row_metric_totals.first().copied().unwrap_or(0.0)));
            output.push(row_data);
        }

        // Final Grand Total Row
        let mut total_row = vec![CellValue::Text("Grand Total".into())];
        for col_k in &sorted_col_keys {
            for (v_idx, v_def) in val_defs.iter().enumerate() {
                let c_vals = col_values.get(&(col_k.clone(), v_idx)).cloned().unwrap_or_default();
                let ct = Self::compute_aggregation(&c_vals, v_def.aggregation);
                total_row.push(CellValue::Number(ct));
            }
        }
        total_row.push(CellValue::Number(grand_totals.first().copied().unwrap_or(0.0)));
        output.push(total_row);

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
            ..Default::default()
        };

        let pivot = PivotCube::build_pivot(&matrix, 3, &config);
        assert_eq!(pivot.len(), 4); // Header + Hardware + Software + Grand Total
        assert_eq!(pivot[1][1], CellValue::Number(250.0)); // Hardware total
        assert_eq!(pivot[2][1], CellValue::Number(300.0)); // Software total
        assert_eq!(pivot[3][1], CellValue::Number(550.0)); // Grand total
    }

    #[test]
    fn test_pivot_cube_2d_and_show_values_as() {
        let mut matrix = SparseMatrix::new();
        // Headers: Category, Region, Amount
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Category".into()));
        matrix.set_value(CellCoordinate::new(0, 1), CellValue::Text("Region".into()));
        matrix.set_value(CellCoordinate::new(0, 2), CellValue::Text("Amount".into()));

        // Row 1: Hardware, North, 100
        matrix.set_value(CellCoordinate::new(1, 0), CellValue::Text("Hardware".into()));
        matrix.set_value(CellCoordinate::new(1, 1), CellValue::Text("North".into()));
        matrix.set_value(CellCoordinate::new(1, 2), CellValue::Number(100.0));

        // Row 2: Hardware, South, 300
        matrix.set_value(CellCoordinate::new(2, 0), CellValue::Text("Hardware".into()));
        matrix.set_value(CellCoordinate::new(2, 1), CellValue::Text("South".into()));
        matrix.set_value(CellCoordinate::new(2, 2), CellValue::Number(300.0));

        // Row 3: Software, North, 200
        matrix.set_value(CellCoordinate::new(3, 0), CellValue::Text("Software".into()));
        matrix.set_value(CellCoordinate::new(3, 1), CellValue::Text("North".into()));
        matrix.set_value(CellCoordinate::new(3, 2), CellValue::Number(200.0));

        // Row 4: Software, South, 400
        matrix.set_value(CellCoordinate::new(4, 0), CellValue::Text("Software".into()));
        matrix.set_value(CellCoordinate::new(4, 1), CellValue::Text("South".into()));
        matrix.set_value(CellCoordinate::new(4, 2), CellValue::Number(400.0));

        let mut config = PivotConfig {
            row_field_cols: vec![0], // Category
            col_field_cols: vec![1], // Region
            values: vec![PivotValueDef {
                col: 2,
                name: "Revenue".to_string(),
                aggregation: PivotAggregation::Sum,
                show_as: ShowValuesAs::Normal,
            }],
            ..Default::default()
        };

        let pivot = PivotCube::build_pivot_cube_2d(&matrix, 4, &config);
        // Header + Hardware + Software + Grand Total = 4 rows
        assert_eq!(pivot.len(), 4);
        // North col, South col, Grand Total
        assert_eq!(pivot[1][0], CellValue::Text("Hardware".into()));
        assert_eq!(pivot[1][1], CellValue::Number(100.0)); // Hardware - North
        assert_eq!(pivot[1][2], CellValue::Number(300.0)); // Hardware - South
        assert_eq!(pivot[1][3], CellValue::Number(400.0)); // Hardware Total

        // Test Percent of Grand Total (Grand Total = 1000.0)
        config.values[0].show_as = ShowValuesAs::PercentOfGrandTotal;
        let pct_pivot = PivotCube::build_pivot_cube_2d(&matrix, 4, &config);
        assert_eq!(pct_pivot[1][1], CellValue::Number(10.0)); // 100 / 1000 = 10%
        assert_eq!(pct_pivot[1][2], CellValue::Number(300.0 / 1000.0 * 100.0)); // 30%
    }
}
