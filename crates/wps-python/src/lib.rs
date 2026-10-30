pub mod dataframe;
pub mod dataframe_bridge;
pub mod engine;
pub mod macro_engine;
pub mod macro_runner;
pub mod plot;
pub mod sandbox;

pub use dataframe::{PyDataFrame, PySeries};
pub use dataframe_bridge::DataFrameBridge;
pub use engine::{NativePythonEngine, PyError, PyOutput, PythonEngine};
pub use macro_engine::MacroEngine;
pub use macro_runner::{MacroExecutionLog, MacroRunner, OfficeContext};
pub use plot::{MatplotlibPlotter, SvgPlot};
pub use sandbox::{
    PyCellExecRequest, PyEvalOutput, PyMacroReport, PyRuntimeError, PyVariableSummary,
    PythonResourceGovernor, PythonSandbox, WorkspaceMacroContext,
};

#[cfg(test)]
mod tests {
    use super::*;
    use wps_sheets::cell::{CellCoordinate, CellRange, CellValue};
    use wps_sheets::sparse_matrix::SparseMatrix;

    #[test]
    fn test_python_dataframe_creation_and_describe() {
        let mut matrix = SparseMatrix::new();
        // Setup table:
        // A1: "Sales", B1: "Cost"
        // A2: 100,     B2: 40
        // A3: 200,     B3: 80
        // A4: 300,     B4: 120
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Sales".to_string()));
        matrix.set_value(CellCoordinate::new(0, 1), CellValue::Text("Cost".to_string()));

        matrix.set_value(CellCoordinate::new(1, 0), CellValue::Number(100.0));
        matrix.set_value(CellCoordinate::new(1, 1), CellValue::Number(40.0));

        matrix.set_value(CellCoordinate::new(2, 0), CellValue::Number(200.0));
        matrix.set_value(CellCoordinate::new(2, 1), CellValue::Number(80.0));

        matrix.set_value(CellCoordinate::new(3, 0), CellValue::Number(300.0));
        matrix.set_value(CellCoordinate::new(3, 1), CellValue::Number(120.0));

        let range = CellRange::new(CellCoordinate::new(0, 0), CellCoordinate::new(3, 1));
        let df = PyDataFrame::from_sheet_range(&matrix, range);

        assert_eq!(df.num_cols(), 2);
        assert_eq!(df.num_rows(), 3);

        let sales = df.get_column("Sales").unwrap();
        assert_eq!(sales.mean(), Some(200.0));
        assert_eq!(sales.min(), Some(100.0));
        assert_eq!(sales.max(), Some(300.0));

        let desc = df.describe();
        assert_eq!(desc.num_cols(), 3); // metric + Sales + Cost
        assert_eq!(desc.num_rows(), 8); // count, mean, std, min, 25%, 50%, 75%, max
    }

    #[test]
    fn test_python_cell_eval_engine() {
        let mut matrix = SparseMatrix::new();
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Revenue".to_string()));
        matrix.set_value(CellCoordinate::new(1, 0), CellValue::Number(500.0));
        matrix.set_value(CellCoordinate::new(2, 0), CellValue::Number(1500.0));

        let mut engine = NativePythonEngine::new();
        let script = r#"
            df = xl("A1:A3")
            df['Revenue'].sum()
        "#;

        let out = engine.eval_cell(script, &matrix).expect("Execution succeeds");
        assert_eq!(out, PyOutput::Scalar(CellValue::Number(2000.0)));
    }

    #[test]
    fn test_python_plot_spilling() {
        let mut matrix = SparseMatrix::new();
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Month".to_string()));
        matrix.set_value(CellCoordinate::new(0, 1), CellValue::Text("Growth".to_string()));

        matrix.set_value(CellCoordinate::new(1, 0), CellValue::Number(1.0));
        matrix.set_value(CellCoordinate::new(1, 1), CellValue::Number(10.0));

        matrix.set_value(CellCoordinate::new(2, 0), CellValue::Number(2.0));
        matrix.set_value(CellCoordinate::new(2, 1), CellValue::Number(25.0));

        let mut engine = NativePythonEngine::new();
        let script = r#"
            df = xl("A1:B3")
            plt.plot(df['Month'], df['Growth'])
        "#;

        let out = engine.eval_cell(script, &matrix).expect("Plot generated");
        match out {
            PyOutput::Plot(plot) => {
                assert!(plot.svg_xml.contains("<svg"));
                assert!(plot.svg_xml.contains("stroke=\"#5266eb\""));
            }
            _ => panic!("Expected SVG plot output"),
        }
    }

    #[test]
    fn test_cross_module_macro_automation() {
        let mut writer_doc = wps_writer::ast::Document::new("Doc", "Author");
        let mut sheets_matrix = SparseMatrix::new();
        let mut slides_deck = wps_slides::scene_graph::PresentationDeck::new("Pitch");
        let mut pdf_doc = wps_pdf::PdfDocument::new("Manual");

        let mut ctx = OfficeContext {
            writer: Some(&mut writer_doc),
            sheets: Some(&mut sheets_matrix),
            slides: Some(&mut slides_deck),
            pdf: Some(&mut pdf_doc),
        };

        let macro_script = r#"
            sheets.set("B2", 999)
            writer.add_heading("Financial Summary", 1)
            writer.add_paragraph("Fiscal results exceeded expectations.")
            slides.add_slide("Q4 Highlights")
        "#;

        let log = MacroRunner::run_macro(macro_script, &mut ctx).expect("Macro ran");
        assert_eq!(log.actions_performed, 4);

        // Verify Sheets mutated
        let b2 = sheets_matrix.get_cell(CellCoordinate::new(1, 1));
        assert!(b2.is_some());
        assert_eq!(b2.unwrap().value, CellValue::Number(999.0));

        // Verify Writer mutated
        assert!(writer_doc.sections[0].elements.len() >= 2);

        // Verify Slides mutated
        assert_eq!(slides_deck.slides.len(), 2);
    }
}
