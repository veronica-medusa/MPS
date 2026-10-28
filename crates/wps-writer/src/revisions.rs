use serde::{Deserialize, Serialize};
use std::ops::Range;
use crate::buffer::{AttributeMap, RopeBuffer, RunStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MarkupMode {
    #[default]
    AllMarkup,
    SimpleMarkup,
    NoMarkup,
    Original,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RevisionType {
    Insert,
    Delete { deleted_text: String },
    FormatChange { previous_style: RunStyle },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Revision {
    pub id: String,
    pub author: String,
    pub timestamp_sec: u64,
    pub range: Range<usize>,
    pub revision_type: RevisionType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevisionManager {
    pub tracking_enabled: bool,
    pub current_author: String,
    pub markup_mode: MarkupMode,
    pub revisions: Vec<Revision>,
    next_id: usize,
}

impl Default for RevisionManager {
    fn default() -> Self {
        Self {
            tracking_enabled: false,
            current_author: "OpenWPS Author".to_string(),
            markup_mode: MarkupMode::AllMarkup,
            revisions: Vec::new(),
            next_id: 1,
        }
    }
}

impl RevisionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_insert(&mut self, range: Range<usize>) -> String {
        let id = format!("rev-{}", self.next_id);
        self.next_id += 1;
        self.revisions.push(Revision {
            id: id.clone(),
            author: self.current_author.clone(),
            timestamp_sec: 0,
            range,
            revision_type: RevisionType::Insert,
        });
        id
    }

    pub fn record_delete(&mut self, range: Range<usize>, deleted_text: &str) -> String {
        let id = format!("rev-{}", self.next_id);
        self.next_id += 1;
        self.revisions.push(Revision {
            id: id.clone(),
            author: self.current_author.clone(),
            timestamp_sec: 0,
            range,
            revision_type: RevisionType::Delete {
                deleted_text: deleted_text.to_string(),
            },
        });
        id
    }

    pub fn record_format(&mut self, range: Range<usize>, previous_style: RunStyle) -> String {
        let id = format!("rev-{}", self.next_id);
        self.next_id += 1;
        self.revisions.push(Revision {
            id: id.clone(),
            author: self.current_author.clone(),
            timestamp_sec: 0,
            range,
            revision_type: RevisionType::FormatChange { previous_style },
        });
        id
    }

    /// Accepts an individual revision, committing it permanently
    pub fn accept_revision(&mut self, id: &str, buffer: &mut RopeBuffer, _attrs: &mut AttributeMap) -> bool {
        if let Some(pos) = self.revisions.iter().position(|r| r.id == id) {
            let rev = self.revisions.remove(pos);
            match rev.revision_type {
                RevisionType::Insert => {
                    // Permanently accepted: text stays
                }
                RevisionType::Delete { .. } => {
                    // Permanently accepted: remove from buffer
                    buffer.delete(rev.range);
                }
                RevisionType::FormatChange { .. } => {
                    // New format stays
                }
            }
            true
        } else {
            false
        }
    }

    /// Rejects an individual revision, reverting the change
    pub fn reject_revision(&mut self, id: &str, buffer: &mut RopeBuffer, attrs: &mut AttributeMap) -> bool {
        if let Some(pos) = self.revisions.iter().position(|r| r.id == id) {
            let rev = self.revisions.remove(pos);
            match rev.revision_type {
                RevisionType::Insert => {
                    // Reverted: delete inserted text
                    buffer.delete(rev.range.clone());
                    attrs.shift_on_delete(rev.range);
                }
                RevisionType::Delete { deleted_text } => {
                    // Reverted: text was never truly deleted in rope if tracked, or re-insert
                    buffer.insert(rev.range.start, &deleted_text);
                    attrs.shift_on_insert(rev.range.start, deleted_text.chars().count());
                }
                RevisionType::FormatChange { previous_style } => {
                    // Reverted: restore old style
                    attrs.apply_style(rev.range, previous_style);
                }
            }
            true
        } else {
            false
        }
    }

    /// Accepts all tracked revisions
    pub fn accept_all(&mut self, buffer: &mut RopeBuffer, attrs: &mut AttributeMap) {
        let mut ids: Vec<String> = self.revisions.iter().map(|r| r.id.clone()).collect();
        // Process deletions from right to left to avoid offset shifting issues
        ids.reverse();
        for id in ids {
            self.accept_revision(&id, buffer, attrs);
        }
    }

    /// Rejects all tracked revisions
    pub fn reject_all(&mut self, buffer: &mut RopeBuffer, attrs: &mut AttributeMap) {
        let mut ids: Vec<String> = self.revisions.iter().map(|r| r.id.clone()).collect();
        ids.reverse();
        for id in ids {
            self.reject_revision(&id, buffer, attrs);
        }
    }
}
