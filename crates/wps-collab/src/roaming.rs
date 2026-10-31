use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub snapshot_id: String,
    pub document_id: String,
    pub timestamp_ms: u64,
    pub author: String,
    pub label: String,
    pub byte_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    pub metadata: SnapshotMetadata,
    pub content_bytes: Vec<u8>,
}

#[derive(Default)]
pub struct FileRoamingStore {
    // Map of snapshot_id -> DocumentSnapshot
    snapshots: HashMap<String, DocumentSnapshot>,
    // Map of document_id -> list of snapshot_ids in chronological order
    doc_history: HashMap<String, Vec<String>>,
}

impl FileRoamingStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes a content-addressed SHA256-like hex hash for the snapshot
    fn hash_content(data: &[u8], timestamp_ms: u64) -> String {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a 64-bit basis
        for b in data {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= timestamp_ms;
        hash = hash.wrapping_mul(0x100000001b3);
        format!("{:016x}", hash)
    }

    /// Creates and persists a point-in-time document snapshot
    pub fn create_snapshot(
        &mut self,
        doc_id: &str,
        author: &str,
        label: &str,
        content: &[u8],
        timestamp_ms: u64,
    ) -> String {
        let snapshot_id = Self::hash_content(content, timestamp_ms);

        let metadata = SnapshotMetadata {
            snapshot_id: snapshot_id.clone(),
            document_id: doc_id.to_string(),
            timestamp_ms,
            author: author.to_string(),
            label: label.to_string(),
            byte_size: content.len(),
        };

        let snapshot = DocumentSnapshot {
            metadata,
            content_bytes: content.to_vec(),
        };

        self.snapshots.insert(snapshot_id.clone(), snapshot);
        self.doc_history.entry(doc_id.to_string()).or_default().push(snapshot_id.clone());

        snapshot_id
    }

    /// Retrieves snapshot content for instant point-in-time reversion
    pub fn get_snapshot(&self, snapshot_id: &str) -> Option<&DocumentSnapshot> {
        self.snapshots.get(snapshot_id)
    }

    /// Lists chronological version history for a given document
    pub fn list_history(&self, doc_id: &str) -> Vec<SnapshotMetadata> {
        if let Some(ids) = self.doc_history.get(doc_id) {
            ids.iter().filter_map(|id| self.snapshots.get(id).map(|s| s.metadata.clone())).collect()
        } else {
            Vec::new()
        }
    }
}
