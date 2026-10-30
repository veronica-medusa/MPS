use wps_collab::crdt_engine::{
    CrdtOperation, LamportClock, MultiEngineCrdtDoc,
};
use wps_collab::file_roaming::FileRoamingStore;
use wps_collab::sync_protocol::{CollabWireMessage, SyncCoordinator};

#[test]
fn test_peer_crdt_offline_convergence() {
    let mut peer_a = MultiEngineCrdtDoc::new("doc-shared-1", 101);
    let mut peer_b = MultiEngineCrdtDoc::new("doc-shared-1", 102);

    // 1. Peer A types "Hello " in Writer while offline
    let mut clock_a = LamportClock::new(0, 101);
    let ops_a = peer_a.writer.insert_text(0, "Hello ", clock_a);
    for op in ops_a {
        peer_a.apply_op(CrdtOperation::Writer(op));
    }

    // 2. Peer B updates cell A1 in Spreadsheets while offline
    let mut clock_b = LamportClock::new(0, 102);
    let op_cell_b = peer_b.sheets.update_cell("Sheet1", 0, 0, "42000", clock_b.tick());
    peer_b.apply_op(CrdtOperation::Sheets(op_cell_b));

    // 3. Peer A updates cell B2 while offline
    let op_cell_a = peer_a.sheets.update_cell("Sheet1", 1, 1, "99999", clock_a.tick());
    peer_a.apply_op(CrdtOperation::Sheets(op_cell_a));

    // 4. Exchange SyncStep1 & SyncStep2 handshake: Peer A -> Peer B
    let step1_b = SyncCoordinator::create_sync_step_1(&peer_b);
    if let CollabWireMessage::SyncStep1 { state_vector, .. } = step1_b {
        let step2_a = SyncCoordinator::respond_sync_step_1(&peer_a, &state_vector);
        if let CollabWireMessage::SyncStep2 { updates, .. } = step2_a {
            SyncCoordinator::apply_sync_step_2(&mut peer_b, updates);
        }
    }

    // 5. Exchange SyncStep1 & SyncStep2 handshake: Peer B -> Peer A
    let step1_a = SyncCoordinator::create_sync_step_1(&peer_a);
    if let CollabWireMessage::SyncStep1 { state_vector, .. } = step1_a {
        let step2_b = SyncCoordinator::respond_sync_step_1(&peer_b, &state_vector);
        if let CollabWireMessage::SyncStep2 { updates, .. } = step2_b {
            SyncCoordinator::apply_sync_step_2(&mut peer_a, updates);
        }
    }

    // 6. Assert 100% convergence across both peers
    assert_eq!(peer_a.writer.to_string(), "Hello ");
    assert_eq!(peer_b.writer.to_string(), "Hello ");

    let cell_a1_a = peer_a.sheets.cells.get("Sheet1:0:0").unwrap();
    let cell_a1_b = peer_b.sheets.cells.get("Sheet1:0:0").unwrap();
    assert_eq!(cell_a1_a.raw_input, "42000");
    assert_eq!(cell_a1_b.raw_input, "42000");

    let cell_b2_a = peer_a.sheets.cells.get("Sheet1:1:1").unwrap();
    let cell_b2_b = peer_b.sheets.cells.get("Sheet1:1:1").unwrap();
    assert_eq!(cell_b2_a.raw_input, "99999");
    assert_eq!(cell_b2_b.raw_input, "99999");
}

#[test]
fn test_file_roaming_store_deduplication_and_time_machine() {
    let mut store = FileRoamingStore::new();
    let doc_id = "quarterly-report-2026";

    // Version 1: Initial draft
    let content_v1 = b"Quarterly Report Draft 1: Initial metrics established.";
    let rec1 = store.save_version(
        doc_id,
        "Alice Chen",
        "Initial Checkpoint",
        content_v1,
        "2026-10-30T10:00:00Z",
    );
    assert_eq!(rec1.version_id, 1);
    assert_eq!(rec1.parent_version_id, None);

    // Version 2: Identical content saved again (e.g. auto-save) -> SHA-256 deduplicated in blob storage
    let rec2 = store.save_version(
        doc_id,
        "Auto-Save Service",
        "Auto-Save (Unchanged)",
        content_v1,
        "2026-10-30T10:05:00Z",
    );
    assert_eq!(rec2.version_id, 2);
    assert_eq!(rec2.blob_hash, rec1.blob_hash, "Identical content must produce identical BlobHash");
    assert_eq!(store.blob_storage.len(), 1, "Deduplication must store only 1 unique blob in storage");

    // Version 3: Expanded content
    let content_v3 = b"Quarterly Report Draft 3: Initial metrics established with 150+ formula DAG calculations.";
    let rec3 = store.save_version(
        doc_id,
        "Bob Smith",
        "Added Spreadsheet Calculations",
        content_v3,
        "2026-10-30T10:30:00Z",
    );
    assert_eq!(rec3.version_id, 3);
    assert_eq!(store.blob_storage.len(), 2, "Distinct content must store 2 unique blobs");

    // Compute diff between v1 and v3
    let diff = store.diff_versions(doc_id, 1, 3).expect("Diff calculation should succeed");
    assert_eq!(diff.from_version, 1);
    assert_eq!(diff.to_version, 3);
    assert!(diff.added_words > 0);

    // Time-machine rollback / restore v1
    let restored_v1 = store.restore_version(doc_id, 1).expect("Restoration of v1 should succeed");
    assert_eq!(restored_v1, content_v1, "Restored content must match initial bytes exactly");
}
