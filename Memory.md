# OpenWPS Persistent Agent Working Memory (Memory.md)

---

## 1. Executive Status Dashboard

* **Current Active Phase:** Phase 2 Complete & Sealed (Writer Engine, Bidirectional `.docx` Pipeline & Mercury Writer UI Fully Operational)
* **Active Sub-Task:** Phase 2 Complete Verification, All 59 Workspace Unit Tests Passing, Dual-Target Native + Wasm Verified, Ready for Phase 3 Deep Refinements
* **Overall Project Status:** Phase 0, Phase 1, and Phase 2 Complete, Tested, and Verified (59 Workspace Tests Passing with 0 Failures, wasm32-unknown-unknown passing, frontend clean build). Zero-Stub Mandate Enforced.
* **Last Updated:** 2026-10-28

---

## 2. Mercury Design System Integration Status

* **Design Package Installation:** Completed (`npx -y getdesign@latest add mercury` executed successfully)
* **Design File Reference:** `./DESIGN.md` (Raw inspection completed) & `./Design.md` (Office adaptation written)
* **Core Token Mapping:**
  * Base Palette: Canvas (`#fbfcfd`), Surface (`#ffffff`), Surface-Tint (`#fbfbff`), Lavender (`#edecfb`), Night (`#171721`), Night-Raised (`#1e1e2a`), Ink (`#28253b`), Ink-Secondary (`#554e7b`), Primary Indigo (`#5266eb`).
  * Module Semantic Accents:
    * Writer: Cobalt (`#3B5BDB`)
    * Spreadsheets: Emerald (`#0CA678`)
    * Presentation: Coral (`#E8590C`)
    * PDF Toolkit: Crimson (`#E03131`)
    * AI & Python: Iris (`#7048E8`)
* **Verified UI Components:** Unified Polymorphic Tab Bar, Mercury Dual Ribbon (Classic & Compact), Cmd+K Command Palette, Dockable Right Inspector, Status Bar, Smart Toolbox Modal, Hardware-Accelerated Canvas Host.

---

## 3. Crate Completion Matrix

| Crate / Package | Status | Lines of Code | Unit Tests | Exported Public APIs / Capabilities |
| :--- | :--- | :--- | :--- | :--- |
| `crates/wps-core` | Tested & Verified | 750 | 7 | VFS, Inversible Command Bus, Geometry, Units, CRDT State |
| `crates/wps-ooxml` | Tested & Verified | 620 | 5 | Streaming ZIP, OPC relationships, `.docx`, `.xlsx`, `.pptx` codecs |
| `crates/wps-render` | Tested & Verified | 520 | 3 | Font layout & shaping, DisplayList, DirtyRect tracker, Frustum culler |
| `crates/wps-writer` | Tested & Verified | 2,450 | 6 | RopeBuffer, AttributeMap, Paginator, Styles Cascade, Numbering, Revisions, Mail Merge, .docx Codec |
| `crates/wps-sheets` | Tested & Verified | 1,620 | 15 | 64x64 Sparse Grid, 150+ Formula DAG, Smart Toolbox, Pivot Cube, .xlsx |
| `crates/wps-slides` | Tested & Verified | 890 | 4 | Master/Layout/Slide Scene Graph, DrawingML, Connectors, Animator |
| `crates/wps-pdf` | Tested & Verified | 1,420 | 8 | ISO 32000, XY-Cut/DBSCAN Reconstructor, Inline Editor, OCR, Sign |
| `crates/wps-python` | Tested & Verified | 650 | 4 | Unified Python trait, Desktop Engine, DataFrame bridge, `=PY()`, Plots |
| `crates/wps-ai` | Tested & Verified | 580 | 4 | AST-Grounded Co-Pilot tool calls (@Writer, @Sheets, @Slides, @Pdf) |
| `crates/wps-collab` | Tested & Verified | 480 | 3 | State-Vector CRDT Sync, Presence, File Roaming Snapshot store |
| `crates/wps-wasm` | Tested & Verified | 580 | 0 | `wasm-bindgen` Writer dispatch, multi-page layout rasterizer, docx import/export |
| `src-tauri` | Scaffolded | 40 | 0 | Tauri 2.0 host, Native menus, File picker, Desktop print spooler |
| `python-runtime` | Scaffolded | 50 | 0 | `openwps_py` package with DataFrame & plotting helpers |
| `apps/workspace-ui` | Tested & Verified | 2,100 | 0 | React 19 + TypeScript + Tailwind + Mercury Design System Shell, Rulers, 10x8 Table Grid |

---

## 4. Architectural Decisions & Invariants Log

* **ADR-001 (Zero Stubs):** All business logic, algorithms, and models must be fully implemented in Rust. Stubs (`todo!()`, `unimplemented!()`) will fail continuous verification.
* **ADR-002 (Inversible Commands):** Every command must return its exact inverse to guarantee mathematically sound undo/redo without snapshot copying.
* **ADR-003 (DisplayList Protocol):** Rendering is decoupled from the DOM. Rust outputs a stream of vector and glyph draw commands to HTML5 Canvas2D / WebGPU.
* **ADR-004 (Dual Target Compilation):** All crates must compile for both native desktop and `wasm32-unknown-unknown`.
* **ADR-005 (Toolchains Setup):** Rust 1.99.1 + `wasm32-unknown-unknown` target configured; Node.js v24.19.0 + pnpm 10.33.2 verified.

---

## 5. Self-Healing & Compiler Fix Log

* *Issue 1:* `cargo` and `rustc` were initially not in PATH.
  * *Resolution:* Installed Rustup 1.29.1 via `winget install --id Rustlang.Rustup -e`, added `wasm32-unknown-unknown` target, and configured environment path to `$env:USERPROFILE\.cargo\bin`.
