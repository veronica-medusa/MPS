use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wps_core::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficeModule {
    Writer,
    Spreadsheets,
    Presentation,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CursorLocation {
    TextOffset(usize),
    GridCell { row: u32, col: u16 },
    SlideObject { slide_id: String, shape_id: Option<String> },
    PdfPoint { page_index: usize, x: f64, y: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub name: String,
    pub color: Color,
    pub active_module: OfficeModule,
    pub cursor: CursorLocation,
    pub last_active_ms: u64,
}

pub struct PresenceRegistry {
    users: HashMap<String, UserPresence>,
}

impl Default for PresenceRegistry {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl PresenceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_user(&mut self, presence: UserPresence) {
        self.users.insert(presence.user_id.clone(), presence);
    }

    pub fn remove_user(&mut self, user_id: &str) {
        self.users.remove(user_id);
    }

    pub fn cleanup_inactive(&mut self, current_time_ms: u64, timeout_ms: u64) {
        self.users.retain(|_, u| current_time_ms.saturating_sub(u.last_active_ms) < timeout_ms);
    }

    pub fn get_active_users(&self) -> Vec<UserPresence> {
        let mut list: Vec<_> = self.users.values().cloned().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }
}
