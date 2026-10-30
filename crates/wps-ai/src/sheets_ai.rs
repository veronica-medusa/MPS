//! AST-Grounded `@SheetsAI` Co-Pilot & Data Analyst (`wps-ai`)
//!
//! Implements:
//! 1. Schema Context Extraction: headers, inferred types (`Numeric`, `Currency`, `Date`, `Category`, `Text`), sample values.
//! 2. Natural Language to Formula (e.g. `=SUMIFS(...)`, `=XLOOKUP(...)`).
//! 3. Natural Language to Embedded Python `=PY("...")`.
//! 4. Statistical Anomaly & Insights Engine: mean, variance, Z-scores ($|Z| > 2.5$), Pearson correlation.
//! 5. Automated Pivot Table & Chart Recommender.

use crate::provider::{BuiltInLocalEngine, LlmClient, LlmPromptRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wps_sheets::cell::{CellCoordinate, CellRange};
use wps_sheets::sparse_matrix::SparseMatrix;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    Numeric,
    Currency,
    Date,
    Category,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub col_index: u16,
    pub header: String,
    pub inferred_type: ColumnType,
    pub sample_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSchema {
    pub table_range: CellRange,
    pub columns: Vec<ColumnSchema>,
    pub total_rows: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetAnomaly {
    pub coord: CellCoordinate,
    pub value: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub z_score: f64,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorrelationPair {
    pub col_a: String,
    pub col_b: String,
    pub pearson_r: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetInsightsReport {
    pub summary: String,
    pub anomalies: Vec<SheetAnomaly>,
    pub correlations: Vec<CorrelationPair>,
    pub metrics: Vec<(String, f64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChartRecommendation {
    pub chart_type: String, // "Column", "Line", "Scatter", "Bar"
    pub title: String,
    pub category_col: u16,
    pub value_cols: Vec<u16>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PivotRecommendation {
    pub row_fields: Vec<String>,
    pub column_fields: Vec<String>,
    pub value_field: String,
    pub aggregation: String, // "SUM", "AVERAGE", "COUNT"
    pub chart: Option<ChartRecommendation>,
}

pub struct SheetsAi;

impl SheetsAi {
    /// Extracts table schema and column types from a worksheet region.
    pub fn extract_schema(matrix: &SparseMatrix, range: CellRange) -> SheetSchema {
        let mut columns = Vec::new();
        let header_row = range.start.row;

        for col in range.start.col..=range.end.col {
            let header_coord = CellCoordinate::new(header_row, col);
            let header = matrix
                .get_cell(header_coord)
                .map(|c| c.value.as_string())
                .unwrap_or_else(|| format!("Col_{}", col + 1));

            let mut samples = Vec::new();
            let mut num_numeric = 0;
            let mut num_currency = 0;
            let mut num_date = 0;
            let mut total_data_rows = 0;

            for row in (header_row + 1)..=range.end.row {
                let cell_coord = CellCoordinate::new(row, col);
                if let Some(cell) = matrix.get_cell(cell_coord) {
                    if !cell.value.is_empty() {
                        total_data_rows += 1;
                        let s = cell.value.as_string();
                        if samples.len() < 3 {
                            samples.push(s.clone());
                        }

                        if s.starts_with('$') || s.starts_with('€') || s.starts_with('£') {
                            num_currency += 1;
                        } else if s.contains('-') && s.len() == 10 {
                            num_date += 1;
                        } else if cell.value.as_number().is_some() {
                            num_numeric += 1;
                        }
                    }
                }
            }

            let inferred_type = if num_currency > total_data_rows / 2 && total_data_rows > 0 {
                ColumnType::Currency
            } else if num_date > total_data_rows / 2 && total_data_rows > 0 {
                ColumnType::Date
            } else if num_numeric > total_data_rows / 2 && total_data_rows > 0 {
                ColumnType::Numeric
            } else if total_data_rows > 0 && samples.iter().all(|x| x.len() < 20) {
                ColumnType::Category
            } else {
                ColumnType::Text
            };

            columns.push(ColumnSchema {
                col_index: col,
                header,
                inferred_type,
                sample_values: samples,
            });
        }

        let total_rows = if range.end.row >= range.start.row {
            range.end.row - range.start.row
        } else {
            0
        };

        SheetSchema {
            table_range: range,
            columns,
            total_rows,
        }
    }

    /// Converts a natural language query into a high-performance formula.
    pub fn natural_language_to_formula(schema: &SheetSchema, prompt: &str) -> String {
        let p = prompt.to_lowercase();
        let engine = BuiltInLocalEngine::new();

        let req = LlmPromptRequest {
            system_prompt: "You are an Excel / Spreadsheets formula expert.".to_string(),
            user_prompt: format!(
                "Generate a formula for schema columns {:?} and query: {}",
                schema.columns.iter().map(|c| &c.header).collect::<Vec<_>>(),
                prompt
            ),
            temperature: 0.1,
            max_tokens: 128,
        };

        if let Ok(res) = engine.generate_completion(&req) {
            if res.content.starts_with('=') {
                return res.content.trim().to_string();
            }
        }

        // Deterministic heuristics
        if p.contains("sum") && p.contains("revenue") {
            let start_a1 = schema.table_range.start.to_a1_string();
            let end_a1 = schema.table_range.end.to_a1_string();
            format!("=SUM({}:{})", start_a1, end_a1)
        } else if p.contains("average") || p.contains("mean") {
            let start_a1 = schema.table_range.start.to_a1_string();
            let end_a1 = schema.table_range.end.to_a1_string();
            format!("=AVERAGE({}:{})", start_a1, end_a1)
        } else if p.contains("lookup") || p.contains("find") {
            "=XLOOKUP(A2, Products!A:A, Products!C:C, \"Not Found\", 0)".to_string()
        } else {
            format!("=SUM({}:{})", schema.table_range.start.to_a1_string(), schema.table_range.end.to_a1_string())
        }
    }

    /// Converts a natural language query into an embedded Python `=PY("...")` expression.
    pub fn natural_language_to_python(schema: &SheetSchema, prompt: &str) -> String {
        let p = prompt.to_lowercase();
        let range_str = format!("{}:{}", schema.table_range.start.to_a1_string(), schema.table_range.end.to_a1_string());

        if p.contains("groupby") || (p.contains("group") && p.contains("sum")) {
            let cat_col = schema.columns.iter().find(|c| c.inferred_type == ColumnType::Category || c.inferred_type == ColumnType::Text)
                .map(|c| c.header.as_str())
                .unwrap_or("Region");
            let num_col = schema.columns.iter().find(|c| c.inferred_type == ColumnType::Numeric || c.inferred_type == ColumnType::Currency)
                .map(|c| c.header.as_str())
                .unwrap_or("Revenue");
            format!("=PY(\"df = xl('{}', headers=True)\\ndf.groupby('{}')['{}'].sum()\")", range_str, cat_col, num_col)
        } else if p.contains("describe") || p.contains("summary") {
            format!("=PY(\"df = xl('{}', headers=True)\\ndf.describe()\")", range_str)
        } else {
            format!("=PY(\"df = xl('{}', headers=True)\\ndf.head(10)\")", range_str)
        }
    }

    /// Computes statistical profile, Z-score outliers ($|Z| > 2.5$), and Pearson correlation.
    pub fn detect_anomalies_and_insights(matrix: &SparseMatrix, range: CellRange) -> SheetInsightsReport {
        let mut anomalies = Vec::new();
        let mut col_values: HashMap<u16, Vec<(u32, f64)>> = HashMap::new();
        let mut col_names: HashMap<u16, String> = HashMap::new();

        // 1. Gather values
        let header_row = range.start.row;
        for col in range.start.col..=range.end.col {
            let header_coord = CellCoordinate::new(header_row, col);
            let name = matrix.get_cell(header_coord)
                .map(|c| c.value.as_string())
                .unwrap_or_else(|| format!("Col_{}", col + 1));
            col_names.insert(col, name);

            let mut vals = Vec::new();
            for row in (header_row + 1)..=range.end.row {
                let coord = CellCoordinate::new(row, col);
                if let Some(cell) = matrix.get_cell(coord) {
                    if let Some(n) = cell.value.as_number() {
                        vals.push((row, n));
                    }
                }
            }
            if !vals.is_empty() {
                col_values.insert(col, vals);
            }
        }

        let mut metrics = Vec::new();

        // 2. Anomaly Detection (Z-scores)
        for (col, vals) in &col_values {
            let n = vals.len() as f64;
            if n < 3.0 {
                continue;
            }

            let sum: f64 = vals.iter().map(|(_, v)| *v).sum();
            let mean = sum / n;
            let variance: f64 = vals.iter().map(|(_, v)| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
            let std_dev = variance.sqrt();

            let col_name = col_names.get(col).cloned().unwrap_or_default();
            metrics.push((format!("{}_mean", col_name), mean));
            metrics.push((format!("{}_std_dev", col_name), std_dev));

            if std_dev > 1e-6 {
                for (row, val) in vals {
                    let z_score = (val - mean) / std_dev;
                    if z_score.abs() > 2.5 {
                        anomalies.push(SheetAnomaly {
                            coord: CellCoordinate::new(*row, *col),
                            value: *val,
                            mean,
                            std_dev,
                            z_score,
                            description: format!(
                                "Value {:.2} in {} exceeds normal range by {:.1} standard deviations",
                                val, col_name, z_score
                            ),
                        });
                    }
                }
            }
        }

        // 3. Pearson Correlation between numeric columns
        let mut correlations = Vec::new();
        let cols_vec: Vec<u16> = col_values.keys().cloned().collect();
        for i in 0..cols_vec.len() {
            for j in (i + 1)..cols_vec.len() {
                let c1 = cols_vec[i];
                let c2 = cols_vec[j];
                let v1 = &col_values[&c1];
                let v2 = &col_values[&c2];

                // Align by row
                let map2: HashMap<u32, f64> = v2.iter().cloned().collect();
                let mut pairs = Vec::new();
                for (r, val1) in v1 {
                    if let Some(val2) = map2.get(r) {
                        pairs.push((*val1, *val2));
                    }
                }

                if pairs.len() >= 3 {
                    let n = pairs.len() as f64;
                    let m1: f64 = pairs.iter().map(|(x, _)| *x).sum::<f64>() / n;
                    let m2: f64 = pairs.iter().map(|(_, y)| *y).sum::<f64>() / n;

                    let num: f64 = pairs.iter().map(|(x, y)| (x - m1) * (y - m2)).sum();
                    let den1: f64 = pairs.iter().map(|(x, _)| (x - m1).powi(2)).sum();
                    let den2: f64 = pairs.iter().map(|(_, y)| (y - m2).powi(2)).sum();
                    let den = (den1 * den2).sqrt();

                    if den > 1e-9 {
                        let r = num / den;
                        correlations.push(CorrelationPair {
                            col_a: col_names[&c1].clone(),
                            col_b: col_names[&c2].clone(),
                            pearson_r: (r * 1000.0).round() / 1000.0,
                        });
                    }
                }
            }
        }

        let summary = format!(
            "Analyzed {} rows: detected {} statistical anomalies and {} pairwise column correlations.",
            range.row_count(),
            anomalies.len(),
            correlations.len()
        );

        SheetInsightsReport {
            summary,
            anomalies,
            correlations,
            metrics,
        }
    }

    /// Recommends optimal Pivot Table and Chart configuration for a given dataset.
    pub fn recommend_pivot_and_chart(matrix: &SparseMatrix, range: CellRange) -> PivotRecommendation {
        let schema = Self::extract_schema(matrix, range);

        let category_col = schema.columns.iter().find(|c| c.inferred_type == ColumnType::Category || c.inferred_type == ColumnType::Text);
        let numeric_col = schema.columns.iter().find(|c| c.inferred_type == ColumnType::Numeric || c.inferred_type == ColumnType::Currency);

        let row_field = category_col.map(|c| c.header.clone()).unwrap_or_else(|| "Category".to_string());
        let val_field = numeric_col.map(|c| c.header.clone()).unwrap_or_else(|| "Total".to_string());

        let cat_idx = category_col.map(|c| c.col_index).unwrap_or(range.start.col);
        let val_idx = numeric_col.map(|c| c.col_index).unwrap_or(range.end.col);

        PivotRecommendation {
            row_fields: vec![row_field.clone()],
            column_fields: vec![],
            value_field: val_field.clone(),
            aggregation: "SUM".to_string(),
            chart: Some(ChartRecommendation {
                chart_type: "Column".to_string(),
                title: format!("{} by {}", val_field, row_field),
                category_col: cat_idx,
                value_cols: vec![val_idx],
            }),
        }
    }
}
