pub mod crdt_engine;
pub mod crdt_sync;
pub mod file_roaming;
pub mod presence;
pub mod roaming;
pub mod sync_protocol;

pub use crdt_engine::{
    ClientId, CrdtOperation, LamportClock, MultiEngineCrdtDoc, PdfAnnotationCrdtRecord,
    PdfCrdtBinding, SheetsCrdtBinding, SlideMutation, SlidesCrdtBinding, SpreadsheetCellMutation,
    WriterCrdtBinding, WriterTextOp,
};
pub use crdt_sync::{CollabOperation, CrdtSession, CrdtUpdate, SyncMessage};
pub use file_roaming::{BlobHash, FileRoamingStore as ModernFileRoamingStore, RoamingVersionRecord, VersionDiffSummary};
pub use presence::{CollaboratorPresence, CursorLocation, OfficeModule, PresenceRegistry, UserPresence};
pub use roaming::{DocumentSnapshot, FileRoamingStore, SnapshotMetadata};
pub use sync_protocol::{CollabWireMessage, OfflineMutationQueue, SyncCoordinator};

#[cfg(test)]
mod tests {
    use super::*;
    use wps_core::color::Color;

    #[test]
    fn test_crdt_sync_convergence() {
        let mut client_a = CrdtSession::new("client-A");
        let mut client_b = CrdtSession::new("client-B");

        // Client A types "Hello"
        let update_a = client_a.apply_local_op(
            CollabOperation::TextInsert { offset: 0, text: "Hello ".to_string() },
            1000,
        );

        // Client B edits cell A1
        let update_b = client_b.apply_local_op(
            CollabOperation::CellUpdate { row: 0, col: 0, value_json: "42".to_string() },
            1005,
        );

        // Sync A -> B
        let step1_b = client_b.create_sync_step_1();
        if let SyncMessage::SyncStep1 { state_vector } = step1_b {
            let step2_a = client_a.respond_sync_step_1(&state_vector);
            if let SyncMessage::SyncStep2 { updates } = step2_a {
                for u in updates {
                    client_b.merge_remote_update(u);
                }
            }
        }

        // Sync B -> A
        client_a.merge_remote_update(update_b);

        // Both clients now possess both operations in identical order
        assert_eq!(client_a.update_log.len(), 2);
        assert_eq!(client_b.update_log.len(), 2);
        assert_eq!(client_a.update_log[0].op, update_a.op);
        assert_eq!(client_b.update_log[0].op, update_a.op);
    }

    #[test]
    fn test_presence_registry_and_cleanup() {
        let mut reg = PresenceRegistry::new();

        reg.update_user(UserPresence {
            user_id: "user-1".to_string(),
            name: "Alice".to_string(),
            color: Color::rgb(59, 91, 219),
            active_module: OfficeModule::Writer,
            cursor: CursorLocation::TextOffset(120),
            last_active_ms: 10_000,
        });

        reg.update_user(UserPresence {
            user_id: "user-2".to_string(),
            name: "Bob".to_string(),
            color: Color::rgb(12, 166, 120),
            active_module: OfficeModule::Spreadsheets,
            cursor: CursorLocation::GridCell { row: 5, col: 2 },
            last_active_ms: 5_000, // Stale
        });

        assert_eq!(reg.get_active_users().len(), 2);

        // Cleanup with 10_000ms current time and 4_000ms timeout
        // user-2 last active at 5_000 -> 10_000 - 5_000 = 5_000 > 4_000 (Timed out)
        reg.cleanup_inactive(10_000, 4_000);
        let active = reg.get_active_users();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Alice");
    }

    #[test]
    fn test_file_roaming_snapshots() {
        let mut store = FileRoamingStore::new();

        let v1_id = store.create_snapshot("doc-123", "Alice", "Initial Draft", b"Version 1 content", 1000);
        let v2_id = store.create_snapshot("doc-123", "Bob", "Added Financials", b"Version 2 content", 2000);

        let history = store.list_history("doc-123");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].label, "Initial Draft");
        assert_eq!(history[1].label, "Added Financials");

        // Point-in-time recovery
        let v1 = store.get_snapshot(&v1_id).unwrap();
        assert_eq!(v1.content_bytes, b"Version 1 content");

        let v2 = store.get_snapshot(&v2_id).unwrap();
        assert_eq!(v2.content_bytes, b"Version 2 content");
    }
}
