use crate::cell::{CellCoordinate, CellRange, CellValue};
use crate::model::{Workbook, Worksheet};
use crate::sparse_matrix::SparseMatrix;
use quick_xml::events::Event;
use quick_xml::Reader;
use wps_ooxml::{
    ContentTypes, OoxmlError, OoxmlPackage, Relationships, SharedStringsTable, StylesSheet,
    CT_XLSX_SHARED_STRINGS, CT_XLSX_STYLES, CT_XLSX_WORKBOOK, CT_XLSX_WORKSHEET,
    REL_OFFICE_DOCUMENT, REL_SHARED_STRINGS, REL_STYLES, REL_WORKSHEET,
};

pub struct XlsxCodec;

impl XlsxCodec {
    /// Exports a single SparseMatrix into an XLSX package
    pub fn export_xlsx(matrix: &SparseMatrix, max_rows: u32, max_cols: u16) -> Result<Vec<u8>, OoxmlError> {
        let mut wb = Workbook::new();
        wb.sheets[0].matrix = matrix.clone();
        Self::export_workbook_xlsx_with_bounds(&wb, Some((max_rows, max_cols)))
    }

    /// Exports a full multi-sheet Workbook into an XLSX package
    pub fn export_workbook_xlsx(wb: &Workbook) -> Result<Vec<u8>, OoxmlError> {
        Self::export_workbook_xlsx_with_bounds(wb, None)
    }

    fn export_workbook_xlsx_with_bounds(
        wb: &Workbook,
        bounds: Option<(u32, u16)>,
    ) -> Result<Vec<u8>, OoxmlError> {
        let mut pkg = OoxmlPackage::new();
        let mut sst = SharedStringsTable::new();

        // 1. [Content_Types].xml
        let mut ct = ContentTypes::new();
        ct.overrides.insert("/xl/workbook.xml".to_string(), CT_XLSX_WORKBOOK.to_string());
        ct.overrides.insert("/xl/sharedStrings.xml".to_string(), CT_XLSX_SHARED_STRINGS.to_string());
        ct.overrides.insert("/xl/styles.xml".to_string(), CT_XLSX_STYLES.to_string());

        for (idx, _) in wb.sheets.iter().enumerate() {
            let part_name = format!("/xl/worksheets/sheet{}.xml", idx + 1);
            ct.overrides.insert(part_name, CT_XLSX_WORKSHEET.to_string());
        }
        pkg.set_part_string("[Content_Types].xml", &ct.to_xml());

        // 2. _rels/.rels
        let mut root_rels = Relationships::new();
        root_rels.add("rId1", REL_OFFICE_DOCUMENT, "xl/workbook.xml");
        pkg.set_part_string("_rels/.rels", &root_rels.to_xml());

        // 3. xl/_rels/workbook.xml.rels
        let mut wb_rels = Relationships::new();
        for (idx, _) in wb.sheets.iter().enumerate() {
            let rid = format!("rId{}", idx + 1);
            let target = format!("worksheets/sheet{}.xml", idx + 1);
            wb_rels.add(&rid, REL_WORKSHEET, &target);
        }
        let sst_rid = format!("rId{}", wb.sheets.len() + 1);
        wb_rels.add(&sst_rid, REL_SHARED_STRINGS, "sharedStrings.xml");
        let styles_rid = format!("rId{}", wb.sheets.len() + 2);
        wb_rels.add(&styles_rid, REL_STYLES, "styles.xml");
        pkg.set_part_string("xl/_rels/workbook.xml.rels", &wb_rels.to_xml());

        // 4. xl/workbook.xml
        let mut wb_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets>");
        for (idx, ws) in wb.sheets.iter().enumerate() {
            let rid = format!("rId{}", idx + 1);
            wb_xml.push_str(&format!(
                "<sheet name=\"{}\" sheetId=\"{}\" r:id=\"{}\"/>",
                ws.name,
                idx + 1,
                rid
            ));
        }
        wb_xml.push_str("</sheets></workbook>");
        pkg.set_part_string("xl/workbook.xml", &wb_xml);

        // 5. xl/styles.xml
        let styles = StylesSheet::new();
        pkg.set_part_string("xl/styles.xml", &styles.to_spreadsheet_styles_xml());

        // 6. Worksheets
        for (idx, ws) in wb.sheets.iter().enumerate() {
            let max_r = bounds.map(|b| b.0).unwrap_or_else(|| ws.matrix.max_populated_row().max(50));
            let max_c = bounds.map(|b| b.1).unwrap_or_else(|| ws.matrix.max_populated_col().max(20));

            let mut sheet_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">");

            // Freeze panes if present
            if let Some(fp) = ws.freeze_pane {
                sheet_xml.push_str(&format!(
                    "<sheetViews><sheetView tabSelected=\"1\" workbookViewId=\"0\"><pane ySplit=\"{}\" xSplit=\"{}\" topLeftCell=\"{}\" activePane=\"bottomRight\" state=\"frozen\"/></sheetView></sheetViews>",
                    fp.split_row, fp.split_col, fp.top_left_cell.to_a1_string()
                ));
            }

            sheet_xml.push_str("<sheetData>");

            for r in 0..=max_r {
                let mut row_cells_xml = String::new();
                for c in 0..=max_c {
                    let coord = CellCoordinate::new(r, c);
                    if let Some(cell) = ws.matrix.get_cell(coord) {
                        if !cell.value.is_empty() || cell.formula.is_some() {
                            let a1 = coord.to_a1_string();
                            row_cells_xml.push_str(&format!("<c r=\"{}\"", a1));

                            if let Some(ref form) = cell.formula {
                                row_cells_xml.push_str("><f>");
                                row_cells_xml.push_str(&quick_xml::escape::escape(form));
                                row_cells_xml.push_str("</f><v>");
                                row_cells_xml.push_str(&cell.value.as_string());
                                row_cells_xml.push_str("</v></c>");
                            } else {
                                match &cell.value {
                                    CellValue::Number(n) => {
                                        row_cells_xml.push_str("><v>");
                                        row_cells_xml.push_str(&n.to_string());
                                        row_cells_xml.push_str("</v></c>");
                                    }
                                    CellValue::Text(t) => {
                                        let sst_idx = sst.get_or_insert(t);
                                        row_cells_xml.push_str(" t=\"s\"><v>");
                                        row_cells_xml.push_str(&sst_idx.to_string());
                                        row_cells_xml.push_str("</v></c>");
                                    }
                                    CellValue::Boolean(b) => {
                                        row_cells_xml.push_str(" t=\"b\"><v>");
                                        row_cells_xml.push_str(if *b { "1" } else { "0" });
                                        row_cells_xml.push_str("</v></c>");
                                    }
                                    _ => {
                                        row_cells_xml.push_str("/>");
                                    }
                                }
                            }
                        }
                    }
                }

                if !row_cells_xml.is_empty() {
                    sheet_xml.push_str(&format!("<row r=\"{}\">", r + 1));
                    sheet_xml.push_str(&row_cells_xml);
                    sheet_xml.push_str("</row>");
                }
            }

            sheet_xml.push_str("</sheetData>");

            // Merged regions
            if !ws.merged_regions.is_empty() {
                sheet_xml.push_str(&format!("<mergeCells count=\"{}\">", ws.merged_regions.len()));
                for m in &ws.merged_regions {
                    sheet_xml.push_str(&format!(
                        "<mergeCell ref=\"{}:{}\"/>",
                        m.start.to_a1_string(),
                        m.end.to_a1_string()
                    ));
                }
                sheet_xml.push_str("</mergeCells>");
            }

            sheet_xml.push_str("</worksheet>");
            let part_name = format!("xl/worksheets/sheet{}.xml", idx + 1);
            pkg.set_part_string(&part_name, &sheet_xml);
        }

        // 7. xl/sharedStrings.xml
        pkg.set_part_string("xl/sharedStrings.xml", &sst.to_xml());

        pkg.to_bytes()
    }

    /// Imports a single SparseMatrix from sheet1 of an XLSX package
    pub fn import_xlsx(bytes: &[u8]) -> Result<SparseMatrix, OoxmlError> {
        let wb = Self::import_workbook_xlsx(bytes)?;
        Ok(wb.sheets.into_iter().next().map(|s| s.matrix).unwrap_or_default())
    }

    /// Imports a full multi-sheet Workbook from an XLSX package
    pub fn import_workbook_xlsx(bytes: &[u8]) -> Result<Workbook, OoxmlError> {
        let pkg = OoxmlPackage::from_bytes(bytes)?;

        // Parse SST if available
        let sst = if pkg.has_part("xl/sharedStrings.xml") {
            let sst_xml = pkg.get_part_string("xl/sharedStrings.xml")?;
            SharedStringsTable::from_xml(&sst_xml)?
        } else {
            SharedStringsTable::new()
        };

        // Parse sheets from xl/workbook.xml
        let mut sheet_infos = Vec::new();
        if pkg.has_part("xl/workbook.xml") {
            let wb_xml = pkg.get_part_string("xl/workbook.xml")?;
            let mut reader = Reader::from_str(&wb_xml);
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) if e.local_name().as_ref() == b"sheet" => {
                        let mut name = String::new();
                        let mut sheet_id = 1;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"name" {
                                if let Ok(s) = std::str::from_utf8(&attr.value) {
                                    name = s.to_string();
                                }
                            } else if attr.key.as_ref() == b"sheetId" {
                                if let Ok(s) = std::str::from_utf8(&attr.value) {
                                    sheet_id = s.parse::<usize>().unwrap_or(1);
                                }
                            }
                        }
                        if !name.is_empty() {
                            sheet_infos.push((sheet_id, name));
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                    _ => {}
                }
                buf.clear();
            }
        }

        if sheet_infos.is_empty() {
            sheet_infos.push((1, "Sheet1".to_string()));
        }

        let mut sheets = Vec::new();

        for (idx, (sheet_id, name)) in sheet_infos.into_iter().enumerate() {
            let part_name = format!("xl/worksheets/sheet{}.xml", idx + 1);
            if !pkg.has_part(&part_name) {
                continue;
            }

            let sheet_xml = pkg.get_part_string(&part_name)?;
            let mut ws = Worksheet::new(&format!("sheet-{}", sheet_id), &name);

            let mut reader = Reader::from_str(&sheet_xml);
            reader.config_mut().trim_text(true);

            let mut current_coord = CellCoordinate::new(0, 0);
            let mut current_type = String::new();
            let mut in_v = false;
            let mut in_f = false;
            let mut current_val_text = String::new();
            let mut current_formula_text = String::new();
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                        b"c" => {
                            current_type.clear();
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"r" {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Some(coord) = CellCoordinate::parse(s) {
                                            current_coord = coord;
                                        }
                                    }
                                } else if attr.key.as_ref() == b"t" {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        current_type = s.to_string();
                                    }
                                }
                            }
                        }
                        b"v" => {
                            in_v = true;
                            current_val_text.clear();
                        }
                        b"f" => {
                            in_f = true;
                            current_formula_text.clear();
                        }
                        _ => {}
                    },
                    Ok(Event::Empty(ref e)) => {
                        if e.local_name().as_ref() == b"mergeCell" {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"ref" {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Some(range) = CellRange::parse(s) {
                                            ws.merge_range(range);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(Event::Text(ref e)) => {
                        if in_v {
                            current_val_text.push_str(&e.unescape().map_err(|err| OoxmlError::Xml(err.to_string()))?);
                        } else if in_f {
                            current_formula_text.push_str(&e.unescape().map_err(|err| OoxmlError::Xml(err.to_string()))?);
                        }
                    }
                    Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                        b"v" => {
                            in_v = false;
                            let cell_val = match current_type.as_str() {
                                "s" => {
                                    if let Ok(idx) = current_val_text.parse::<usize>() {
                                        sst.get(idx).map(|s| CellValue::Text(s.to_string())).unwrap_or_default()
                                    } else {
                                        CellValue::Empty
                                    }
                                }
                                "b" => CellValue::Boolean(current_val_text == "1"),
                                _ => {
                                    if let Ok(num) = current_val_text.parse::<f64>() {
                                        CellValue::Number(num)
                                    } else {
                                        CellValue::Text(current_val_text.clone())
                                    }
                                }
                            };
                            ws.matrix.set_value(current_coord, cell_val);
                        }
                        b"f" => {
                            in_f = false;
                            if !current_formula_text.is_empty() {
                                ws.matrix.set_formula(current_coord, Some(current_formula_text.clone()));
                            }
                        }
                        _ => {}
                    },
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                    _ => {}
                }
                buf.clear();
            }

            sheets.push(ws);
        }

        if sheets.is_empty() {
            sheets.push(Worksheet::new("sheet-1", "Sheet1"));
        }

        Ok(Workbook {
            sheets,
            active_sheet: 0,
            named_ranges: std::collections::BTreeMap::new(),
            structured_tables: std::collections::BTreeMap::new(),
            calc_mode: crate::model::CalcMode::Automatic,
        })
    }

    /// RFC 4180 streaming CSV exporter
    pub fn export_csv(matrix: &SparseMatrix, max_rows: u32, max_cols: u16) -> String {
        let mut csv = String::new();
        for r in 0..=max_rows {
            let mut row_vals = Vec::new();
            for c in 0..=max_cols {
                let val = matrix
                    .get_cell(CellCoordinate::new(r, c))
                    .map(|cell| cell.value.as_string())
                    .unwrap_or_default();
                if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
                    row_vals.push(format!("\"{}\"", val.replace('"', "\"\"")));
                } else {
                    row_vals.push(val);
                }
            }
            csv.push_str(&row_vals.join(","));
            csv.push('\n');
        }
        csv
    }

    /// RFC 4180 streaming CSV parser with quote escaping and auto type detection
    pub fn import_csv(csv_str: &str) -> SparseMatrix {
        let mut matrix = SparseMatrix::new();
        let mut row: u32 = 0;
        let mut col: u16 = 0;
        let mut field = String::new();
        let mut in_quotes = false;
        let mut chars = csv_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if in_quotes {
                if ch == '"' {
                    if let Some(&'"') = chars.peek() {
                        chars.next();
                        field.push('"');
                    } else {
                        in_quotes = false;
                    }
                } else {
                    field.push(ch);
                }
            } else {
                match ch {
                    '"' => {
                        in_quotes = true;
                    }
                    ',' => {
                        Self::set_parsed_csv_cell(&mut matrix, row, col, &field);
                        field.clear();
                        col += 1;
                    }
                    '\r' => {
                        if let Some(&'\n') = chars.peek() {
                            chars.next();
                        }
                        Self::set_parsed_csv_cell(&mut matrix, row, col, &field);
                        field.clear();
                        row += 1;
                        col = 0;
                    }
                    '\n' => {
                        Self::set_parsed_csv_cell(&mut matrix, row, col, &field);
                        field.clear();
                        row += 1;
                        col = 0;
                    }
                    _ => {
                        field.push(ch);
                    }
                }
            }
        }

        if !field.is_empty() || col > 0 {
            Self::set_parsed_csv_cell(&mut matrix, row, col, &field);
        }

        matrix
    }

    fn set_parsed_csv_cell(matrix: &mut SparseMatrix, row: u32, col: u16, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        let cell_val = if let Ok(n) = trimmed.parse::<f64>() {
            CellValue::Number(n)
        } else if trimmed.eq_ignore_ascii_case("true") {
            CellValue::Boolean(true)
        } else if trimmed.eq_ignore_ascii_case("false") {
            CellValue::Boolean(false)
        } else {
            CellValue::Text(text.to_string())
        };
        matrix.set_value(CellCoordinate::new(row, col), cell_val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xlsx_roundtrip() {
        let mut matrix = SparseMatrix::new();
        let c1 = CellCoordinate::new(0, 0); // A1
        let c2 = CellCoordinate::new(0, 1); // B1
        let c3 = CellCoordinate::new(1, 1); // B2

        matrix.set_value(c1, CellValue::Text("Quarter".into()));
        matrix.set_value(c2, CellValue::Text("Revenue".into()));
        matrix.set_value(c3, CellValue::Number(250000.0));

        let bytes = XlsxCodec::export_xlsx(&matrix, 2, 2).unwrap();
        assert!(!bytes.is_empty());

        let imported = XlsxCodec::import_xlsx(&bytes).unwrap();
        assert_eq!(imported.get_cell(c1).unwrap().value, CellValue::Text("Quarter".into()));
        assert_eq!(imported.get_cell(c2).unwrap().value, CellValue::Text("Revenue".into()));
        assert_eq!(imported.get_cell(c3).unwrap().value, CellValue::Number(250000.0));
    }

    #[test]
    fn test_workbook_xlsx_multi_sheet_roundtrip() {
        let mut wb = Workbook::new();
        wb.sheets[0].name = "Summary".to_string();
        wb.sheets[0].set_cell_value(CellCoordinate::new(0, 0), CellValue::Text("Gross Total".into()));
        wb.sheets[0].set_cell_value(CellCoordinate::new(0, 1), CellValue::Number(9999.0));

        let s2_idx = wb.add_sheet("Details");
        wb.sheets[s2_idx].set_cell_value(CellCoordinate::new(0, 0), CellValue::Text("Sub-Item A".into()));
        wb.sheets[s2_idx].set_cell_value(CellCoordinate::new(0, 1), CellValue::Number(5000.0));

        let bytes = XlsxCodec::export_workbook_xlsx(&wb).unwrap();
        let imported_wb = XlsxCodec::import_workbook_xlsx(&bytes).unwrap();

        assert_eq!(imported_wb.sheets.len(), 2);
        assert_eq!(imported_wb.sheets[0].name, "Summary");
        assert_eq!(imported_wb.sheets[0].get_cell_value(CellCoordinate::new(0, 0)), CellValue::Text("Gross Total".into()));
        assert_eq!(imported_wb.sheets[1].name, "Details");
        assert_eq!(imported_wb.sheets[1].get_cell_value(CellCoordinate::new(0, 0)), CellValue::Text("Sub-Item A".into()));
    }

    #[test]
    fn test_csv_export_and_import() {
        let mut matrix = SparseMatrix::new();
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Item, Name".into()));
        matrix.set_value(CellCoordinate::new(0, 1), CellValue::Number(42.0));
        matrix.set_value(CellCoordinate::new(1, 0), CellValue::Text("Status \"Active\"".into()));
        matrix.set_value(CellCoordinate::new(1, 1), CellValue::Boolean(true));

        let csv = XlsxCodec::export_csv(&matrix, 1, 1);
        assert!(csv.contains("\"Item, Name\",42"));
        assert!(csv.contains("\"Status \"\"Active\"\"\",TRUE"));

        let imported = XlsxCodec::import_csv(&csv);
        assert_eq!(imported.get_cell(CellCoordinate::new(0, 0)).unwrap().value, CellValue::Text("Item, Name".into()));
        assert_eq!(imported.get_cell(CellCoordinate::new(0, 1)).unwrap().value, CellValue::Number(42.0));
        assert_eq!(imported.get_cell(CellCoordinate::new(1, 0)).unwrap().value, CellValue::Text("Status \"Active\"".into()));
        assert_eq!(imported.get_cell(CellCoordinate::new(1, 1)).unwrap().value, CellValue::Boolean(true));
    }
}
