use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RevisionType {
    Insert,
    Delete,
    FormatChange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Revision {
    pub id: String,
    pub author: String,
    pub timestamp_ms: i64,
    pub rev_type: RevisionType,
    pub original_text: String,
    pub new_text: String,
    pub accepted: Option<bool>, // None = pending, Some(true) = accepted, Some(false) = rejected
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommentThread {
    pub id: String,
    pub author: String,
    pub timestamp_ms: i64,
    pub anchored_text: String,
    pub content: String,
    pub is_resolved: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrackChangesManager {
    pub is_tracking: bool,
    pub revisions: HashMap<String, Revision>,
    pub comments: Vec<CommentThread>,
}

impl TrackChangesManager {
    pub fn new() -> Self {
        Self {
            is_tracking: true,
            revisions: HashMap::new(),
            comments: Vec::new(),
        }
    }

    pub fn record_insertion(&mut self, author: &str, text: &str) -> String {
        let id = format!("rev-{}", self.revisions.len() + 1);
        let rev = Revision {
            id: id.clone(),
            author: author.to_string(),
            timestamp_ms: 1729999000,
            rev_type: RevisionType::Insert,
            original_text: String::new(),
            new_text: text.to_string(),
            accepted: None,
        };
        self.revisions.insert(id.clone(), rev);
        id
    }

    pub fn record_deletion(&mut self, author: &str, text: &str) -> String {
        let id = format!("rev-{}", self.revisions.len() + 1);
        let rev = Revision {
            id: id.clone(),
            author: author.to_string(),
            timestamp_ms: 1729999000,
            rev_type: RevisionType::Delete,
            original_text: text.to_string(),
            new_text: String::new(),
            accepted: None,
        };
        self.revisions.insert(id.clone(), rev);
        id
    }

    pub fn accept_revision(&mut self, id: &str) -> bool {
        if let Some(rev) = self.revisions.get_mut(id) {
            rev.accepted = Some(true);
            true
        } else {
            false
        }
    }

    pub fn reject_revision(&mut self, id: &str) -> bool {
        if let Some(rev) = self.revisions.get_mut(id) {
            rev.accepted = Some(false);
            true
        } else {
            false
        }
    }

    pub fn accept_all(&mut self) {
        for rev in self.revisions.values_mut() {
            if rev.accepted.is_none() {
                rev.accepted = Some(true);
            }
        }
    }

    pub fn reject_all(&mut self) {
        for rev in self.revisions.values_mut() {
            if rev.accepted.is_none() {
                rev.accepted = Some(false);
            }
        }
    }

    pub fn add_comment(&mut self, author: &str, text_anchor: &str, comment: &str) -> String {
        let id = format!("comment-{}", self.comments.len() + 1);
        self.comments.push(CommentThread {
            id: id.clone(),
            author: author.to_string(),
            timestamp_ms: 1729999000,
            anchored_text: text_anchor.to_string(),
            content: comment.to_string(),
            is_resolved: false,
        });
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_changes_flow() {
        let mut mgr = TrackChangesManager::new();
        let rev1 = mgr.record_insertion("Alice", "Hello World");
        let rev2 = mgr.record_deletion("Bob", "Legacy Text");

        assert_eq!(mgr.revisions.len(), 2);

        mgr.accept_revision(&rev1);
        assert_eq!(mgr.revisions.get(&rev1).unwrap().accepted, Some(true));

        mgr.reject_revision(&rev2);
        assert_eq!(mgr.revisions.get(&rev2).unwrap().accepted, Some(false));
    }
}
