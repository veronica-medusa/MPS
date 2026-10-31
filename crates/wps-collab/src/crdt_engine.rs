//! Multi-Engine CRDT Document Binding Engine (`wps-collab`)
//!
//! Provides lock-free collaborative editing for Writer (YText-style character buffers),
//! Spreadsheets (LWW Lamport cell maps), Presentation (YArray slide orders & node transforms),
//! and PDF Toolkit (shared annotation records).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Unique identifier for a collaborator client node.
pub type ClientId = u64;

/// Lamport logical timestamp for total causal ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LamportClock {
    pub time: u64,
    pub client_id: ClientId,
}

impl LamportClock {
    pub fn new(time: u64, client_id: ClientId) -> Self {
        Self { time, client_id }
    }

    pub fn tick(&mut self) -> Self {
        self.time += 1;
        *self
    }
}

// ============================================================================
// 1. WRITER CRDT: Collaborative Character Buffer & Attribute Spans
// ============================================================================

/// Character-level mutation for collaborative document text editing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriterTextOp {
    Insert {
        char_id: LamportClock,
        origin_left: Option<LamportClock>,
        origin_right: Option<LamportClock>,
        character: char,
    },
    Delete {
        target_char_id: LamportClock,
    },
    FormatSpan {
        start_char_id: LamportClock,
        end_char_id: LamportClock,
        attribute_key: String,
        attribute_val: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WriterCrdtBinding {
    pub chars: Vec<(LamportClock, char, bool)>, // (id, char, is_deleted)
    pub attributes: Vec<(LamportClock, LamportClock, String, String)>,
}

impl WriterCrdtBinding {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_text(&mut self, offset: usize, text: &str, mut clock: LamportClock) -> Vec<WriterTextOp> {
        let mut ops = Vec::new();
        let active_chars: Vec<usize> = self.chars.iter().enumerate()
            .filter_map(|(idx, (_, _, del))| if !del { Some(idx) } else { None })
            .collect();

        let mut prev_id = if offset > 0 && offset <= active_chars.len() {
            Some(self.chars[active_chars[offset - 1]].0)
        } else {
            None
        };

        for ch in text.chars() {
            let next_clock = clock.tick();
            let op = WriterTextOp::Insert {
                char_id: next_clock,
                origin_left: prev_id,
                origin_right: None,
                character: ch,
            };
            self.apply_text_op(&op);
            ops.push(op);
            prev_id = Some(next_clock);
        }

        ops
    }

    pub fn delete_range(&mut self, offset: usize, len: usize) -> Vec<WriterTextOp> {
        let mut ops = Vec::new();
        let active_chars: Vec<usize> = self.chars.iter().enumerate()
            .filter_map(|(idx, (_, _, del))| if !del { Some(idx) } else { None })
            .collect();

        let end = (offset + len).min(active_chars.len());
        for &idx in &active_chars[offset..end] {
            let op = WriterTextOp::Delete { target_char_id: self.chars[idx].0 };
            self.apply_text_op(&op);
            ops.push(op);
        }

        ops
    }

    pub fn apply_text_op(&mut self, op: &WriterTextOp) {
        match op {
            WriterTextOp::Insert { char_id, origin_left, character, .. } => {
                if self.chars.iter().any(|(id, _, _)| id == char_id) {
                    return; // Idempotent
                }

                let insert_pos = if let Some(left_id) = origin_left {
                    if let Some(pos) = self.chars.iter().position(|(id, _, _)| id == left_id) {
                        pos + 1
                    } else {
                        self.chars.len()
                    }
                } else {
                    0
                };

                self.chars.insert(insert_pos, (*char_id, *character, false));
            }
            WriterTextOp::Delete { target_char_id } => {
                if let Some(pos) = self.chars.iter().position(|(id, _, _)| id == target_char_id) {
                    self.chars[pos].2 = true;
                }
            }
            WriterTextOp::FormatSpan { start_char_id, end_char_id, attribute_key, attribute_val } => {
                self.attributes.push((*start_char_id, *end_char_id, attribute_key.clone(), attribute_val.clone()));
            }
        }
    }
}

impl std::fmt::Display for WriterCrdtBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_, ch, del) in &self.chars {
            if !del {
                write!(f, "{ch}")?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// 2. SPREADSHEETS CRDT: LWW Cell Maps with Dependency Triggers
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpreadsheetCellMutation {
    pub sheet_name: String,
    pub row: u32,
    pub col: u16,
    pub raw_input: String,
    pub style_id: Option<String>,
    pub comment: Option<String>,
    pub timestamp: LamportClock,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SheetsCrdtBinding {
    // Key: "Sheet:Row:Col" -> Mutation
    pub cells: BTreeMap<String, SpreadsheetCellMutation>,
}

impl SheetsCrdtBinding {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_cell(
        &mut self,
        sheet_name: &str,
        row: u32,
        col: u16,
        raw_input: &str,
        clock: LamportClock,
    ) -> SpreadsheetCellMutation {
        let mutation = SpreadsheetCellMutation {
            sheet_name: sheet_name.to_string(),
            row,
            col,
            raw_input: raw_input.to_string(),
            style_id: None,
            comment: None,
            timestamp: clock,
        };

        self.apply_cell_mutation(mutation.clone());
        mutation
    }

    pub fn apply_cell_mutation(&mut self, mutation: SpreadsheetCellMutation) -> bool {
        let key = format!("{}:{}:{}", mutation.sheet_name, mutation.row, mutation.col);
        if let Some(existing) = self.cells.get(&key) {
            // Last-Write-Wins (LWW) Lamport comparison
            if mutation.timestamp <= existing.timestamp {
                return false; // Stale mutation ignored
            }
        }

        self.cells.insert(key, mutation);
        true
    }
}

// ============================================================================
// 3. PRESENTATION CRDT: Slide Order Array & Node Transforms
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlideMutation {
    InsertSlide { slide_id: String, index: usize, clock: LamportClock },
    ReorderSlide { slide_id: String, new_index: usize, clock: LamportClock },
    DeleteSlide { slide_id: String, clock: LamportClock },
    TransformNode {
        slide_id: String,
        node_id: String,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        clock: LamportClock,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SlidesCrdtBinding {
    pub slide_order: Vec<String>,
    pub node_transforms: BTreeMap<String, (f64, f64, f64, f64, LamportClock)>, // (node_id -> x, y, w, h, clock)
}

impl SlidesCrdtBinding {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_mutation(&mut self, mutation: SlideMutation) {
        match mutation {
            SlideMutation::InsertSlide { slide_id, index, .. } => {
                if !self.slide_order.contains(&slide_id) {
                    let idx = index.min(self.slide_order.len());
                    self.slide_order.insert(idx, slide_id);
                }
            }
            SlideMutation::ReorderSlide { slide_id, new_index, .. } => {
                if let Some(pos) = self.slide_order.iter().position(|id| id == &slide_id) {
                    let id = self.slide_order.remove(pos);
                    let target = new_index.min(self.slide_order.len());
                    self.slide_order.insert(target, id);
                }
            }
            SlideMutation::DeleteSlide { slide_id, .. } => {
                self.slide_order.retain(|id| id != &slide_id);
            }
            SlideMutation::TransformNode { node_id, x, y, width, height, clock, .. } => {
                if let Some((_, _, _, _, existing_clock)) = self.node_transforms.get(&node_id) {
                    if clock <= *existing_clock {
                        return;
                    }
                }
                self.node_transforms.insert(node_id, (x, y, width, height, clock));
            }
        }
    }
}

// ============================================================================
// 4. PDF CRDT: Shared Annotations Map
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfAnnotationCrdtRecord {
    pub id: u64,
    pub page_index: usize,
    pub author: String,
    pub contents: String,
    pub is_deleted: bool,
    pub clock: LamportClock,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PdfCrdtBinding {
    pub annotations: BTreeMap<u64, PdfAnnotationCrdtRecord>,
}

impl PdfCrdtBinding {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_annotation(&mut self, record: PdfAnnotationCrdtRecord) -> bool {
        if let Some(existing) = self.annotations.get(&record.id) {
            if record.clock <= existing.clock {
                return false;
            }
        }
        self.annotations.insert(record.id, record);
        true
    }
}

// ============================================================================
// 5. UNIFIED MULTI-ENGINE CRDT DOCUMENT SESSION
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrdtOperation {
    Writer(WriterTextOp),
    Sheets(SpreadsheetCellMutation),
    Slides(SlideMutation),
    Pdf(PdfAnnotationCrdtRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiEngineCrdtDoc {
    pub doc_id: String,
    pub writer: WriterCrdtBinding,
    pub sheets: SheetsCrdtBinding,
    pub slides: SlidesCrdtBinding,
    pub pdf: PdfCrdtBinding,
    pub clock: LamportClock,
    pub state_vector: HashMap<ClientId, u64>,
}

impl MultiEngineCrdtDoc {
    pub fn new(doc_id: &str, client_id: ClientId) -> Self {
        let mut state_vector = HashMap::new();
        state_vector.insert(client_id, 0);

        Self {
            doc_id: doc_id.to_string(),
            writer: WriterCrdtBinding::new(),
            sheets: SheetsCrdtBinding::new(),
            slides: SlidesCrdtBinding::new(),
            pdf: PdfCrdtBinding::new(),
            clock: LamportClock::new(0, client_id),
            state_vector,
        }
    }

    pub fn apply_op(&mut self, op: CrdtOperation) {
        match op {
            CrdtOperation::Writer(top) => self.writer.apply_text_op(&top),
            CrdtOperation::Sheets(cop) => { self.sheets.apply_cell_mutation(cop); },
            CrdtOperation::Slides(sop) => self.slides.apply_mutation(sop),
            CrdtOperation::Pdf(aop) => { self.pdf.apply_annotation(aop); },
        }
    }
}
