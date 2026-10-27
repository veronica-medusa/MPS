use crate::ast::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use wps_core::Color;
use wps_ooxml::{
    ContentTypes, OoxmlError, OoxmlPackage, Relationships, CT_DOCX_MAIN, REL_OFFICE_DOCUMENT,
};

pub struct DocxCodec;

impl DocxCodec {
    pub fn export_docx(doc: &Document) -> Result<Vec<u8>, OoxmlError> {
        let mut pkg = OoxmlPackage::new();

        // 1. [Content_Types].xml
        let mut ct = ContentTypes::new();
        ct.overrides
            .insert("/word/document.xml".to_string(), CT_DOCX_MAIN.to_string());
        pkg.set_part_string("[Content_Types].xml", &ct.to_xml());

        // 2. _rels/.rels
        let mut root_rels = Relationships::new();
        root_rels.add("rId1", REL_OFFICE_DOCUMENT, "word/document.xml");
        pkg.set_part_string("_rels/.rels", &root_rels.to_xml());

        // 3. word/_rels/document.xml.rels
        let doc_rels = Relationships::new();
        pkg.set_part_string("word/_rels/document.xml.rels", &doc_rels.to_xml());

        // 4. word/document.xml
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>");

        for section in &doc.sections {
            for elem in &section.elements {
                match elem {
                    BlockElement::Paragraph(p) => {
                        xml.push_str("<w:p>");
                        if !p.style_name.is_empty() && p.style_name != "Normal" {
                            xml.push_str("<w:pPr><w:pStyle w:val=\"");
                            xml.push_str(&quick_xml::escape::escape(&p.style_name));
                            xml.push_str("\"/></w:pPr>");
                        }
                        for inline in &p.inlines {
                            match inline {
                                InlineElement::Run(r) => {
                                    xml.push_str("<w:r><w:rPr>");
                                    if r.bold {
                                        xml.push_str("<w:b/>");
                                    }
                                    if r.italic {
                                        xml.push_str("<w:i/>");
                                    }
                                    if r.underline {
                                        xml.push_str("<w:u w:val=\"single\"/>");
                                    }
                                    if r.strike {
                                        xml.push_str("<w:strike/>");
                                    }
                                    xml.push_str(&format!("<w:sz w:val=\"{}\"/>", (r.size_pt * 2.0).round() as i64));
                                    xml.push_str(&format!("<w:color w:val=\"{}\"/>", r.color.to_hex().trim_start_matches('#')));
                                    xml.push_str("</w:rPr><w:t>");
                                    xml.push_str(&quick_xml::escape::escape(&r.text));
                                    xml.push_str("</w:t></w:r>");
                                }
                                InlineElement::Field(f) => {
                                    xml.push_str("<w:fldSimple w:instr=\"");
                                    match &f.field_type {
                                        FieldType::Page => xml.push_str("PAGE"),
                                        FieldType::NumPages => xml.push_str("NUMPAGES"),
                                        FieldType::Date => xml.push_str("DATE"),
                                        FieldType::MergeField(name) => {
                                            xml.push_str(&format!("MERGEFIELD {}", quick_xml::escape::escape(name)))
                                        }
                                        _ => xml.push_str("FIELD"),
                                    }
                                    xml.push_str("\"><w:r><w:t>");
                                    xml.push_str(&quick_xml::escape::escape(&f.cached_value));
                                    xml.push_str("</w:t></w:r></w:fldSimple>");
                                }
                                _ => {}
                            }
                        }
                        xml.push_str("</w:p>");
                    }
                    BlockElement::Table(tbl) => {
                        xml.push_str("<w:tbl>");
                        for row in &tbl.rows {
                            xml.push_str("<w:tr>");
                            for cell in &row.cells {
                                xml.push_str("<w:tc>");
                                for cell_elem in &cell.elements {
                                    if let BlockElement::Paragraph(p) = cell_elem {
                                        xml.push_str("<w:p><w:r><w:t>");
                                        xml.push_str(&quick_xml::escape::escape(&p.plain_text()));
                                        xml.push_str("</w:t></w:r></w:p>");
                                    }
                                }
                                if cell.elements.is_empty() {
                                    xml.push_str("<w:p/>");
                                }
                                xml.push_str("</w:tc>");
                            }
                            xml.push_str("</w:tr>");
                        }
                        xml.push_str("</w:tbl>");
                    }
                    _ => {}
                }
            }
        }

        xml.push_str("</w:body></w:document>");
        pkg.set_part_string("word/document.xml", &xml);

        pkg.to_bytes()
    }

    pub fn import_docx(bytes: &[u8]) -> Result<Document, OoxmlError> {
        let pkg = OoxmlPackage::from_bytes(bytes)?;
        let xml_str = pkg.get_part_string("word/document.xml")?;

        let mut doc = Document::new("Imported Document", "Unknown Author");
        let mut reader = Reader::from_str(&xml_str);
        reader.config_mut().trim_text(false);

        let mut in_t = false;
        let mut current_p = Paragraph::new("p0");
        let mut current_r = TextRun::default();
        let mut current_text = String::new();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                    b"p" => {
                        current_p = Paragraph::new(&format!("p-{}", doc.sections[0].elements.len() + 1));
                    }
                    b"r" => {
                        current_r = TextRun::default();
                    }
                    b"t" => {
                        in_t = true;
                        current_text.clear();
                    }
                    _ => {}
                },
                Ok(Event::Empty(ref e)) => match e.local_name().as_ref() {
                    b"b" => current_r.bold = true,
                    b"i" => current_r.italic = true,
                    b"u" => current_r.underline = true,
                    b"strike" => current_r.strike = true,
                    b"sz" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"val" || attr.key.as_ref() == b"w:val" {
                                if let Ok(val_str) = std::str::from_utf8(&attr.value) {
                                    if let Ok(half_pt) = val_str.parse::<f64>() {
                                        current_r.size_pt = half_pt / 2.0;
                                    }
                                }
                            }
                        }
                    }
                    b"color" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"val" || attr.key.as_ref() == b"w:val" {
                                if let Ok(val_str) = std::str::from_utf8(&attr.value) {
                                    if let Ok(color) = Color::from_hex(val_str) {
                                        current_r.color = color;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                },
                Ok(Event::Text(ref e)) => {
                    if in_t {
                        current_text.push_str(&e.unescape().map_err(|err| OoxmlError::Xml(err.to_string()))?);
                    }
                }
                Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                    b"t" => {
                        in_t = false;
                        current_r.text = current_text.clone();
                    }
                    b"r" => {
                        if !current_r.text.is_empty() {
                            current_p.add_run(current_r.clone());
                        }
                    }
                    b"p" => {
                        doc.add_paragraph(current_p.clone());
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docx_roundtrip() {
        let mut original = Document::new("Report", "Author");
        let mut p1 = Paragraph::new("p1");
        p1.add_run(TextRun::new("Executive Summary").bold().with_size(16.0));
        original.add_paragraph(p1);

        let mut p2 = Paragraph::new("p2");
        p2.add_run(TextRun::new("OpenWPS brings high-craft productivity.").italic());
        original.add_paragraph(p2);

        let bytes = DocxCodec::export_docx(&original).unwrap();
        assert!(!bytes.is_empty());

        let imported = DocxCodec::import_docx(&bytes).unwrap();
        assert_eq!(imported.sections[0].elements.len(), 2);

        let BlockElement::Paragraph(imp_p1) = &imported.sections[0].elements[0] else { panic!() };
        assert_eq!(imp_p1.plain_text(), "Executive Summary");

        let BlockElement::Paragraph(imp_p2) = &imported.sections[0].elements[1] else { panic!() };
        assert_eq!(imp_p2.plain_text(), "OpenWPS brings high-craft productivity.");
    }
}
