// OpenWPS Desktop Host (Tauri 2.0 Entry Point)
use std::sync::Mutex;
use wps_wasm::WorkspaceSession;

pub struct DesktopState {
    pub session: Mutex<WorkspaceSession>,
}

fn main() {
    println!("Initializing OpenWPS Desktop Host with Mercury Design System...");
    let session = WorkspaceSession::new();
    println!(
        "OpenWPS Desktop Session ready with active tab: {}",
        session.get_active_tab_id()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use wps_wasm::ModuleKind;

    #[test]
    fn test_desktop_session_initialization() {
        let mut session = WorkspaceSession::new();
        assert_eq!(session.get_active_module(), ModuleKind::Writer);
        assert_eq!(session.get_active_tab_id(), "tab-writer-1");

        // Switch to sheets tab
        let switched = session.switch_tab("tab-sheets-1");
        assert!(switched);
        assert_eq!(session.get_active_module(), ModuleKind::Sheets);

        // Universal export
        let bytes = session.export_tab_bytes("xlsx").unwrap();
        assert!(!bytes.is_empty());
    }
}
