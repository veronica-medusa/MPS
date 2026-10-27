use crate::error::OoxmlError;
use std::collections::HashMap;

pub const NS_RELATIONSHIPS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
pub const NS_CONTENT_TYPES: &str = "http://schemas.openxmlformats.org/package/2006/content-types";

pub const REL_OFFICE_DOCUMENT: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";
pub const REL_STYLES: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
pub const REL_SHARED_STRINGS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings";
pub const REL_WORKSHEET: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet";
pub const REL_SLIDE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
pub const REL_SLIDE_MASTER: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster";
pub const REL_SLIDE_LAYOUT: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout";
pub const REL_THEME: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";

pub const CT_DOCX_MAIN: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml";
pub const CT_XLSX_WORKBOOK: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml";
pub const CT_XLSX_WORKSHEET: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml";
pub const CT_XLSX_SHARED_STRINGS: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml";
pub const CT_XLSX_STYLES: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml";
pub const CT_PPTX_PRESENTATION: &str = "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml";
pub const CT_PPTX_SLIDE: &str = "application/vnd.openxmlformats-officedocument.presentationml.slide+xml";

#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    pub id: String,
    pub rel_type: String,
    pub target: String,
    pub target_mode: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Relationships {
    pub items: HashMap<String, Relationship>,
}

impl Relationships {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: &str, rel_type: &str, target: &str) {
        self.items.insert(
            id.to_string(),
            Relationship {
                id: id.to_string(),
                rel_type: rel_type.to_string(),
                target: target.to_string(),
                target_mode: None,
            },
        );
    }

    pub fn find_by_type(&self, rel_type: &str) -> Option<&Relationship> {
        self.items.values().find(|r| r.rel_type == rel_type)
    }

    pub fn find_all_by_type(&self, rel_type: &str) -> Vec<&Relationship> {
        self.items.values().filter(|r| r.rel_type == rel_type).collect()
    }

    pub fn from_xml(xml: &str) -> Result<Self, OoxmlError> {
        let mut rels = Self::new();
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Empty(ref e)) | Ok(quick_xml::events::Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"Relationship" {
                        let mut id = String::new();
                        let mut rel_type = String::new();
                        let mut target = String::new();
                        let mut target_mode = None;

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"Id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                                b"Type" => rel_type = String::from_utf8_lossy(&attr.value).to_string(),
                                b"Target" => target = String::from_utf8_lossy(&attr.value).to_string(),
                                b"TargetMode" => target_mode = Some(String::from_utf8_lossy(&attr.value).to_string()),
                                _ => {}
                            }
                        }

                        if !id.is_empty() {
                            rels.items.insert(
                                id.clone(),
                                Relationship {
                                    id,
                                    rel_type,
                                    target,
                                    target_mode,
                                },
                            );
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }
        Ok(rels)
    }

    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">");
        let mut sorted_items: Vec<&Relationship> = self.items.values().collect();
        sorted_items.sort_by(|a, b| a.id.cmp(&b.id));

        for rel in sorted_items {
            xml.push_str(&format!(
                "<Relationship Id=\"{}\" Type=\"{}\" Target=\"{}\"",
                quick_xml::escape::escape(&rel.id),
                quick_xml::escape::escape(&rel.rel_type),
                quick_xml::escape::escape(&rel.target)
            ));
            if let Some(mode) = &rel.target_mode {
                xml.push_str(&format!(" TargetMode=\"{}\"", quick_xml::escape::escape(mode)));
            }
            xml.push_str("/>");
        }
        xml.push_str("</Relationships>");
        xml
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContentTypes {
    pub defaults: HashMap<String, String>, // ext -> content_type
    pub overrides: HashMap<String, String>, // part_name -> content_type
}

impl ContentTypes {
    pub fn new() -> Self {
        let mut ct = Self::default();
        ct.defaults.insert("rels".to_string(), "application/vnd.openxmlformats-package.relationships+xml".to_string());
        ct.defaults.insert("xml".to_string(), "application/xml".to_string());
        ct
    }

    pub fn from_xml(xml: &str) -> Result<Self, OoxmlError> {
        let mut ct = Self::default();
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Empty(ref e)) => {
                    let local = e.local_name();
                    if local.as_ref() == b"Default" {
                        let mut ext = String::new();
                        let mut content_type = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"Extension" => ext = String::from_utf8_lossy(&attr.value).to_string(),
                                b"ContentType" => content_type = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                        if !ext.is_empty() {
                            ct.defaults.insert(ext, content_type);
                        }
                    } else if local.as_ref() == b"Override" {
                        let mut part = String::new();
                        let mut content_type = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"PartName" => part = String::from_utf8_lossy(&attr.value).to_string(),
                                b"ContentType" => content_type = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                        if !part.is_empty() {
                            ct.overrides.insert(part, content_type);
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }
        Ok(ct)
    }

    pub fn to_xml(&self) -> String {
        let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">");
        for (ext, ct) in &self.defaults {
            xml.push_str(&format!("<Default Extension=\"{}\" ContentType=\"{}\"/>", ext, ct));
        }
        for (part, ct) in &self.overrides {
            xml.push_str(&format!("<Override PartName=\"{}\" ContentType=\"{}\"/>", part, ct));
        }
        xml.push_str("</Types>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationships_xml() {
        let mut rels = Relationships::new();
        rels.add("rId1", REL_OFFICE_DOCUMENT, "word/document.xml");
        let xml = rels.to_xml();

        let parsed = Relationships::from_xml(&xml).unwrap();
        let rel = parsed.find_by_type(REL_OFFICE_DOCUMENT).unwrap();
        assert_eq!(rel.id, "rId1");
        assert_eq!(rel.target, "word/document.xml");
    }

    #[test]
    fn test_content_types_xml() {
        let mut ct = ContentTypes::new();
        ct.overrides.insert("/word/document.xml".to_string(), CT_DOCX_MAIN.to_string());
        let xml = ct.to_xml();

        let parsed = ContentTypes::from_xml(&xml).unwrap();
        assert_eq!(parsed.overrides.get("/word/document.xml").unwrap(), CT_DOCX_MAIN);
    }
}
