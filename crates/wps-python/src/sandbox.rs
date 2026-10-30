//! Unified Python Sandbox & Execution Governor (`wps-python`)
//!
//! Provides the cross-platform execution contract for embedded Python execution,
//! enforcing timeouts, memory caps, safe AST parsing, and deterministic variable inspection.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wps_sheets::cell::CellValue;
use wps_sheets::sparse_matrix::SparseMatrix;
use wps_writer::ast::Document as WriterDocument;
use wps_slides::scene_graph::PresentationDeck as SlidesDeck;
use wps_pdf::PdfDocument;

/// Execution request for a single spreadsheet `=PY(...)` cell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyCellExecRequest {
    pub script: String,
    pub cell_ref: Option<String>,
    pub active_sheet: Option<String>,
    pub output_mode: Option<String>, // "scalar", "spill", "object", "plot"
    pub max_execution_ms: Option<u64>,
}

/// Output modes returned by `=PY()` cell evaluation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PyEvalOutput {
    /// Mode 1: Single scalar cell value (Number, Text, Boolean, Empty).
    Scalar(CellValue),
    /// Mode 2: Dynamic array spill matrix (2D grid of values).
    SpillMatrix(Vec<Vec<CellValue>>),
    /// Mode 3: Python object handle (e.g. DataFrame) displayed as a Mercury chip.
    ObjectHandle {
        handle_id: String,
        class_name: String,
        shape: (usize, usize),
        preview_rows: Vec<Vec<String>>,
    },
    /// Mode 4: Vector SVG chart rendered by Matplotlib.
    SvgPlot {
        svg_xml: String,
        title: String,
        width_pt: f32,
        height_pt: f32,
    },
}

impl PyEvalOutput {
    pub fn to_cell_value(&self) -> CellValue {
        match self {
            PyEvalOutput::Scalar(v) => v.clone(),
            PyEvalOutput::SpillMatrix(matrix) => {
                if matrix.is_empty() || matrix[0].is_empty() {
                    CellValue::Empty
                } else if matrix.len() == 1 && matrix[0].len() == 1 {
                    matrix[0][0].clone()
                } else {
                    CellValue::Text(format!("<Array: {}x{}>", matrix.len(), matrix[0].len()))
                }
            }
            PyEvalOutput::ObjectHandle { class_name, shape, .. } => {
                CellValue::Text(format!("[🐍 {} ({}x{})]", class_name, shape.0, shape.1))
            }
            PyEvalOutput::SvgPlot { title, .. } => {
                CellValue::Text(format!("<Plot: {}>", title))
            }
        }
    }
}

/// Runtime errors during Python evaluation or macro execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum PyRuntimeError {
    #[error("Python Execution Timeout: exceeded {0}ms")]
    Timeout(u64),
    #[error("Memory Cap Exceeded: output exceeded {0} cells")]
    MemoryCapExceeded(usize),
    #[error("Python Syntax Error: {0}")]
    SyntaxError(String),
    #[error("Python Name Error: {0}")]
    NameError(String),
    #[error("Python Runtime Error: {0}")]
    ExecutionError(String),
    #[error("Permission Denied: OS import or network/filesystem restricted: {0}")]
    PermissionDenied(String),
}

/// Context provided to cross-application `wp.*` Python macros.
pub struct WorkspaceMacroContext<'a> {
    pub writer: Option<&'a mut WriterDocument>,
    pub sheets: Option<&'a mut SparseMatrix>,
    pub slides: Option<&'a mut SlidesDeck>,
    pub pdf: Option<&'a mut PdfDocument>,
}

/// Execution report returned after running a multi-document macro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyMacroReport {
    pub actions_performed: usize,
    pub messages: Vec<String>,
    pub duration_us: u64,
    pub success: bool,
}

/// Variable summary for inspection in the Python Studio side drawer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PyVariableSummary {
    pub name: String,
    pub type_name: String,
    pub shape: Option<(usize, usize)>,
    pub preview: String,
}

/// Unified cross-platform Python sandbox trait.
pub trait PythonSandbox: Send + Sync {
    /// Executes a spreadsheet formula cell script with dependency tracking & safety caps.
    fn execute_cell_script(
        &mut self,
        req: PyCellExecRequest,
        matrix: &SparseMatrix,
    ) -> Result<PyEvalOutput, PyRuntimeError>;

    /// Executes a multi-document automation macro.
    fn execute_workspace_macro(
        &mut self,
        script: &str,
        ctx: &mut WorkspaceMacroContext,
    ) -> Result<PyMacroReport, PyRuntimeError>;

    /// Returns all variables currently bound in the global Python session.
    fn inspect_variables(&self) -> BTreeMap<String, PyVariableSummary>;

    /// Clears the global Python namespace.
    fn reset_namespace(&mut self);
}

/// Configurable resource governor for Python executions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonResourceGovernor {
    pub max_execution_ms: u64,
    pub max_output_cells: usize,
    pub allow_network: bool,
    pub allow_filesystem: bool,
}

impl Default for PythonResourceGovernor {
    fn default() -> Self {
        Self {
            max_execution_ms: 5000,
            max_output_cells: 100_000,
            allow_network: false,
            allow_filesystem: false,
        }
    }
}
