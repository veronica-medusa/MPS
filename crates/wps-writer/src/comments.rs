use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommentReply {
    pub id: String,
    pub author: String,
    pub created_at_sec: u64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub created_at_sec: u64,
    pub range: Range<usize>,
    pub text: String,
    pub replies: Vec<CommentReply>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CommentStore {
    pub comments: HashMap<String, Comment>,
    next_id: usize,
}

impl CommentStore {
    pub fn new() -> Self {
        Self {
            comments: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add_comment(&mut self, author: &str, range: Range<usize>, text: &str) -> String {
        let id = format!("comment-{}", self.next_id);
        self.next_id += 1;
        self.comments.insert(id.clone(), Comment {
            id: id.clone(),
            author: author.to_string(),
            created_at_sec: 0,
            range,
            text: text.to_string(),
            replies: Vec::new(),
            resolved: false,
        });
        id
    }

    pub fn reply_comment(&mut self, comment_id: &str, author: &str, text: &str) -> bool {
        if let Some(comment) = self.comments.get_mut(comment_id) {
            let reply_id = format!("{}-reply-{}", comment_id, comment.replies.len() + 1);
            comment.replies.push(CommentReply {
                id: reply_id,
                author: author.to_string(),
                created_at_sec: 0,
                text: text.to_string(),
            });
            true
        } else {
            false
        }
    }

    pub fn resolve_comment(&mut self, comment_id: &str) -> bool {
        if let Some(comment) = self.comments.get_mut(comment_id) {
            comment.resolved = true;
            true
        } else {
            false
        }
    }

    pub fn delete_comment(&mut self, comment_id: &str) -> bool {
        self.comments.remove(comment_id).is_some()
    }
}
