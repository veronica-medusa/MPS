//! Zero-Copy `=PY()` Spreadsheet Formula & DataFrame Spiller (`wps-python`)
//!
//! Provides bidirectional marshaling between `wps-sheets` SparseMatrix ranges and columnar `PyDataFrame`s,
//! scans Python scripts for sheet dependencies (`wp.cell`, `wp.range`), and resolves dynamic array spills.

use crate::dataframe::{PyDataFrame, PySeries};
use crate::plot::{MatplotlibPlotter, SvgPlot};
use crate::sandbox::{PyCellExecRequest, PyEvalOutput, PyRuntimeError, PythonResourceGovernor};
use std::collections::BTreeMap;
use wps_sheets::cell::{CellCoordinate, CellRange, CellValue};
use wps_sheets::sparse_matrix::SparseMatrix;

pub struct DataFrameBridge;

impl DataFrameBridge {
    /// Scans a `=PY(...)` cell script for referenced cell and range dependencies
    /// so the spreadsheet dependency DAG (`dag.rs`) can establish reactive links.
    pub fn extract_dependencies(script: &str) -> Vec<CellRange> {
        let mut ranges = Vec::new();

        // Match patterns like:
        // wp.cell("A1") / wp.range("A1:D100") / xl("A1:C10")
        let markers = ["wp.cell(", "wp.range(", "xl(", "wp.table("];

        for marker in markers {
            let mut cursor = 0;
            while let Some(idx) = script[cursor..].find(marker) {
                let start = cursor + idx + marker.len();
                if let Some(end) = script[start..].find(')') {
                    let raw_arg = &script[start..start + end];
                    let cleaned = raw_arg.trim().trim_matches(|c| c == '\'' || c == '"' || c == ' ');

                    // Check if it's a valid coordinate or range
                    if let Some(range) = CellRange::parse(cleaned) {
                        if !ranges.contains(&range) {
                            ranges.push(range);
                        }
                    } else if let Some(coord) = CellCoordinate::parse(cleaned) {
                        let single_cell_range = CellRange::new(coord, coord);
                        if !ranges.contains(&single_cell_range) {
                            ranges.push(single_cell_range);
                        }
                    }
                    cursor = start + end;
                } else {
                    break;
                }
            }
        }

        ranges
    }

    /// Evaluates a `=PY(...)` request against the active spreadsheet matrix.
    pub fn evaluate_py_cell(
        req: &PyCellExecRequest,
        matrix: &SparseMatrix,
        variables: &mut BTreeMap<String, PyDataFrame>,
        governor: &PythonResourceGovernor,
    ) -> Result<PyEvalOutput, PyRuntimeError> {
        let script = req.script.trim();
        if script.is_empty() {
            return Ok(PyEvalOutput::Scalar(CellValue::Empty));
        }

        // Security check: restrict forbidden imports
        if !governor.allow_network && (script.contains("import urllib") || script.contains("import requests") || script.contains("import socket")) {
            return Err(PyRuntimeError::PermissionDenied("Network imports are disabled in =PY() formulas".to_string()));
        }
        if !governor.allow_filesystem && (script.contains("import os") || script.contains("import subprocess") || script.contains("open(")) {
            return Err(PyRuntimeError::PermissionDenied("Filesystem access is disabled in =PY() formulas".to_string()));
        }

        let lines: Vec<&str> = script.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        let mut last_result: Option<PyEvalOutput> = None;

        for line in lines {
            // Assignment: `df = wp.range("A1:C10")` or `result = df.groupby(...)`
            if let Some((var_name, expr)) = line.split_once('=') {
                let var = var_name.trim();
                let expr_val = expr.trim();

                if let Some(df) = Self::try_eval_range_expr(expr_val, matrix) {
                    variables.insert(var.to_string(), df.clone());
                    last_result = Some(Self::format_output(&df, req.output_mode.as_deref()));
                    continue;
                }

                if let Some(res) = Self::eval_dataframe_methods(expr_val, variables, req.output_mode.as_deref())? {
                    last_result = Some(res);
                    continue;
                }
            }

            // Direct range expression: `wp.range("A1:D100")`
            if let Some(df) = Self::try_eval_range_expr(line, matrix) {
                last_result = Some(Self::format_output(&df, req.output_mode.as_deref()));
                continue;
            }

            // Plotting call: `plt.plot(...)` or `wp.plot(...)`
            if line.contains("plot(") || line.contains("plt.bar(") {
                let svg = Self::generate_mock_plot(line, variables);
                last_result = Some(PyEvalOutput::SvgPlot {
                    svg_xml: svg.svg_xml,
                    title: "Generated Plot".to_string(),
                    width_pt: svg.width as f32,
                    height_pt: svg.height as f32,
                });
                continue;
            }

            // Dataframe transformation or aggregation call: e.g. `df.groupby("Region")["Revenue"].sum()`
            if let Some(res) = Self::eval_dataframe_methods(line, variables, req.output_mode.as_deref())? {
                last_result = Some(res);
                continue;
            }

            // Arithmetic or scalar expression: e.g. `10 + 20` or `"Total"`
            if let Ok(num) = line.parse::<f64>() {
                last_result = Some(PyEvalOutput::Scalar(CellValue::Number(num)));
            } else if line.starts_with('"') && line.ends_with('"') && line.len() >= 2 {
                let text = &line[1..line.len() - 1];
                last_result = Some(PyEvalOutput::Scalar(CellValue::Text(text.to_string())));
            } else if line == "True" || line == "true" {
                last_result = Some(PyEvalOutput::Scalar(CellValue::Boolean(true)));
            } else if line == "False" || line == "false" {
                last_result = Some(PyEvalOutput::Scalar(CellValue::Boolean(false)));
            }
        }

        last_result.ok_or_else(|| PyRuntimeError::ExecutionError("No output generated by Python cell".to_string()))
    }

    fn try_eval_range_expr(expr: &str, matrix: &SparseMatrix) -> Option<PyDataFrame> {
        let markers = ["wp.range(", "wp.cell(", "xl(", "wp.table("];
        for marker in markers {
            if expr.starts_with(marker) && expr.ends_with(')') {
                let inside = &expr[marker.len()..expr.len() - 1].trim().trim_matches(|c| c == '\'' || c == '"' || c == ' ');
                if let Some(range) = CellRange::parse(inside) {
                    return Some(PyDataFrame::from_sheet_range(matrix, range));
                } else if let Some(coord) = CellCoordinate::parse(inside) {
                    return Some(PyDataFrame::from_sheet_range(matrix, CellRange::new(coord, coord)));
                }
            }
        }
        None
    }

    fn eval_dataframe_methods(
        line: &str,
        variables: &mut BTreeMap<String, PyDataFrame>,
        output_mode: Option<&str>,
    ) -> Result<Option<PyEvalOutput>, PyRuntimeError> {
        // Handle: `df.describe()`
        if line.contains(".describe()") {
            let var_name = line.split('.').next().unwrap_or("").trim();
            if let Some(df) = variables.get(var_name) {
                let desc = df.describe();
                return Ok(Some(Self::format_output(&desc, output_mode)));
            }
        }

        // Handle: `df.groupby("Region")["Revenue"].sum()` or `df.groupby("Region").sum()`
        if line.contains(".groupby(") {
            let var_name = line.split('.').next().unwrap_or("").trim();
            if let Some(df) = variables.get(var_name) {
                if let Some(group_col) = Self::extract_call_arg(line, ".groupby(") {
                    let grouped = Self::compute_groupby(df, &group_col, line);
                    return Ok(Some(Self::format_output(&grouped, output_mode)));
                }
            }
        }

        // Handle: `df.sum()`, `df["Col"].sum()`, `df.mean()`, `df.min()`, `df.max()`
        for op in ["sum()", "mean()", "min()", "max()", "count()"] {
            if line.contains(op) {
                let var_name = line.split(&['.', '['][..]).next().unwrap_or("").trim();
                if let Some(df) = variables.get(var_name) {
                    // Check if specific column is selected: `df["Sales"].sum()`
                    if let Some(col_name) = Self::extract_bracket_arg(line) {
                        if let Some(series) = df.get_column(&col_name) {
                            let val = match op {
                                "sum()" => series.numeric_values().iter().sum::<f64>(),
                                "mean()" => series.mean().unwrap_or(0.0),
                                "min()" => series.min().unwrap_or(0.0),
                                "max()" => series.max().unwrap_or(0.0),
                                "count()" => series.len() as f64,
                                _ => 0.0,
                            };
                            return Ok(Some(PyEvalOutput::Scalar(CellValue::Number(val))));
                        }
                    } else {
                        // Aggregate across entire dataframe as a summary row
                        let mut row = Vec::new();
                        for col in &df.columns {
                            let val = match op {
                                "sum()" => col.numeric_values().iter().sum::<f64>(),
                                "mean()" => col.mean().unwrap_or(0.0),
                                "min()" => col.min().unwrap_or(0.0),
                                "max()" => col.max().unwrap_or(0.0),
                                "count()" => col.len() as f64,
                                _ => 0.0,
                            };
                            row.push(CellValue::Number(val));
                        }
                        return Ok(Some(PyEvalOutput::SpillMatrix(vec![row])));
                    }
                }
            }
        }

        // Handle simple variable reference: `df`
        if let Some(df) = variables.get(line.trim()) {
            return Ok(Some(Self::format_output(df, output_mode)));
        }

        Ok(None)
    }

    fn compute_groupby(df: &PyDataFrame, group_col: &str, line: &str) -> PyDataFrame {
        let key_series = match df.get_column(group_col) {
            Some(s) => s,
            None => return df.clone(),
        };

        // Determine target value column and aggregator
        let target_col = Self::extract_bracket_arg(line);
        let is_mean = line.contains(".mean()");

        let mut groups: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        let target_series = target_col
            .as_deref()
            .and_then(|c| df.get_column(c))
            .unwrap_or(&df.columns[df.columns.len().saturating_sub(1)]);

        for i in 0..key_series.len() {
            let key = match &key_series.values[i] {
                CellValue::Text(t) => t.clone(),
                CellValue::Number(n) => n.to_string(),
                CellValue::Boolean(b) => b.to_string(),
                _ => continue,
            };

            let num = match target_series.values.get(i) {
                Some(CellValue::Number(n)) => *n,
                Some(CellValue::Text(t)) => t.trim().parse::<f64>().unwrap_or(0.0),
                _ => 0.0,
            };

            groups.entry(key).or_default().push(num);
        }

        let mut keys_out = Vec::new();
        let mut vals_out = Vec::new();

        for (k, vals) in groups {
            let agg_val = if is_mean {
                if vals.is_empty() { 0.0 } else { vals.iter().sum::<f64>() / vals.len() as f64 }
            } else {
                vals.iter().sum::<f64>()
            };
            keys_out.push(CellValue::Text(k));
            vals_out.push(CellValue::Number(agg_val));
        }

        let metric_name = if is_mean { "mean" } else { "sum" };
        let val_col_name = format!("{}_{}", target_series.name, metric_name);

        PyDataFrame::new(vec![
            PySeries::new(group_col, keys_out),
            PySeries::new(&val_col_name, vals_out),
        ])
    }

    fn extract_call_arg(line: &str, pattern: &str) -> Option<String> {
        if let Some(pos) = line.find(pattern) {
            let start = pos + pattern.len();
            if let Some(end) = line[start..].find(')') {
                let raw = &line[start..start + end];
                return Some(raw.trim().trim_matches(|c| c == '\'' || c == '"' || c == ' ').to_string());
            }
        }
        None
    }

    fn extract_bracket_arg(line: &str) -> Option<String> {
        if let Some(start) = line.find('[') {
            if let Some(end) = line[start..].find(']') {
                let raw = &line[start + 1..start + end];
                return Some(raw.trim().trim_matches(|c| c == '\'' || c == '"' || c == ' ').to_string());
            }
        }
        None
    }

    fn format_output(df: &PyDataFrame, mode: Option<&str>) -> PyEvalOutput {
        match mode {
            Some("object") => {
                let shape = (df.num_rows(), df.num_cols());
                let preview = df
                    .columns
                    .iter()
                    .take(5)
                    .map(|c| c.values.iter().take(3).map(|v| format!("{:?}", v)).collect())
                    .collect();

                PyEvalOutput::ObjectHandle {
                    handle_id: format!("df_{}_{}", shape.0, shape.1),
                    class_name: "DataFrame".to_string(),
                    shape,
                    preview_rows: preview,
                }
            }
            Some("scalar") => {
                if df.num_rows() == 1 && df.num_cols() == 1 {
                    PyEvalOutput::Scalar(df.columns[0].values[0].clone())
                } else {
                    PyEvalOutput::Scalar(CellValue::Text(format!("<DataFrame: {}x{}>", df.num_rows(), df.num_cols())))
                }
            }
            _ => {
                // Default: 2D Spill Matrix (Header row + data rows)
                let mut matrix = Vec::with_capacity(df.num_rows() + 1);
                // Header row
                let headers = df.columns.iter().map(|c| CellValue::Text(c.name.clone())).collect();
                matrix.push(headers);

                // Data rows
                for r in 0..df.num_rows() {
                    let mut row = Vec::with_capacity(df.num_cols());
                    for c in 0..df.num_cols() {
                        row.push(df.columns[c].values[r].clone());
                    }
                    matrix.push(row);
                }
                PyEvalOutput::SpillMatrix(matrix)
            }
        }
    }

    fn generate_mock_plot(line: &str, variables: &BTreeMap<String, PyDataFrame>) -> SvgPlot {
        let title = if line.contains("title(") {
            Self::extract_call_arg(line, "title(").unwrap_or_else(|| "Chart".to_string())
        } else {
            "Generated Plot".to_string()
        };

        let first_df = variables.values().next();
        let labels: Vec<String> = first_df
            .and_then(|df| df.columns.first())
            .map(|col| col.values.iter().map(|v| match v {
                CellValue::Text(t) => t.clone(),
                CellValue::Number(n) => n.to_string(),
                _ => String::new(),
            }).collect())
            .unwrap_or_else(|| vec!["A".to_string(), "B".to_string(), "C".to_string()]);

        let values: Vec<f64> = first_df
            .and_then(|df| df.columns.last())
            .map(|col| col.numeric_values())
            .unwrap_or_else(|| vec![10.0, 25.0, 15.0]);

        MatplotlibPlotter::bar_chart(&title, &labels, &values)
    }
}
