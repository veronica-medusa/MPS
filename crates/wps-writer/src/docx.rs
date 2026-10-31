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
        ct.overrides.insert("/word/document.xml".to_string(), CT_DOCX_MAIN.to_string());
        ct.overrides.insert("/word/styles.xml".to_string(), "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml".to_string());
        pkg.set_part_string("[Content_Types].xml", &ct.to_xml());

        // 2. _rels/.rels
        let mut root_rels = Relationships::new();
        root_rels.add("rId1", REL_OFFICE_DOCUMENT, "word/document.xml");
        pkg.set_part_string("_rels/.rels", &root_rels.to_xml());

        // 3. word/_rels/document.xml.rels
        let mut doc_rels = Relationships::new();
        doc_rels.add("rIdStyles", "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles", "styles.xml");
        pkg.set_part_string("word/_rels/document.xml.rels", &doc_rels.to_xml());

        // 4. word/styles.xml
        let mut styles_xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">");
        for (id, style) in &doc.styles.styles {
            styles_xml.push_str(&format!(
                "<w:style w:type=\"paragraph\" w:styleId=\"{}\"><w:name w:val=\"{}\"/>",
                quick_xml::escape::escape(id),
                quick_xml::escape::escape(&style.name)
            ));
            if let Some(ref based) = style.based_on {
                styles_xml.push_str(&format!("<w:basedOn w:val=\"{}\"/>", quick_xml::escape::escape(based)));
            }
            styles_xml.push_str("</w:style>");
        }
        styles_xml.push_str("</w:styles>");
        pkg.set_part_string("word/styles.xml", &styles_xml);

        // 5. word/document.xml
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>");

        for section in &doc.sections {
            for elem in &section.elements {
                match elem {
                    BlockNode::Paragraph(p) => {
                        xml.push_str("<w:p>");
                        xml.push_str("<w:pPr>");
                        if !p.style_id.is_empty() && p.style_id != "Normal" {
                            xml.push_str(&format!("<w:pStyle w:val=\"{}\"/>", quick_xml::escape::escape(&p.style_id)));
                        }
                        let align_str = match p.alignment {
                            TextAlignment::Center => "center",
                            TextAlignment::Right => "right",
                            TextAlignment::Justify => "both",
                            TextAlignment::Left => "left",
                        };
                        xml.push_str(&format!("<w:jc w:val=\"{}\"/>", align_str));
                        xml.push_str("</w:pPr>");

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
                                    xml.push_str("</w:rPr><w:t xml:space=\"preserve\">");
                                    xml.push_str(&quick_xml::escape::escape(&r.text));
                                    xml.push_str("</w:t></w:r>");
                                }
                                InlineElement::Field(f) => {
                                    xml.push_str("<w:fldSimple w:instr=\"");
                                    match &f.field_type {
                                        DynamicField::Page => xml.push_str("PAGE"),
                                        DynamicField::NumPages => xml.push_str("NUMPAGES"),
                                        DynamicField::Date => xml.push_str("DATE"),
                                        DynamicField::MergeField(name) => {
                                            xml.push_str(&format!("MERGEFIELD {}", quick_xml::escape::escape(name)))
                                        }
                                        _ => xml.push_str("FIELD"),
                                    }
                                    xml.push_str("\"><w:r><w:t xml:space=\"preserve\">");
                                    xml.push_str(&quick_xml::escape::escape(&f.cached_value));
                                    xml.push_str("</w:t></w:r></w:fldSimple>");
                                }
                                _ => {}
                            }
                        }
                        xml.push_str("</w:p>");
                    }
                    BlockNode::Table(tbl) => {
                        xml.push_str("<w:tbl>");
                        xml.push_str("<w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>");
                        for row in &tbl.rows {
                            xml.push_str("<w:tr>");
                            if row.is_header || row.is_header_repeat {
                                xml.push_str("<w:trPr><w:tblHeader/></w:trPr>");
                            }
                            for cell in &row.cells {
                                xml.push_str("<w:tc>");
                                if cell.grid_span > 1 {
                                    xml.push_str(&format!("<w:tcPr><w:gridSpan w:val=\"{}\"/></w:tcPr>", cell.grid_span));
                                }
                                for cell_elem in &cell.elements {
                                    if let BlockNode::Paragraph(p) = cell_elem {
                                        xml.push_str("<w:p><w:r><w:t xml:space=\"preserve\">");
                                        xml.push_str(&quick_xml::escape::escape(p.plain_text()));
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
                    BlockNode::FloatingObject(_) => {}
                }
            }

            // Section properties
            xml.push_str(&format!(
                "<w:sectPr><w:pgSz w:w=\"{}\" w:h=\"{}\"/><w:pgMar w:top=\"{}\" w:bottom=\"{}\" w:left=\"{}\" w:right=\"{}\"/></w:sectPr>",
                section.page_setup.width_twips,
                section.page_setup.height_twips,
                section.page_setup.margins.top_twips,
                section.page_setup.margins.bottom_twips,
                section.page_setup.margins.left_twips,
                section.page_setup.margins.right_twips
            ));
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
        let mut in_tbl = false;
        let mut _in_tr = false;
        let mut in_tc = false;

        let mut current_p = ParagraphNode::new("p0");
        let mut current_r = TextRun::default();
        let mut current_text = String::new();
        let mut current_tbl = TableNode::default();
        let mut current_row = TableRow::default();
        let mut current_cell = TableCell::default();

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                    b"tbl" => {
                        in_tbl = true;
                        current_tbl = TableNode::default();
                    }
                    b"tr" => {
                        _in_tr = true;
                        current_row = TableRow::default();
                    }
                    b"tc" => {
                        in_tc = true;
                        current_cell = TableCell::default();
                    }
                    b"p" => {
                        let id = format!("p-{}", doc.sections[0].elements.len() + 1);
                        current_p = ParagraphNode::new(&id);
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
                    b"tblHeader" => current_row.is_header_repeat = true,
                    b"pStyle" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"val" || attr.key.as_ref() == b"w:val" {
                                if let Ok(val_str) = std::str::from_utf8(&attr.value) {
                                    current_p.style_id = val_str.to_string();
                                }
                            }
                        }
                    }
                    b"gridSpan" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"val" || attr.key.as_ref() == b"w:val" {
                                if let Ok(val_str) = std::str::from_utf8(&attr.value) {
                                    if let Ok(span) = val_str.parse::<u16>() {
                                        current_cell.grid_span = span;
                                    }
                                }
                            }
                        }
                    }
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
                        if in_tc {
                            current_cell.elements.push(BlockNode::Paragraph(current_p.clone()));
                        } else if !in_tbl {
                            doc.add_paragraph(current_p.clone());
                        }
                    }
                    b"tc" => {
                        in_tc = false;
                        current_row.cells.push(current_cell.clone());
                    }
                    b"tr" => {
                        _in_tr = false;
                        current_tbl.rows.push(current_row.clone());
                    }
                    b"tbl" => {
                        in_tbl = false;
                        doc.sections[0].elements.push(BlockNode::Table(current_tbl.clone()));
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

    /// Imports Markdown text directly into Document AST
    pub fn import_markdown(md: &str) -> Document {
        let mut doc = Document::new("Markdown Document", "OpenWPS");
        for (idx, line) in md.lines().enumerate() {
            let id = format!("md-p-{}", idx + 1);
            let trimmed = line.trim();

            if let Some(h) = trimmed.strip_prefix("# ") {
                doc.add_paragraph(ParagraphNode::heading(&id, 1, h));
            } else if let Some(h) = trimmed.strip_prefix("## ") {
                doc.add_paragraph(ParagraphNode::heading(&id, 2, h));
            } else if let Some(h) = trimmed.strip_prefix("### ") {
                doc.add_paragraph(ParagraphNode::heading(&id, 3, h));
            } else if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
                let mut p = ParagraphNode::new(&id);
                p.style_id = "List Paragraph".to_string();
                p.indent_left_twips = Twips(360);
                p.add_run(TextRun::new(item));
                doc.add_paragraph(p);
            } else if !trimmed.is_empty() {
                let mut p = ParagraphNode::new(&id);
                p.add_run(TextRun::new(trimmed));
                doc.add_paragraph(p);
            }
        }
        doc
    }

    /// Exports Document AST to Markdown formatted string
    pub fn export_markdown(doc: &Document) -> String {
        let mut md = String::new();
        for sec in &doc.sections {
            for elem in &sec.elements {
                match elem {
                    BlockNode::Paragraph(p) => {
                        if p.style_id == "Heading 1" {
                            md.push_str(&format!("# {}\n\n", p.plain_text()));
                        } else if p.style_id == "Heading 2" {
                            md.push_str(&format!("## {}\n\n", p.plain_text()));
                        } else if p.style_id == "Heading 3" {
                            md.push_str(&format!("### {}\n\n", p.plain_text()));
                        } else if p.style_id == "List Paragraph" {
                            md.push_str(&format!("- {}\n", p.plain_text()));
                        } else {
                            md.push_str(&format!("{}\n\n", p.plain_text()));
                        }
                    }
                    BlockNode::Table(t) => {
                        for (r_idx, row) in t.rows.iter().enumerate() {
                            let cells: Vec<String> = row.cells.iter().map(|c| {
                                c.elements.iter().map(|e| e.plain_text()).collect::<Vec<_>>().join(" ")
                            }).collect();
                            md.push_str(&format!("| {} |\n", cells.join(" | ")));
                            if r_idx == 0 {
                                let sep = vec!["---"; row.cells.len()].join(" | ");
                                md.push_str(&format!("| {} |\n", sep));
                            }
                        }
                        md.push('\n');
                    }
                    _ => {}
                }
            }
        }
        md
    }

    /// Imports plain text into Document AST
    pub fn import_txt(txt: &str) -> Document {
        let mut doc = Document::new("Text Document", "OpenWPS");
        for (idx, line) in txt.lines().enumerate() {
            let id = format!("txt-p-{}", idx + 1);
            let mut p = ParagraphNode::new(&id);
            p.add_run(TextRun::new(line));
            doc.add_paragraph(p);
        }
        doc
    }

    /// Exports Document AST to plain text
    pub fn export_txt(doc: &Document) -> String {
        let mut out = String::new();
        for sec in &doc.sections {
            for elem in &sec.elements {
                out.push_str(&elem.plain_text());
                out.push('\n');
            }
        }
        out
    }
}
