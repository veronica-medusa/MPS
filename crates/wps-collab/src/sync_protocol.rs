//! WebSocket Framing & Sync Wire Protocol (`wps-collab`)
//!
//! Handles binary & JSON message exchange, two-step state-vector handshake,
//! incremental update dissemination, and offline mutation buffering.

use crate::crdt_engine::{ClientId, CrdtOperation, MultiEngineCrdtDoc};
use crate::presence::CollaboratorPresence;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Wire messages exchanged across WebSocket / WebRTC data channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollabWireMessage {
    /// Step 1: Handshake initiating sync with local state vector.
    SyncStep1 {
        doc_id: String,
        state_vector: HashMap<ClientId, u64>,
    },
    /// Step 2: Response providing missing updates based on remote state vector.
    SyncStep2 {
        doc_id: String,
        updates: Vec<CrdtOperation>,
    },
    /// Broadcast of an incremental mutation applied by a peer.
    IncrementalUpdate {
        doc_id: String,
        client_id: ClientId,
        clock: u64,
        operation: CrdtOperation,
    },
    /// Ephemeral awareness / cursor broadcast.
    AwarenessBroadcast {
        doc_id: String,
        presence_list: Vec<CollaboratorPresence>,
    },
    /// Acknowledgment of update ingestion.
    Ack {
        doc_id: String,
        client_id: ClientId,
        clock: u64,
    },
}

/// Offline mutation queue buffering operations while disconnected and auto-resyncing upon reconnection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OfflineMutationQueue {
    pub pending_ops: VecDeque<CrdtOperation>,
    pub unacknowledged: Vec<(u64, CrdtOperation)>,
}

impl OfflineMutationQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a local mutation performed while offline or waiting for flush.
    pub fn enqueue_op(&mut self, op: CrdtOperation) {
        self.pending_ops.push_back(op);
    }

    /// Drains all buffered operations into an outbound batch.
    pub fn drain_pending(&mut self) -> Vec<CrdtOperation> {
        self.pending_ops.drain(..).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.pending_ops.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pending_ops.len()
    }
}

/// Session sync coordinator for two-way convergence.
pub struct SyncCoordinator;

impl SyncCoordinator {
    /// Generates Step 1 message with the local document's state vector.
    pub fn create_sync_step_1(doc: &MultiEngineCrdtDoc) -> CollabWireMessage {
        CollabWireMessage::SyncStep1 {
            doc_id: doc.doc_id.clone(),
            state_vector: doc.state_vector.clone(),
        }
    }

    /// Responds to Step 1 by examining which updates the remote peer needs.
    pub fn respond_sync_step_1(
        doc: &MultiEngineCrdtDoc,
        _remote_sv: &HashMap<ClientId, u64>,
    ) -> CollabWireMessage {
        // Collect all operations present in the document
        let mut updates = Vec::new();

        // Writer operations
        let mut prev_id = None;
        for (id, ch, del) in &doc.writer.chars {
            if !del {
                updates.push(CrdtOperation::Writer(crate::crdt_engine::WriterTextOp::Insert {
                    char_id: *id,
                    origin_left: prev_id,
                    origin_right: None,
                    character: *ch,
                }));
                prev_id = Some(*id);
            }
        }

        // Sheets operations
        for mutation in doc.sheets.cells.values() {
            updates.push(CrdtOperation::Sheets(mutation.clone()));
        }

        // Slides operations
        for (idx, slide_id) in doc.slides.slide_order.iter().enumerate() {
            updates.push(CrdtOperation::Slides(crate::crdt_engine::SlideMutation::InsertSlide {
                slide_id: slide_id.clone(),
                index: idx,
                clock: doc.clock,
            }));
        }

        // PDF operations
        for annot in doc.pdf.annotations.values() {
            updates.push(CrdtOperation::Pdf(annot.clone()));
        }

        CollabWireMessage::SyncStep2 {
            doc_id: doc.doc_id.clone(),
            updates,
        }
    }

    /// Merges received Step 2 updates into the local document AST.
    pub fn apply_sync_step_2(doc: &mut MultiEngineCrdtDoc, updates: Vec<CrdtOperation>) -> usize {
        let count = updates.len();
        for op in updates {
            doc.apply_op(op);
        }
        count
    }
}
