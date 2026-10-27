use crate::error::OoxmlError;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Debug, Clone, Default)]
pub struct OoxmlPackage {
    parts: HashMap<String, Vec<u8>>,
}

impl OoxmlPackage {
    pub fn new() -> Self {
        Self {
            parts: HashMap::new(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OoxmlError> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| OoxmlError::Zip(e.to_string()))?;
        let mut parts = HashMap::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| OoxmlError::Zip(e.to_string()))?;
            let name = file.name().to_string();
            let mut buffer = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buffer)?;
            parts.insert(name, buffer);
        }

        Ok(Self { parts })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, OoxmlError> {
        let mut buffer = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            let mut sorted_keys: Vec<&String> = self.parts.keys().collect();
            sorted_keys.sort();

            for key in sorted_keys {
                writer.start_file(key, options).map_err(|e| OoxmlError::Zip(e.to_string()))?;
                let data = &self.parts[key];
                writer.write_all(data)?;
            }
            writer.finish().map_err(|e| OoxmlError::Zip(e.to_string()))?;
        }
        Ok(buffer)
    }

    pub fn get_part(&self, path: &str) -> Option<&[u8]> {
        let clean_path = path.trim_start_matches('/');
        self.parts.get(clean_path).map(|v| v.as_slice())
    }

    pub fn get_part_string(&self, path: &str) -> Result<String, OoxmlError> {
        let bytes = self.get_part(path).ok_or_else(|| OoxmlError::MissingPart(path.to_string()))?;
        String::from_utf8(bytes.to_vec()).map_err(|e| OoxmlError::Xml(e.to_string()))
    }

    pub fn set_part(&mut self, path: &str, content: Vec<u8>) {
        let clean_path = path.trim_start_matches('/').to_string();
        self.parts.insert(clean_path, content);
    }

    pub fn set_part_string(&mut self, path: &str, content: &str) {
        self.set_part(path, content.as_bytes().to_vec());
    }

    pub fn has_part(&self, path: &str) -> bool {
        let clean_path = path.trim_start_matches('/');
        self.parts.contains_key(clean_path)
    }

    pub fn part_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.parts.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ooxml_package_roundtrip() {
        let mut pkg = OoxmlPackage::new();
        pkg.set_part_string("[Content_Types].xml", "<Types></Types>");
        pkg.set_part_string("word/document.xml", "<w:document></w:document>");

        let bytes = pkg.to_bytes().unwrap();
        let loaded = OoxmlPackage::from_bytes(&bytes).unwrap();

        assert!(loaded.has_part("[Content_Types].xml"));
        assert!(loaded.has_part("word/document.xml"));
        assert_eq!(loaded.get_part_string("word/document.xml").unwrap(), "<w:document></w:document>");
    }
}
