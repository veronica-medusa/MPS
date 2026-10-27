use crate::cell::{CellCoordinate, CellValue};
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
    pub fn export_xlsx(matrix: &SparseMatrix, max_rows: u32, max_cols: u16) -> Result<Vec<u8>, OoxmlError> {
        let mut pkg = OoxmlPackage::new();
        let mut sst = SharedStringsTable::new();

        // 1. [Content_Types].xml
        let mut ct = ContentTypes::new();
        ct.overrides.insert("/xl/workbook.xml".to_string(), CT_XLSX_WORKBOOK.to_string());
        ct.overrides.insert("/xl/worksheets/sheet1.xml".to_string(), CT_XLSX_WORKSHEET.to_string());
        ct.overrides.insert("/xl/sharedStrings.xml".to_string(), CT_XLSX_SHARED_STRINGS.to_string());
        ct.overrides.insert("/xl/styles.xml".to_string(), CT_XLSX_STYLES.to_string());
        pkg.set_part_string("[Content_Types].xml", &ct.to_xml());

        // 2. _rels/.rels
        let mut root_rels = Relationships::new();
        root_rels.add("rId1", REL_OFFICE_DOCUMENT, "xl/workbook.xml");
        pkg.set_part_string("_rels/.rels", &root_rels.to_xml());

        // 3. xl/_rels/workbook.xml.rels
        let mut wb_rels = Relationships::new();
        wb_rels.add("rId1", REL_WORKSHEET, "worksheets/sheet1.xml");
        wb_rels.add("rId2", REL_SHARED_STRINGS, "sharedStrings.xml");
        wb_rels.add("rId3", REL_STYLES, "styles.xml");
        pkg.set_part_string("xl/_rels/workbook.xml.rels", &wb_rels.to_xml());

        // 4. xl/workbook.xml
        let wb_xml = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets><sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"rId1\"/></sheets></workbook>";
        pkg.set_part_string("xl/workbook.xml", wb_xml);

        // 5. xl/styles.xml
        let styles = StylesSheet::new();
        pkg.set_part_string("xl/styles.xml", &styles.to_spreadsheet_styles_xml());

        // 6. Build sheetData & collect shared strings
        let mut sheet_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\"><sheetData>");

        for r in 0..=max_rows {
            let mut row_cells_xml = String::new();
            for c in 0..=max_cols {
                let coord = CellCoordinate::new(r, c);
                if let Some(cell) = matrix.get_cell(coord) {
                    if !cell.value.is_empty() || cell.formula.is_some() {
                        let a1 = coord.to_a1_string();
                        row_cells_xml.push_str(&format!("<c r=\"{}\"", a1));

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

            if !row_cells_xml.is_empty() {
                sheet_xml.push_str(&format!("<row r=\"{}\">", r + 1));
                sheet_xml.push_str(&row_cells_xml);
                sheet_xml.push_str("</row>");
            }
        }

        sheet_xml.push_str("</sheetData></worksheet>");
        pkg.set_part_string("xl/worksheets/sheet1.xml", &sheet_xml);

        // 7. xl/sharedStrings.xml
        pkg.set_part_string("xl/sharedStrings.xml", &sst.to_xml());

        pkg.to_bytes()
    }

    pub fn import_xlsx(bytes: &[u8]) -> Result<SparseMatrix, OoxmlError> {
        let pkg = OoxmlPackage::from_bytes(bytes)?;

        // Parse SST if available
        let sst = if pkg.has_part("xl/sharedStrings.xml") {
            let sst_xml = pkg.get_part_string("xl/sharedStrings.xml")?;
            SharedStringsTable::from_xml(&sst_xml)?
        } else {
            SharedStringsTable::new()
        };

        // Parse sheet1.xml
        let sheet_xml = pkg.get_part_string("xl/worksheets/sheet1.xml")?;
        let mut matrix = SparseMatrix::new();

        let mut reader = Reader::from_str(&sheet_xml);
        reader.config_mut().trim_text(true);

        let mut current_coord = CellCoordinate::new(0, 0);
        let mut current_type = String::new();
        let mut in_v = false;
        let mut current_val_text = String::new();

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
                    _ => {}
                },
                Ok(Event::Text(ref e)) => {
                    if in_v {
                        current_val_text.push_str(&e.unescape().map_err(|err| OoxmlError::Xml(err.to_string()))?);
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
                        matrix.set_value(current_coord, cell_val);
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(matrix)
    }

    pub fn export_csv(matrix: &SparseMatrix, max_rows: u32, max_cols: u16) -> String {
        let mut csv = String::new();
        for r in 0..=max_rows {
            let mut row_vals = Vec::new();
            for c in 0..=max_cols {
                let val = matrix.get_cell(CellCoordinate::new(r, c)).map(|cell| cell.value.as_string()).unwrap_or_default();
                if val.contains(',') || val.contains('"') || val.contains('\n') {
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
    fn test_csv_export() {
        let mut matrix = SparseMatrix::new();
        matrix.set_value(CellCoordinate::new(0, 0), CellValue::Text("Item, Name".into()));
        matrix.set_value(CellCoordinate::new(0, 1), CellValue::Number(42.0));

        let csv = XlsxCodec::export_csv(&matrix, 0, 1);
        assert!(csv.contains("\"Item, Name\",42"));
    }
}
