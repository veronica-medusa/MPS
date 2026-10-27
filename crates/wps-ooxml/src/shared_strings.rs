use crate::error::OoxmlError;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct SharedStringsTable {
    strings: Vec<String>,
    index_map: HashMap<String, usize>,
}

impl SharedStringsTable {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            index_map: HashMap::new(),
        }
    }

    pub fn get_or_insert(&mut self, text: &str) -> usize {
        if let Some(&idx) = self.index_map.get(text) {
            idx
        } else {
            let idx = self.strings.len();
            let owned = text.to_string();
            self.index_map.insert(owned.clone(), idx);
            self.strings.push(owned);
            idx
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    pub fn from_xml(xml: &str) -> Result<Self, OoxmlError> {
        let mut sst = Self::new();
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().trim_text(false);

        let mut in_t = false;
        let mut current_text = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    if e.local_name().as_ref() == b"t" {
                        in_t = true;
                        current_text.clear();
                    }
                }
                Ok(quick_xml::events::Event::Text(ref e)) => {
                    if in_t {
                        current_text.push_str(&e.unescape().map_err(|err| OoxmlError::Xml(err.to_string()))?);
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    if e.local_name().as_ref() == b"t" {
                        in_t = false;
                    } else if e.local_name().as_ref() == b"si" {
                        sst.get_or_insert(&current_text);
                        current_text.clear();
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(OoxmlError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Ok(sst)
    }

    pub fn to_xml(&self) -> String {
        let mut xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<sst xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" count=\"{}\" uniqueCount=\"{}\">",
            self.strings.len(),
            self.strings.len()
        );
        for s in &self.strings {
            xml.push_str("<si><t>");
            xml.push_str(&quick_xml::escape::escape(s));
            xml.push_str("</t></si>");
        }
        xml.push_str("</sst>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sst_xml_roundtrip() {
        let mut sst = SharedStringsTable::new();
        let id0 = sst.get_or_insert("Revenue");
        let id1 = sst.get_or_insert("Cost");
        let id2 = sst.get_or_insert("Revenue"); // Duplicate

        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 0);
        assert_eq!(sst.len(), 2);

        let xml = sst.to_xml();
        let loaded = SharedStringsTable::from_xml(&xml).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get(0), Some("Revenue"));
        assert_eq!(loaded.get(1), Some("Cost"));
    }
}
