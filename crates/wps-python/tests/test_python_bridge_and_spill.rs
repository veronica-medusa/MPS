use std::collections::BTreeMap;
use wps_sheets::cell::{CellCoordinate, CellRange, CellValue};
use wps_sheets::sparse_matrix::SparseMatrix;
use wps_writer::ast::Document as WriterDocument;
use wps_slides::scene_graph::PresentationDeck as SlidesDeck;
use wps_pdf::PdfDocument;
use wps_python::{
    DataFrameBridge, MacroEngine, PyCellExecRequest, PyEvalOutput, PythonResourceGovernor,
    WorkspaceMacroContext,
};

#[test]
fn test_python_groupby_and_spill_matrix() {
    let mut matrix = SparseMatrix::new();

    // Populate a 10-row sales table:
    // Row 0: Headers ("Region", "Product", "Revenue")
    matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Region".to_string()));
    matrix.set_value(CellCoordinate::new(0, 1), CellValue::Text("Product".to_string()));
    matrix.set_value(CellCoordinate::new(0, 2), CellValue::Text("Revenue".to_string()));

    let sales_data = vec![
        ("North", "Widget A", 1200.0),
        ("South", "Widget B", 800.0),
        ("North", "Widget B", 950.0),
        ("East",  "Widget A", 1400.0),
        ("West",  "Widget C", 600.0),
        ("East",  "Widget B", 1100.0),
        ("North", "Widget C", 1300.0),
        ("South", "Widget A", 750.0),
        ("West",  "Widget B", 850.0),
    ];

    for (idx, (reg, prod, rev)) in sales_data.iter().enumerate() {
        let r = (idx + 1) as u32;
        matrix.set_value(CellCoordinate::new(r, 0), CellValue::Text(reg.to_string()));
        matrix.set_value(CellCoordinate::new(r, 1), CellValue::Text(prod.to_string()));
        matrix.set_value(CellCoordinate::new(r, 2), CellValue::Number(*rev));
    }

    // Verify dependency extraction
    let script = "df = wp.range('A1:C10')\nresult = df.groupby('Region')['Revenue'].sum()";
    let deps = DataFrameBridge::extract_dependencies(script);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0], CellRange::parse("A1:C10").unwrap());

    // Execute Python cell
    let req = PyCellExecRequest {
        script: script.to_string(),
        cell_ref: Some("E2".to_string()),
        active_sheet: Some("Sheet1".to_string()),
        output_mode: Some("spill".to_string()),
        max_execution_ms: Some(5000),
    };

    let mut variables = BTreeMap::new();
    let governor = PythonResourceGovernor::default();

    let output = DataFrameBridge::evaluate_py_cell(&req, &matrix, &mut variables, &governor)
        .expect("Python cell execution should succeed");

    match output {
        PyEvalOutput::SpillMatrix(spill) => {
            // Header + 4 distinct regions (East, North, South, West)
            assert_eq!(spill.len(), 5, "Expected header row + 4 grouped region rows");
            assert_eq!(spill[0][0], CellValue::Text("Region".to_string()));
            assert_eq!(spill[0][1], CellValue::Text("Revenue_sum".to_string()));

            // Find North total: 1200 + 950 + 1300 = 3450
            let north_row = spill.iter().find(|r| r[0] == CellValue::Text("North".to_string()));
            assert!(north_row.is_some());
            assert_eq!(north_row.unwrap()[1], CellValue::Number(3450.0));

            // Find East total: 1400 + 1100 = 2500
            let east_row = spill.iter().find(|r| r[0] == CellValue::Text("East".to_string()));
            assert!(east_row.is_some());
            assert_eq!(east_row.unwrap()[1], CellValue::Number(2500.0));
        }
        _ => panic!("Expected SpillMatrix output mode"),
    }
}

#[test]
fn test_cross_document_wp_macro_execution() {
    let mut writer = WriterDocument::new("Strategic Report", "Author");
    let mut sheets = SparseMatrix::new();
    let mut slides = SlidesDeck::new("Q4 Business Review");
    let mut pdf = PdfDocument::new("Annual Filing");

    // Initialize sheets data
    sheets.set_value(CellCoordinate::new(0, 0), CellValue::Text("Target".to_string()));
    sheets.set_value(CellCoordinate::new(0, 1), CellValue::Number(500000.0));

    let macro_script = r#"
# Cross-Document Python Automation Macro
wp.writer.insert_heading("Executive Financial Summary", level=1)
wp.writer.insert_paragraph("The following projections were synchronized directly from the workbook:")
wp.sheets.set_cell("B1", 750000.0)
wp.slides.add_slide("Q4 Strategy & Revenue Projections")
wp.pdf.add_watermark("CONFIDENTIAL")
"#;

    let mut ctx = WorkspaceMacroContext {
        writer: Some(&mut writer),
        sheets: Some(&mut sheets),
        slides: Some(&mut slides),
        pdf: Some(&mut pdf),
    };

    let report = MacroEngine::execute(macro_script, &mut ctx)
        .expect("Macro execution should succeed");

    assert_eq!(report.actions_performed, 5);
    assert!(report.success);

    // Verify Writer changes
    assert_eq!(writer.sections[0].elements.len(), 2);
    let h1 = &writer.sections[0].elements[0];
    assert!(format!("{:?}", h1).contains("Executive Financial Summary"));

    // Verify Sheets changes
    assert_eq!(
        sheets.get_cell(CellCoordinate::new(0, 1)).map(|c| &c.value),
        Some(&CellValue::Number(750000.0))
    );

    // Verify Slides changes (initial deck had 1 slide, macro added 2nd slide)
    assert_eq!(slides.slides.len(), 2);
    let added_slide = &slides.slides[1];
    assert_eq!(added_slide.nodes.len(), 1);
    match &added_slide.nodes[0] {
        wps_slides::scene_graph::SlideNode::AutoShape(shape) => {
            assert_eq!(shape.name, "Q4 Strategy & Revenue Projections");
        }
        _ => panic!("Expected AutoShape node"),
    }
}
