//! WebAssembly & JavaScript File I/O Bridge (`wps-wasm::file_bridge`)
//!
//! Provides `#[wasm_bindgen]` functions to import and export `.docx`, `.xlsx`, `.csv`,
//! `.pptx`, and `.pdf` files using in-memory `Uint8Array` buffers without touching the disk.

use wasm_bindgen::prelude::*;
use wps_ooxml::file_io::{FileFormatKind, FileIoEngine, OfficeDocumentPayload};

/// Imports binary file bytes (`Uint8Array`) and returns the parsed `OfficeDocumentPayload` as a JSON string.
#[wasm_bindgen(js_name = importFileBytes)]
pub fn import_file_bytes(file_name: &str, bytes: &[u8]) -> Result<String, JsValue> {
    let payload = FileIoEngine::import_bytes(file_name, bytes)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse {}: {}", file_name, e)))?;
    serde_json::to_string(&payload)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize payload: {}", e)))
}

/// Exports an `OfficeDocumentPayload` JSON string into binary document bytes (`Uint8Array`).
#[wasm_bindgen(js_name = exportDocumentBytes)]
pub fn export_document_bytes(payload_json: &str) -> Result<Vec<u8>, JsValue> {
    let payload: OfficeDocumentPayload = serde_json::from_str(payload_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid OfficeDocumentPayload JSON: {}", e)))?;
    FileIoEngine::export_bytes(&payload)
        .map_err(|e| JsValue::from_str(&format!("Export failed: {}", e)))
}

/// Detects the office file format from file name and leading magic bytes.
#[wasm_bindgen(js_name = detectFileFormat)]
pub fn detect_file_format(file_name: &str, bytes: &[u8]) -> Result<String, JsValue> {
    let format = FileFormatKind::from_filename_or_magic(file_name, bytes)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&format)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Returns the standard MIME type for an office file format.
#[wasm_bindgen(js_name = getMimeType)]
pub fn get_mime_type(format_str: &str) -> String {
    match format_str.to_ascii_lowercase().as_str() {
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        "csv" => "text/csv; charset=utf-8".to_string(),
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
        "pdf" => "application/pdf".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}
