use crate::error::CoreError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq)]
pub struct FileMetadata {
    pub size: usize,
    pub is_dir: bool,
    pub created_at_ms: i64,
    pub modified_at_ms: i64,
}

pub trait VirtualFileSystem: Send + Sync {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, CoreError>;
    fn write_file(&self, path: &str, content: &[u8]) -> Result<(), CoreError>;
    fn exists(&self, path: &str) -> bool;
    fn remove_file(&self, path: &str) -> Result<(), CoreError>;
    fn list_dir(&self, dir_path: &str) -> Result<Vec<String>, CoreError>;
    fn metadata(&self, path: &str) -> Result<FileMetadata, CoreError>;
}

type VfsStorage = Arc<RwLock<HashMap<String, (Vec<u8>, i64)>>>;

/// In-Memory Virtual File System (Thread-safe, testable, wasm-friendly)
#[derive(Clone, Default)]
pub struct MemoryVfs {
    files: VfsStorage,
}

impl MemoryVfs {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl VirtualFileSystem for MemoryVfs {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, CoreError> {
        let files = self.files.read().map_err(|e| CoreError::VfsIo(e.to_string()))?;
        files.get(path)
            .map(|(data, _)| data.clone())
            .ok_or_else(|| CoreError::FileNotFound(path.to_string()))
    }

    fn write_file(&self, path: &str, content: &[u8]) -> Result<(), CoreError> {
        let mut files = self.files.write().map_err(|e| CoreError::VfsIo(e.to_string()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        files.insert(path.to_string(), (content.to_vec(), now));
        Ok(())
    }

    fn exists(&self, path: &str) -> bool {
        let files = self.files.read().unwrap();
        files.contains_key(path)
    }

    fn remove_file(&self, path: &str) -> Result<(), CoreError> {
        let mut files = self.files.write().map_err(|e| CoreError::VfsIo(e.to_string()))?;
        files.remove(path)
            .map(|_| ())
            .ok_or_else(|| CoreError::FileNotFound(path.to_string()))
    }

    fn list_dir(&self, dir_path: &str) -> Result<Vec<String>, CoreError> {
        let files = self.files.read().map_err(|e| CoreError::VfsIo(e.to_string()))?;
        let prefix = if dir_path.ends_with('/') || dir_path.is_empty() {
            dir_path.to_string()
        } else {
            format!("{}/", dir_path)
        };

        let matches = files.keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        Ok(matches)
    }

    fn metadata(&self, path: &str) -> Result<FileMetadata, CoreError> {
        let files = self.files.read().map_err(|e| CoreError::VfsIo(e.to_string()))?;
        let (data, time) = files.get(path)
            .ok_or_else(|| CoreError::FileNotFound(path.to_string()))?;
        Ok(FileMetadata {
            size: data.len(),
            is_dir: false,
            created_at_ms: *time,
            modified_at_ms: *time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_vfs() {
        let vfs = MemoryVfs::new();
        let path = "/docs/document.docx";
        let content = b"PK\x03\x04test_ooxml";

        assert!(!vfs.exists(path));
        vfs.write_file(path, content).unwrap();
        assert!(vfs.exists(path));

        let read = vfs.read_file(path).unwrap();
        assert_eq!(read, content);

        let meta = vfs.metadata(path).unwrap();
        assert_eq!(meta.size, content.len());

        let list = vfs.list_dir("/docs").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], path);

        vfs.remove_file(path).unwrap();
        assert!(!vfs.exists(path));
    }
}
