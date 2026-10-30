//! Live Multi-Cursor Awareness & Presence Registry (`wps-collab`)
//!
//! Tracks real-time collaborator viewports, carets, selections, and laser pointers
//! across Writer, Spreadsheets, Presentation, and PDF Toolkit.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wps_core::color::Color;

/// Office module identifier for user presence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficeModule {
    Writer,
    Spreadsheets,
    Presentation,
    Pdf,
}

/// Cursor and selection location within a specific document module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CursorLocation {
    WriterCaret {
        paragraph_id: u64,
        char_offset: usize,
        selection_end: Option<usize>,
    },
    SheetSelection {
        sheet_id: u32,
        active_cell: (u32, u16), // (row, col)
        range: (u32, u16, u32, u16), // (r0, c0, r1, c1)
    },
    SlideNodeSelection {
        slide_index: usize,
        selected_node_ids: Vec<u64>,
        laser_point: Option<(f32, f32)>,
    },
    PdfViewportCursor {
        page_index: usize,
        page_x_pt: f32,
        page_y_pt: f32,
    },
    TextOffset(usize),
    GridCell { row: u32, col: u16 },
    None,
}

/// Legacy UserPresence model for backward compatibility with phase 1-5 tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub name: String,
    pub color: Color,
    pub last_active_ms: u64,
    pub active_module: OfficeModule,
    pub cursor: CursorLocation,
}

/// Real-time collaborator awareness state broadcast to peers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollaboratorPresence {
    pub client_id: u64,
    pub user_id: String,
    pub display_name: String,
    pub avatar_color_hex: String,
    pub active_tab_id: String,
    pub last_active_ms: u64,
    pub cursor_location: CursorLocation,
}

impl CollaboratorPresence {
    pub fn new(
        client_id: u64,
        user_id: &str,
        display_name: &str,
        avatar_color_hex: &str,
        active_tab_id: &str,
        now_ms: u64,
    ) -> Self {
        Self {
            client_id,
            user_id: user_id.to_string(),
            display_name: display_name.to_string(),
            avatar_color_hex: avatar_color_hex.to_string(),
            active_tab_id: active_tab_id.to_string(),
            last_active_ms: now_ms,
            cursor_location: CursorLocation::None,
        }
    }

    pub fn is_stale(&self, current_time_ms: u64, timeout_ms: u64) -> bool {
        current_time_ms.saturating_sub(self.last_active_ms) > timeout_ms
    }
}

/// In-memory presence registry managing all active collaborators on a document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresenceRegistry {
    pub collaborators: HashMap<u64, CollaboratorPresence>,
    pub users: HashMap<String, UserPresence>,
}

impl PresenceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_presence(&mut self, presence: CollaboratorPresence) {
        self.collaborators.insert(presence.client_id, presence);
    }

    pub fn remove_peer(&mut self, client_id: u64) -> Option<CollaboratorPresence> {
        self.collaborators.remove(&client_id)
    }

    pub fn purge_stale_peers(&mut self, current_time_ms: u64, timeout_ms: u64) -> Vec<u64> {
        let mut stale_ids = Vec::new();
        for (&client_id, presence) in &self.collaborators {
            if presence.is_stale(current_time_ms, timeout_ms) {
                stale_ids.push(client_id);
            }
        }

        for id in &stale_ids {
            self.collaborators.remove(id);
        }

        stale_ids
    }

    pub fn active_collaborators(&self) -> Vec<CollaboratorPresence> {
        self.collaborators.values().cloned().collect()
    }

    // --- Backward Compatibility for Legacy Tests ---
    pub fn update_user(&mut self, user: UserPresence) {
        self.users.insert(user.user_id.clone(), user);
    }

    pub fn get_active_users(&self) -> Vec<&UserPresence> {
        self.users.values().collect()
    }

    pub fn cleanup_inactive(&mut self, current_time_ms: u64, timeout_ms: u64) {
        self.users.retain(|_, u| current_time_ms.saturating_sub(u.last_active_ms) <= timeout_ms);
    }
}
