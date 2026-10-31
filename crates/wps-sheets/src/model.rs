use crate::cell::{CellCoordinate, CellRange, CellValue};
use crate::sparse_matrix::SparseMatrix;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use wps_core::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SheetVisibility {
    #[default]
    Visible,
    Hidden,
    VeryHidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum CalcMode {
    #[default]
    Automatic,
    Manual,
    Iterative {
        max_iterations: u32,
        max_change: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FreezePane {
    pub split_row: u32,
    pub split_col: u16,
    pub top_left_cell: CellCoordinate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColMetric {
    pub width: f64,
    pub hidden: bool,
    pub outline_level: u8,
}

impl Default for ColMetric {
    fn default() -> Self {
        Self {
            width: 72.0,
            hidden: false,
            outline_level: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowMetric {
    pub height: f64,
    pub hidden: bool,
    pub outline_level: u8,
}

impl Default for RowMetric {
    fn default() -> Self {
        Self {
            height: 20.0,
            hidden: false,
            outline_level: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoFilterState {
    pub range: CellRange,
    pub column_filters: BTreeMap<u16, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionRuleType {
    CellValueIs {
        op: String,
        val1: String,
        val2: Option<String>,
    },
    TextContains(String),
    DuplicateValues,
    UniqueValues,
    TopPercent(f64),
    AboveAverage,
    DataBar {
        min_val: f64,
        max_val: f64,
        bar_color: Color,
    },
    ColorScale {
        min_color: Color,
        max_color: Color,
    },
    CustomFormula(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalFormatRule {
    pub range: CellRange,
    pub rule: ConditionRuleType,
    pub text_color: Option<Color>,
    pub background_color: Option<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValidationAlertStyle {
    #[default]
    Stop,
    Warning,
    Information,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationType {
    AnyValue,
    WholeNumber { min: i64, max: i64 },
    Decimal { min: f64, max: f64 },
    List(Vec<String>),
    Date { start: String, end: String },
    TextLength { min: usize, max: usize },
    CustomFormula(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataValidationRule {
    pub range: CellRange,
    pub rule: ValidationType,
    pub allow_blank: bool,
    pub alert_style: ValidationAlertStyle,
    pub error_title: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Column,
    StackedColumn,
    Bar,
    Line,
    Area,
    Pie,
    Doughnut,
    Scatter,
    Waterfall,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetChart {
    pub id: String,
    pub title: String,
    pub chart_type: ChartType,
    pub data_range: CellRange,
    pub series_in_rows: bool,
    pub pos_x: f64,
    pub pos_y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SparklineType {
    Line,
    Column,
    WinLoss,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SparklineGroup {
    pub spark_type: SparklineType,
    pub source_range: CellRange,
    pub target_cell: CellCoordinate,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredTableColumn {
    pub name: String,
    pub formula: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructuredTable {
    pub name: String,
    pub range: CellRange,
    pub has_header: bool,
    pub has_totals_row: bool,
    pub columns: Vec<StructuredTableColumn>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Worksheet {
    pub id: String,
    pub name: String,
    pub tab_color: Option<Color>,
    pub visibility: SheetVisibility,
    pub matrix: SparseMatrix,
    pub col_metrics: BTreeMap<u16, ColMetric>,
    pub row_metrics: BTreeMap<u32, RowMetric>,
    pub merged_regions: Vec<CellRange>,
    pub freeze_pane: Option<FreezePane>,
    pub auto_filter: Option<AutoFilterState>,
    pub reading_view_spotlight: bool,
    pub conditional_formats: Vec<ConditionalFormatRule>,
    pub data_validations: Vec<DataValidationRule>,
    pub charts: Vec<SheetChart>,
    pub sparklines: Vec<SparklineGroup>,
}

impl Worksheet {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            tab_color: None,
            visibility: SheetVisibility::Visible,
            matrix: SparseMatrix::new(),
            col_metrics: BTreeMap::new(),
            row_metrics: BTreeMap::new(),
            merged_regions: Vec::new(),
            freeze_pane: None,
            auto_filter: None,
            reading_view_spotlight: false,
            conditional_formats: Vec::new(),
            data_validations: Vec::new(),
            charts: Vec::new(),
            sparklines: Vec::new(),
        }
    }

    pub fn set_cell_value(&mut self, coord: CellCoordinate, val: CellValue) {
        self.matrix.set_value(coord, val);
    }

    pub fn get_cell_value(&self, coord: CellCoordinate) -> CellValue {
        self.matrix.get_cell(coord).map(|c| c.value.clone()).unwrap_or(CellValue::Empty)
    }

    pub fn merge_range(&mut self, range: CellRange) {
        self.merged_regions.push(range);
        self.matrix.merged_ranges.push(range);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workbook {
    pub sheets: Vec<Worksheet>,
    pub active_sheet: usize,
    pub named_ranges: BTreeMap<String, String>,
    pub structured_tables: BTreeMap<String, StructuredTable>,
    pub calc_mode: CalcMode,
}

impl Default for Workbook {
    fn default() -> Self {
        Self {
            sheets: vec![Worksheet::new("sheet-1", "Sheet1")],
            active_sheet: 0,
            named_ranges: BTreeMap::new(),
            structured_tables: BTreeMap::new(),
            calc_mode: CalcMode::Automatic,
        }
    }
}

impl Workbook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_worksheet(&self) -> &Worksheet {
        &self.sheets[self.active_sheet.min(self.sheets.len().saturating_sub(1))]
    }

    pub fn active_worksheet_mut(&mut self) -> &mut Worksheet {
        let idx = self.active_sheet.min(self.sheets.len().saturating_sub(1));
        &mut self.sheets[idx]
    }

    pub fn add_sheet(&mut self, name: &str) -> usize {
        let id = format!("sheet-{}", self.sheets.len() + 1);
        let ws = Worksheet::new(&id, name);
        self.sheets.push(ws);
        self.sheets.len() - 1
    }

    pub fn get_sheet_by_name(&self, name: &str) -> Option<&Worksheet> {
        self.sheets.iter().find(|s| s.name.eq_ignore_ascii_case(name))
    }

    pub fn get_sheet_by_name_mut(&mut self, name: &str) -> Option<&mut Worksheet> {
        self.sheets.iter_mut().find(|s| s.name.eq_ignore_ascii_case(name))
    }
}
