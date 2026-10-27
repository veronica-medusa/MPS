use crate::error::CoreError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LamportTimestamp {
    pub counter: u64,
    pub actor_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemotePresence {
    pub actor_id: String,
    pub display_name: String,
    pub color_hex: String,
    pub active_tab_id: String,
    pub cursor_pos: Option<usize>,
    pub selection_range: Option<(usize, usize)>,
    pub last_seen_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrdtCharacter {
    pub id: LamportTimestamp,
    pub ch: char,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtOperation {
    Insert {
        id: LamportTimestamp,
        after_id: Option<LamportTimestamp>,
        ch: char,
    },
    Delete {
        id: LamportTimestamp,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrdtDocument {
    pub actor_id: String,
    pub counter: u64,
    pub characters: Vec<CrdtCharacter>,
}

impl CrdtDocument {
    pub fn new(actor_id: &str) -> Self {
        Self {
            actor_id: actor_id.to_string(),
            counter: 0,
            characters: Vec::new(),
        }
    }

    pub fn next_timestamp(&mut self) -> LamportTimestamp {
        self.counter += 1;
        LamportTimestamp {
            counter: self.counter,
            actor_id: self.actor_id.clone(),
        }
    }

    pub fn insert_local(&mut self, visible_index: usize, ch: char) -> CrdtOperation {
        let ts = self.next_timestamp();
        let after_id = if visible_index == 0 {
            None
        } else {
            let mut seen = 0;
            let mut id = None;
            for c in &self.characters {
                if !c.is_deleted {
                    seen += 1;
                    if seen == visible_index {
                        id = Some(c.id.clone());
                        break;
                    }
                }
            }
            id
        };

        let op = CrdtOperation::Insert {
            id: ts.clone(),
            after_id: after_id.clone(),
            ch,
        };
        self.apply_operation(op.clone()).expect("Local insert must succeed");
        op
    }

    pub fn delete_local(&mut self, visible_index: usize) -> Result<CrdtOperation, CoreError> {
        let mut seen = 0;
        let mut target_id = None;
        for c in &self.characters {
            if !c.is_deleted {
                if seen == visible_index {
                    target_id = Some(c.id.clone());
                    break;
                }
                seen += 1;
            }
        }

        let id = target_id.ok_or_else(|| CoreError::CrdtSync("Index out of bounds".into()))?;
        let op = CrdtOperation::Delete { id };
        self.apply_operation(op.clone())?;
        Ok(op)
    }

    pub fn apply_operation(&mut self, op: CrdtOperation) -> Result<(), CoreError> {
        match op {
            CrdtOperation::Insert { id, after_id, ch } => {
                if id.counter > self.counter {
                    self.counter = id.counter;
                }

                if self.characters.iter().any(|c| c.id == id) {
                    return Ok(()); // Idempotent
                }

                let insert_pos = match after_id {
                    None => {
                        // Insert at start, preserving total order among concurrent roots
                        let mut pos = 0;
                        while pos < self.characters.len() && self.characters[pos].id > id {
                            pos += 1;
                        }
                        pos
                    }
                    Some(prev_id) => {
                        let idx = self.characters.iter().position(|c| c.id == prev_id)
                            .ok_or_else(|| CoreError::CrdtSync(format!("Preceding node {:?} not found", prev_id)))?;
                        let mut pos = idx + 1;
                        while pos < self.characters.len() && self.characters[pos].id > id {
                            pos += 1;
                        }
                        pos
                    }
                };

                self.characters.insert(insert_pos, CrdtCharacter {
                    id,
                    ch,
                    is_deleted: false,
                });
                Ok(())
            }
            CrdtOperation::Delete { id } => {
                if let Some(c) = self.characters.iter_mut().find(|c| c.id == id) {
                    c.is_deleted = true;
                }
                Ok(())
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.characters.iter()
            .filter(|c| !c.is_deleted)
            .map(|c| c.ch)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_convergence() {
        let mut site_a = CrdtDocument::new("site-a");
        let mut site_b = CrdtDocument::new("site-b");

        let op1 = site_a.insert_local(0, 'H');
        let op2 = site_a.insert_local(1, 'i');

        site_b.apply_operation(op1).unwrap();
        site_b.apply_operation(op2).unwrap();

        assert_eq!(site_b.to_string(), "Hi");

        // Concurrent edits
        let op_a = site_a.insert_local(2, '!');
        let op_b = site_b.insert_local(2, '?');

        site_a.apply_operation(op_b).unwrap();
        site_b.apply_operation(op_a).unwrap();

        assert_eq!(site_a.to_string(), site_b.to_string());
    }
}
