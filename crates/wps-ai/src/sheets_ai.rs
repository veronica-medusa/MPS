//! `@SheetsAI` — Schema-Aware NL-to-Formula, NL-to-Python & Statistical Anomaly Engine (`wps-ai`)
//!
//! Capabilities:
//! 1. Extracts compact `SheetTableSchema` from raw cell matrices without leaking 1M rows.
//! 2. Natural-Language-to-Formula generator (`SUMIFS`, `AVERAGEIFS`, `COUNTIFS`, `XLOOKUP`, `MAXIFS`, `MINIFS`).
//! 3. Natural-Language-to-Python (`=PY(...)`) generator for `DataFrameBridge`.
//! 4. Automated Statistical Anomaly Detector (Z-score |Z| >= 2.2 & IQR outliers) + Pearson
//!    correlation & linear regression trend discovery.
//! 5. One-click Pivot Table & Chart builder recommendations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnSemanticType {
    Numeric,
    Currency,
    Date,
    Category,
    FreeText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetColumnDescriptor {
    pub col_index: usize,
    pub col_letter: String,
    pub header_name: String,
    pub semantic_type: ColumnSemanticType,
    pub unique_cardinality: usize,
    pub sample_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetTableSchema {
    pub sheet_name: String,
    pub start_row_1based: usize,
    pub end_row_1based: usize,
    pub columns: Vec<SheetColumnDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaGenerationResult {
    pub natural_language_prompt: String,
    pub generated_formula: String,
    pub explanation: String,
    pub referenced_columns: Vec<String>,
    pub python_alternative: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellAnomalyCard {
    pub cell_coord: String,
    pub row_1based: usize,
    pub col_header: String,
    pub numeric_value: f64,
    pub column_mean: f64,
    pub column_std_dev: f64,
    pub z_score: f64,
    pub iqr_fences: (f64, f64),
    pub severity: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnCorrelationInsight {
    pub col_a_header: String,
    pub col_b_header: String,
    pub pearson_r: f64,
    pub regression_slope: f64,
    pub regression_intercept: f64,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PivotAndChartRecommendation {
    pub title: String,
    pub row_field: String,
    pub value_field: String,
    pub aggregation_func: String,
    pub recommended_chart_type: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetDataInsightsReport {
    pub schema: SheetTableSchema,
    pub anomalies: Vec<CellAnomalyCard>,
    pub correlations: Vec<ColumnCorrelationInsight>,
    pub pivot_recommendations: Vec<PivotAndChartRecommendation>,
}

pub struct SheetsAiAgent;

impl SheetsAiAgent {
    /// Converts a 0-based column index into an Excel/WPS column letter (`0 -> A`, `25 -> Z`, `26 -> AA`).
    pub fn col_index_to_letter(mut col_idx: usize) -> String {
        let mut letters = Vec::new();
        loop {
            let rem = col_idx % 26;
            letters.push((b'A' + (rem as u8)) as char);
            if col_idx < 26 {
                break;
            }
            col_idx = (col_idx / 26) - 1;
        }
        letters.into_iter().rev().collect()
    }

    /// Inspects a 2D table (Row 0 = headers, Rows 1..N = data) and builds a `SheetTableSchema`.
    pub fn extract_table_schema(
        sheet_name: &str,
        headers: &[String],
        data_rows: &[Vec<String>],
    ) -> SheetTableSchema {
        let total_rows = data_rows.len();
        let mut columns = Vec::with_capacity(headers.len());

        for (c_idx, header) in headers.iter().enumerate() {
            let mut non_empty = Vec::new();
            let mut unique_set = std::collections::BTreeSet::new();
            let mut numeric_count = 0usize;
            let mut currency_count = 0usize;
            let mut date_count = 0usize;

            for row in data_rows {
                if let Some(cell) = row.get(c_idx) {
                    let trimmed = cell.trim();
                    if !trimmed.is_empty() {
                        non_empty.push(trimmed.to_string());
                        unique_set.insert(trimmed.to_string());
                        if trimmed.starts_with('$')
                            || trimmed.starts_with('€')
                            || trimmed.starts_with('₹')
                        {
                            currency_count += 1;
                        }
                        if Self::parse_numeric_cell(trimmed).is_some() {
                            numeric_count += 1;
                        } else if Self::looks_like_date(trimmed) {
                            date_count += 1;
                        }
                    }
                }
            }

            let count = non_empty.len().max(1);
            let semantic_type = if currency_count * 2 >= count {
                ColumnSemanticType::Currency
            } else if numeric_count * 2 >= count {
                ColumnSemanticType::Numeric
            } else if date_count * 2 >= count {
                ColumnSemanticType::Date
            } else if unique_set.len() <= (total_rows / 2).max(6) {
                ColumnSemanticType::Category
            } else {
                ColumnSemanticType::FreeText
            };

            columns.push(SheetColumnDescriptor {
                col_index: c_idx,
                col_letter: Self::col_index_to_letter(c_idx),
                header_name: header.clone(),
                semantic_type,
                unique_cardinality: unique_set.len(),
                sample_values: non_empty.into_iter().take(3).collect(),
            });
        }

        SheetTableSchema {
            sheet_name: sheet_name.to_string(),
            start_row_1based: 2,
            end_row_1based: (total_rows + 1).max(2),
            columns,
        }
    }

    /// Translates a natural language query into both an exact Excel/WPS formula (`SUMIFS`, `XLOOKUP`,
    /// `AVERAGEIFS`, `COUNTIFS`, etc.) and an equivalent `=PY(...)` DataFrame formula.
    pub fn natural_language_to_formula(
        prompt: &str,
        schema: &SheetTableSchema,
    ) -> FormulaGenerationResult {
        let lower_prompt = prompt.to_ascii_lowercase();
        let r_start = schema.start_row_1based;
        let r_end = schema.end_row_1based;

        // Identify target numeric/value column mentioned in prompt (or default to last numeric column)
        let target_col = schema
            .columns
            .iter()
            .find(|c| {
                lower_prompt.contains(&c.header_name.to_ascii_lowercase())
                    && matches!(
                        c.semantic_type,
                        ColumnSemanticType::Numeric | ColumnSemanticType::Currency
                    )
            })
            .or_else(|| {
                schema.columns.iter().rev().find(|c| {
                    matches!(
                        c.semantic_type,
                        ColumnSemanticType::Numeric | ColumnSemanticType::Currency
                    )
                })
            })
            .unwrap_or(&schema.columns[0]);

        // Identify filter/category column mentioned in prompt
        let filter_col = schema
            .columns
            .iter()
            .find(|c| {
                c.col_index != target_col.col_index
                    && lower_prompt.contains(&c.header_name.to_ascii_lowercase())
            })
            .or_else(|| {
                schema
                    .columns
                    .iter()
                    .find(|c| c.semantic_type == ColumnSemanticType::Category)
            });

        // Extract quoted filter criterion (e.g. 'APAC' or "Enterprise") if present
        let criterion = Self::extract_quoted_literal(prompt).or_else(|| {
            filter_col.and_then(|fc| {
                fc.sample_values
                    .iter()
                    .find(|v| lower_prompt.contains(&v.to_ascii_lowercase()))
                    .cloned()
            })
        });

        let target_range = format!("{}{}:{}{}", target_col.col_letter, r_start, target_col.col_letter, r_end);
        let last_col_letter = schema
            .columns
            .last()
            .map(|c| c.col_letter.as_str())
            .unwrap_or("D");
        let full_table_range = format!("A1:{}{}", last_col_letter, r_end);

        // Lookup intent (`lookup`, `find`, `xlookup`)
        if lower_prompt.contains("lookup") || lower_prompt.contains("find ") {
            if let Some(fc) = filter_col {
                let lookup_val = criterion.unwrap_or_else(|| "G2".to_string());
                let formatted_key = if lookup_val.chars().all(|c| c.is_ascii_alphanumeric())
                    && lookup_val.len() <= 3
                {
                    lookup_val.clone()
                } else {
                    format!("\"{}\"", lookup_val)
                };
                let lookup_range = format!("{}{}:{}{}", fc.col_letter, r_start, fc.col_letter, r_end);
                let formula = format!(
                    "=XLOOKUP({}, {}, {}, \"Not Found\", 0)",
                    formatted_key, lookup_range, target_range
                );
                let py_code = format!(
                    "=PY(\"df = wp.range('{}').to_df(); return df.loc[df['{}'] == '{}', '{}'].iloc[0]\")",
                    full_table_range, fc.header_name, lookup_val, target_col.header_name
                );
                return FormulaGenerationResult {
                    natural_language_prompt: prompt.to_string(),
                    generated_formula: formula,
                    explanation: format!(
                        "Looks up {} in {} ({}) and returns matching {} ({}).",
                        formatted_key,
                        fc.header_name,
                        lookup_range,
                        target_col.header_name,
                        target_range
                    ),
                    referenced_columns: vec![fc.header_name.clone(), target_col.header_name.clone()],
                    python_alternative: py_code,
                };
            }
        }

        // Aggregation intent (`sum`, `total`, `average`, `mean`, `count`, `max`, `min`)
        let is_avg = lower_prompt.contains("average") || lower_prompt.contains("mean");
        let is_count = lower_prompt.contains("count") || lower_prompt.contains("how many");
        let is_max = lower_prompt.contains("max") || lower_prompt.contains("highest");
        let is_min = lower_prompt.contains("min") || lower_prompt.contains("lowest");

        if let (Some(fc), Some(crit_val)) = (filter_col, criterion) {
            let filter_range = format!("{}{}:{}{}", fc.col_letter, r_start, fc.col_letter, r_end);
            let (func_name, py_agg) = if is_avg {
                ("AVERAGEIFS", "mean")
            } else if is_max {
                ("MAXIFS", "max")
            } else if is_min {
                ("MINIFS", "min")
            } else {
                ("SUMIFS", "sum")
            };

            let formula = if is_count {
                format!("=COUNTIFS({}, \"{}\")", filter_range, crit_val)
            } else {
                format!(
                    "={}({}, {}, \"{}\")",
                    func_name, target_range, filter_range, crit_val
                )
            };

            let py_code = format!(
                "=PY(\"df = wp.range('{}').to_df(); return df[df['{}'] == '{}']['{}'].{}()\")",
                full_table_range, fc.header_name, crit_val, target_col.header_name, py_agg
            );

            return FormulaGenerationResult {
                natural_language_prompt: prompt.to_string(),
                generated_formula: formula,
                explanation: format!(
                    "Aggregates {} ({}) where {} ({}) equals \"{}\".",
                    target_col.header_name, target_range, fc.header_name, filter_range, crit_val
                ),
                referenced_columns: vec![target_col.header_name.clone(), fc.header_name.clone()],
                python_alternative: py_code,
            };
        }

        // Unfiltered aggregation fallback
        let (simple_func, py_agg) = if is_avg {
            ("AVERAGE", "mean")
        } else if is_count {
            ("COUNTA", "count")
        } else if is_max {
            ("MAX", "max")
        } else if is_min {
            ("MIN", "min")
        } else {
            ("SUM", "sum")
        };

        FormulaGenerationResult {
            natural_language_prompt: prompt.to_string(),
            generated_formula: format!("={}({})", simple_func, target_range),
            explanation: format!(
                "Computes {} over {} ({}).",
                simple_func, target_col.header_name, target_range
            ),
            referenced_columns: vec![target_col.header_name.clone()],
            python_alternative: format!(
                "=PY(\"df = wp.range('{}').to_df(); return df['{}'].{}()\")",
                full_table_range, target_col.header_name, py_agg
            ),
        }
    }

    /// Scans a 2D dataset for statistical Z-score & IQR anomalies, pairwise Pearson correlations,
    /// and Pivot Table / Chart recommendations.
    pub fn analyze_sheet_insights_and_anomalies(
        sheet_name: &str,
        headers: &[String],
        data_rows: &[Vec<String>],
    ) -> SheetDataInsightsReport {
        type NumericSeries = (usize, String, Vec<(usize, f64)>);

        let schema = Self::extract_table_schema(sheet_name, headers, data_rows);
        let mut anomalies = Vec::new();
        let mut numeric_columns_series: Vec<NumericSeries> = Vec::new();

        for col in &schema.columns {
            if !matches!(
                col.semantic_type,
                ColumnSemanticType::Numeric | ColumnSemanticType::Currency
            ) {
                continue;
            }

            let mut row_vals: Vec<(usize, f64)> = Vec::new();
            for (r_idx, row) in data_rows.iter().enumerate() {
                if let Some(raw) = row.get(col.col_index) {
                    if let Some(val) = Self::parse_numeric_cell(raw) {
                        row_vals.push((r_idx + 2, val)); // 1-based sheet row (row 1 = header)
                    }
                }
            }

            if row_vals.len() >= 4 {
                let values_only: Vec<f64> = row_vals.iter().map(|&(_, v)| v).collect();
                let n = values_only.len() as f64;
                let mean = values_only.iter().sum::<f64>() / n;
                let variance = values_only
                    .iter()
                    .map(|&v| (v - mean) * (v - mean))
                    .sum::<f64>()
                    / n;
                let std_dev = variance.sqrt();

                let mut sorted_vals = values_only.clone();
                sorted_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let q1 = Self::percentile_sorted(&sorted_vals, 0.25);
                let q3 = Self::percentile_sorted(&sorted_vals, 0.75);
                let iqr = (q3 - q1).max(1e-6);
                let lower_fence = q1 - 1.5 * iqr;
                let upper_fence = q3 + 1.5 * iqr;

                for &(row_1based, val) in &row_vals {
                    let z_score = if std_dev > 1e-9 {
                        (val - mean) / std_dev
                    } else {
                        0.0
                    };
                    let is_iqr_outlier = val < lower_fence || val > upper_fence;

                    // Flag statistically significant spikes/drops (|Z| >= 2.2 or strong IQR outlier)
                    if z_score.abs() >= 2.2 || (is_iqr_outlier && z_score.abs() >= 1.8) {
                        let coord = format!("{}{}", col.col_letter, row_1based);
                        let severity = if z_score.abs() >= 2.8 {
                            "Critical Outlier"
                        } else {
                            "Statistical Anomaly"
                        };
                        anomalies.push(CellAnomalyCard {
                            cell_coord: coord.clone(),
                            row_1based,
                            col_header: col.header_name.clone(),
                            numeric_value: val,
                            column_mean: mean,
                            column_std_dev: std_dev,
                            z_score,
                            iqr_fences: (lower_fence, upper_fence),
                            severity: severity.to_string(),
                            explanation: format!(
                                "Cell {} ({:.2}) in '{}' deviates by Z = {:+.2}σ from column mean ({:.2}).",
                                coord, val, col.header_name, z_score, mean
                            ),
                        });
                    }
                }

                numeric_columns_series.push((col.col_index, col.header_name.clone(), row_vals));
            }
        }

        // Compute pairwise Pearson correlation & linear regression across numeric columns
        let mut correlations = Vec::new();
        for i in 0..numeric_columns_series.len() {
            for j in (i + 1)..numeric_columns_series.len() {
                let (_, ref name_a, ref series_a) = numeric_columns_series[i];
                let (_, ref name_b, ref series_b) = numeric_columns_series[j];

                if let Some((r, slope, intercept)) =
                    Self::compute_pearson_and_regression(series_a, series_b)
                {
                    if r.abs() >= 0.50 {
                        let direction = if r > 0.0 { "positive" } else { "negative" };
                        correlations.push(ColumnCorrelationInsight {
                            col_a_header: name_a.clone(),
                            col_b_header: name_b.clone(),
                            pearson_r: r,
                            regression_slope: slope,
                            regression_intercept: intercept,
                            summary: format!(
                                "Strong {} correlation (r = {:.2}) between '{}' and '{}' (y = {:.2}x {:+.2}).",
                                direction, r, name_a, name_b, slope, intercept
                            ),
                        });
                    }
                }
            }
        }

        // Build Pivot Table & Chart recommendations
        let mut pivot_recommendations = Vec::new();
        let cat_cols: Vec<&SheetColumnDescriptor> = schema
            .columns
            .iter()
            .filter(|c| c.semantic_type == ColumnSemanticType::Category)
            .collect();
        let num_cols: Vec<&SheetColumnDescriptor> = schema
            .columns
            .iter()
            .filter(|c| {
                matches!(
                    c.semantic_type,
                    ColumnSemanticType::Numeric | ColumnSemanticType::Currency
                )
            })
            .collect();

        if let (Some(cat), Some(num)) = (cat_cols.first(), num_cols.first()) {
            pivot_recommendations.push(PivotAndChartRecommendation {
                title: format!("{} by {}", num.header_name, cat.header_name),
                row_field: cat.header_name.clone(),
                value_field: num.header_name.clone(),
                aggregation_func: "SUM".to_string(),
                recommended_chart_type: if cat.unique_cardinality <= 6 {
                    "Doughnut / Clustered Column".to_string()
                } else {
                    "Horizontal Bar Chart".to_string()
                },
                rationale: format!(
                    "'{}' has {} distinct groups—ideal for summarizing total '{}'.",
                    cat.header_name, cat.unique_cardinality, num.header_name
                ),
            });
        }

        SheetDataInsightsReport {
            schema,
            anomalies,
            correlations,
            pivot_recommendations,
        }
    }

    fn parse_numeric_cell(raw: &str) -> Option<f64> {
        let cleaned: String = raw
            .trim()
            .chars()
            .filter(|&c| c != '$' && c != '€' && c != '₹' && c != ',' && c != '%' && c != ' ')
            .collect();
        if cleaned.is_empty() {
            return None;
        }
        cleaned.parse::<f64>().ok()
    }

    fn looks_like_date(raw: &str) -> bool {
        let s = raw.trim();
        (s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-'))
            || (s.len() >= 8 && s.matches('/').count() == 2)
    }

    fn extract_quoted_literal(prompt: &str) -> Option<String> {
        for quote in ['\'', '"'] {
            if let Some(start) = prompt.find(quote) {
                if let Some(end_rel) = prompt[start + 1..].find(quote) {
                    let extracted = &prompt[start + 1..start + 1 + end_rel];
                    if !extracted.trim().is_empty() {
                        return Some(extracted.trim().to_string());
                    }
                }
            }
        }
        None
    }

    fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = (p * ((sorted.len() - 1) as f64)).clamp(0.0, (sorted.len() - 1) as f64);
        let lo = idx.floor() as usize;
        let hi = idx.ceil() as usize;
        let frac = idx - (lo as f64);
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    }

    fn compute_pearson_and_regression(
        series_a: &[(usize, f64)],
        series_b: &[(usize, f64)],
    ) -> Option<(f64, f64, f64)> {
        let map_b: std::collections::BTreeMap<usize, f64> = series_b.iter().copied().collect();
        let pairs: Vec<(f64, f64)> = series_a
            .iter()
            .filter_map(|&(r, va)| map_b.get(&r).map(|&vb| (va, vb)))
            .collect();

        if pairs.len() < 3 {
            return None;
        }
        let n = pairs.len() as f64;
        let mean_x = pairs.iter().map(|&(x, _)| x).sum::<f64>() / n;
        let mean_y = pairs.iter().map(|&(_, y)| y).sum::<f64>() / n;

        let mut cov = 0.0;
        let mut var_x = 0.0;
        let mut var_y = 0.0;
        for &(x, y) in &pairs {
            let dx = x - mean_x;
            let dy = y - mean_y;
            cov += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
        }

        if var_x <= 1e-12 || var_y <= 1e-12 {
            return None;
        }
        let r = (cov / (var_x.sqrt() * var_y.sqrt())).clamp(-1.0, 1.0);
        let slope = cov / var_x;
        let intercept = mean_y - slope * mean_x;
        Some((r, slope, intercept))
    }
}
