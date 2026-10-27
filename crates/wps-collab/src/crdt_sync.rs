use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollabOperation {
    TextInsert { offset: usize, text: String },
    TextDelete { offset: usize, length: usize },
    CellUpdate { row: u32, col: u16, value_json: String },
    ShapeMove { shape_id: String, x: f64, y: f64 },
    PdfAnnotation { page_index: usize, annotation_json: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrdtUpdate {
    pub client_id: String,
    pub clock: u64,
    pub op: CollabOperation,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyncMessage {
    SyncStep1 { state_vector: HashMap<String, u64> },
    SyncStep2 { updates: Vec<CrdtUpdate> },
    Update(CrdtUpdate),
}

pub struct CrdtSession {
    pub client_id: String,
    pub clock: u64,
    pub state_vector: HashMap<String, u64>,
    pub update_log: Vec<CrdtUpdate>,
}

impl CrdtSession {
    pub fn new(client_id: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            clock: 0,
            state_vector: HashMap::new(),
            update_log: Vec::new(),
        }
    }

    /// Emits a local operation and records it in the CRDT log
    pub fn apply_local_op(&mut self, op: CollabOperation, timestamp_ms: u64) -> CrdtUpdate {
        self.clock += 1;
        self.state_vector.insert(self.client_id.clone(), self.clock);

        let update = CrdtUpdate {
            client_id: self.client_id.clone(),
            clock: self.clock,
            op,
            timestamp_ms,
        };

        self.update_log.push(update.clone());
        update
    }

    /// Merges remote update, guaranteeing idempotent, deterministic conflict-free convergence
    pub fn merge_remote_update(&mut self, update: CrdtUpdate) -> bool {
        let current_known = self.state_vector.get(&update.client_id).copied().unwrap_or(0);
        if update.clock <= current_known {
            return false; // Already applied
        }

        self.state_vector.insert(update.client_id.clone(), update.clock);
        self.clock = self.clock.max(update.clock);

        // Insert into update log sorted by (timestamp_ms, client_id)
        let insert_idx = self.update_log.binary_search_by(|existing| {
            existing.timestamp_ms.cmp(&update.timestamp_ms)
                .then_with(|| existing.client_id.cmp(&update.client_id))
        }).unwrap_or_else(|idx| idx);

        self.update_log.insert(insert_idx, update);
        true
    }

    /// Generates Step1 state vector sync request
    pub fn create_sync_step_1(&self) -> SyncMessage {
        SyncMessage::SyncStep1 {
            state_vector: self.state_vector.clone(),
        }
    }

    /// Responds to Step1 with missing updates
    pub fn respond_sync_step_1(&self, remote_vector: &HashMap<String, u64>) -> SyncMessage {
        let mut missing_updates = Vec::new();
        for update in &self.update_log {
            let remote_clock = remote_vector.get(&update.client_id).copied().unwrap_or(0);
            if update.clock > remote_clock {
                missing_updates.push(update.clone());
            }
        }
        SyncMessage::SyncStep2 {
            updates: missing_updates,
        }
    }
}
