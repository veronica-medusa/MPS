use crate::dataframe::PyDataFrame;
use crate::plot::{MatplotlibPlotter, SvgPlot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use wps_sheets::cell::{CellRange, CellValue};
use wps_sheets::sparse_matrix::SparseMatrix;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum PyError {
    #[error("Python Syntax Error: {0}")]
    SyntaxError(String),
    #[error("Python Name Error: {0}")]
    NameError(String),
    #[error("Python Runtime Error: {0}")]
    RuntimeError(String),
    #[error("Python Type Error: {0}")]
    TypeError(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PyOutput {
    None,
    Scalar(CellValue),
    DataFrame(PyDataFrame),
    Array2D(Vec<Vec<CellValue>>),
    Plot(SvgPlot),
}

impl PyOutput {
    pub fn to_cell_value(&self) -> CellValue {
        match self {
            PyOutput::None => CellValue::Empty,
            PyOutput::Scalar(v) => v.clone(),
            PyOutput::DataFrame(df) => {
                // If single value, return scalar
                if df.num_cols() == 1 && df.num_rows() == 1 {
                    df.columns[0].values[0].clone()
                } else {
                    CellValue::Text(format!("<DataFrame: {}x{}>", df.num_rows(), df.num_cols()))
                }
            }
            PyOutput::Array2D(arr) => {
                if arr.len() == 1 && arr[0].len() == 1 {
                    arr[0][0].clone()
                } else {
                    CellValue::Text(format!("<Array: {}x{}>", arr.len(), arr.first().map(|r| r.len()).unwrap_or(0)))
                }
            }
            PyOutput::Plot(_) => CellValue::Text("<Plot: SVG Graphic>".to_string()),
        }
    }
}

pub trait PythonEngine: Send + Sync {
    fn eval_cell(&mut self, script: &str, matrix: &SparseMatrix) -> Result<PyOutput, PyError>;
}

/// Native OpenWPS Python Engine
pub struct NativePythonEngine {
    variables: HashMap<String, PyDataFrame>,
}

impl Default for NativePythonEngine {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl NativePythonEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates python expression / script with pandas DataFrame & matplotlib integration
    pub fn execute(&mut self, script: &str, matrix: &SparseMatrix) -> Result<PyOutput, PyError> {
        let lines: Vec<&str> = script.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
        let mut last_output = PyOutput::None;

        for line in lines {
            // Assignment: `var = ...`
            if let Some((var_name, expr)) = line.split_once('=') {
                let var = var_name.trim();
                let expr_val = expr.trim();

                // `var = xl("A1:C10")`
                if expr_val.starts_with("xl(") && expr_val.ends_with(')') {
                    let inside = &expr_val[3..expr_val.len() - 1].trim_matches(|c| c == '\'' || c == '"' || c == ' ');
                    let range = CellRange::parse(inside).ok_or_else(|| PyError::RuntimeError(format!("Invalid range: {}", inside)))?;
                    let df = PyDataFrame::from_sheet_range(matrix, range);
                    self.variables.insert(var.to_string(), df.clone());
                    last_output = PyOutput::DataFrame(df);
                    continue;
                }
            }

            // Method call on DataFrame: `df.describe()`
            if line.contains(".describe()") {
                let var_name = line.split('.').next().unwrap_or("").trim();
                if let Some(df) = self.variables.get(var_name) {
                    let desc = df.describe();
                    last_output = PyOutput::DataFrame(desc);
                    continue;
                } else {
                    return Err(PyError::NameError(format!("name '{}' is not defined", var_name)));
                }
            }

            // Method call: `df['Col'].mean()`, `df['Col'].sum()`, etc.
            if line.contains(".mean()") || line.contains(".sum()") || line.contains(".min()") || line.contains(".max()") {
                if let Some(col_name) = Self::extract_bracket_name(line) {
                    let var_name = line.split('[').next().unwrap_or("").trim();
                    let df = self.variables.get(var_name).ok_or_else(|| PyError::NameError(format!("name '{}' is not defined", var_name)))?;
                    let series = df.get_column(&col_name).ok_or_else(|| PyError::RuntimeError(format!("column '{}' not found", col_name)))?;

                    let val = if line.contains(".mean()") {
                        series.mean().unwrap_or(0.0)
                    } else if line.contains(".sum()") {
                        series.numeric_values().iter().sum::<f64>()
                    } else if line.contains(".min()") {
                        series.min().unwrap_or(0.0)
                    } else {
                        series.max().unwrap_or(0.0)
                    };

                    last_output = PyOutput::Scalar(CellValue::Number(val));
                    continue;
                }
            }

            // Plotting: `plt.plot(...)` or `plt.bar(...)`
            if line.starts_with("plt.plot(") || line.starts_with("plt.bar(") {
                let plot = self.eval_plot_command(line)?;
                last_output = PyOutput::Plot(plot);
                continue;
            }

            // Simple arithmetic or literal evaluation
            if let Ok(num) = line.parse::<f64>() {
                last_output = PyOutput::Scalar(CellValue::Number(num));
                continue;
            }

            // String literal
            if (line.starts_with('"') && line.ends_with('"')) || (line.starts_with('\'') && line.ends_with('\'')) {
                let s = &line[1..line.len() - 1];
                last_output = PyOutput::Scalar(CellValue::Text(s.to_string()));
                continue;
            }

            // Returning existing DataFrame: `df`
            if let Some(df) = self.variables.get(line) {
                last_output = PyOutput::DataFrame(df.clone());
                continue;
            }
        }

        Ok(last_output)
    }

    fn extract_bracket_name(line: &str) -> Option<String> {
        let start = line.find('[')? + 1;
        let end = line.find(']')?;
        if start < end {
            let inner = line[start..end].trim_matches(|c| c == '\'' || c == '"' || c == ' ');
            Some(inner.to_string())
        } else {
            None
        }
    }

    fn eval_plot_command(&self, line: &str) -> Result<SvgPlot, PyError> {
        if line.starts_with("plt.plot(") {
            let inside = &line[9..line.len() - 1];
            let parts: Vec<&str> = inside.split(',').map(|p| p.trim()).collect();
            if parts.is_empty() {
                return Err(PyError::RuntimeError("plt.plot() requires arguments".to_string()));
            }

            // Extract x and y series
            let (x_vals, y_vals) = if parts.len() >= 2 {
                let col_x = Self::extract_bracket_name(parts[0]).unwrap_or_default();
                let col_y = Self::extract_bracket_name(parts[1]).unwrap_or_default();

                let var_x = parts[0].split('[').next().unwrap_or("").trim();
                let var_y = parts[1].split('[').next().unwrap_or("").trim();

                let df_x = self.variables.get(var_x).ok_or_else(|| PyError::NameError(format!("name '{}' is not defined", var_x)))?;
                let df_y = self.variables.get(var_y).ok_or_else(|| PyError::NameError(format!("name '{}' is not defined", var_y)))?;

                let s_x = df_x.get_column(&col_x).map(|s| s.numeric_values()).unwrap_or_default();
                let s_y = df_y.get_column(&col_y).map(|s| s.numeric_values()).unwrap_or_default();
                (s_x, s_y)
            } else {
                let col = Self::extract_bracket_name(parts[0]).unwrap_or_default();
                let var = parts[0].split('[').next().unwrap_or("").trim();
                let df = self.variables.get(var).ok_or_else(|| PyError::NameError(format!("name '{}' is not defined", var)))?;
                let s_y = df.get_column(&col).map(|s| s.numeric_values()).unwrap_or_default();
                let s_x = (0..s_y.len()).map(|i| i as f64).collect();
                (s_x, s_y)
            };

            let plot = MatplotlibPlotter::line_chart("Python Plot", "X", "Y", &x_vals, &y_vals);
            Ok(plot)
        } else if line.starts_with("plt.bar(") {
            let inside = &line[8..line.len() - 1];
            let parts: Vec<&str> = inside.split(',').map(|p| p.trim()).collect();
            if parts.len() < 2 {
                return Err(PyError::RuntimeError("plt.bar() requires categories and values".to_string()));
            }

            let cat_col = Self::extract_bracket_name(parts[0]).unwrap_or_default();
            let val_col = Self::extract_bracket_name(parts[1]).unwrap_or_default();

            let var_cat = parts[0].split('[').next().unwrap_or("").trim();
            let var_val = parts[1].split('[').next().unwrap_or("").trim();

            let df_cat = self.variables.get(var_cat).ok_or_else(|| PyError::NameError(format!("name '{}' is not defined", var_cat)))?;
            let df_val = self.variables.get(var_val).ok_or_else(|| PyError::NameError(format!("name '{}' is not defined", var_val)))?;

            let cats: Vec<String> = df_cat.get_column(&cat_col)
                .map(|s| s.values.iter().map(|v| v.as_string()).collect::<Vec<String>>())
                .unwrap_or_default();
            let vals = df_val.get_column(&val_col).map(|s| s.numeric_values()).unwrap_or_default();

            let plot = MatplotlibPlotter::bar_chart("Python Bar Chart", &cats, &vals);
            Ok(plot)
        } else {
            Err(PyError::RuntimeError("Unsupported plot command".to_string()))
        }
    }
}

impl PythonEngine for NativePythonEngine {
    fn eval_cell(&mut self, script: &str, matrix: &SparseMatrix) -> Result<PyOutput, PyError> {
        self.execute(script, matrix)
    }
}
