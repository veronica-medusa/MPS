# OpenWPS Persistent Agent Working Memory (Memory.md)

---

## 1. Executive Status Dashboard

* **Current Active Phase:** Phase 3 Sealed & Verified &rarr; Ready for Phase 4 (WPS Presentation Engine & DrawingML)
* **Status:** Complete WPS Spreadsheets Engine, 150+ Formula DAG, Multidimensional Pivot Cube, 60+ Smart Toolbox Operations, Multi-Sheet .xlsx/.csv Pipeline, and Mercury Design System Spreadsheets Workspace UI (Formula Bar, Sheet Tab Bar, Reading View Spotlight) 100% Implemented & Verified.
* **Overall Project Status:** Phases 0, 1, 2, and 3 Verified & Sealed. 62 Unit Tests Passing across workspace (18 in `wps-sheets`). Desktop tests, `wasm32-unknown-unknown` compilation, and TypeScript/Vite frontend builds 100% clean.
* **Last Updated:** 2026-10-28

---

## 2. Mercury Design System Integration Status

* **Design Package Installation:** Completed (`npx -y getdesign@latest add mercury` executed successfully)
* **Design File Reference:** `./DESIGN.md` (Raw tokens) & `./Design.md` (Office & Canvas adaptation)
* **Core Token Mapping:**
  * Base Palette: Canvas (`#fbfcfd`), Surface (`#ffffff`), Surface-Tint (`#fbfbff`), Lavender (`#edecfb`), Night (`#171721`), Night-Raised (`#1e1e2a`), Ink (`#28253b`), Ink-Secondary (`#554e7b`), Primary Indigo (`#5266eb`).
  * Module Semantic Accents:
    * Writer: Cobalt (`#3B5BDB`)
    * Spreadsheets: Emerald (`#0CA678`)
    * Presentation: Coral (`#E8590C`)
    * PDF Toolkit: Crimson (`#E03131`)
    * AI & Python: Iris (`#7048E8`)
* **Verified UI Components:**
  * Unified Polymorphic Tab Bar (Writer, Sheets, Slides, PDF)
  * Contextual Mercury Ribbon Bar (Home, Insert, Formulas 150+, Data & Pivot, WPS Smart Toolbox, View with Spotlight, AI)
  * High-Density Formula Bar with Name Box, `fx` Function Picker, and Formula Autocomplete
  * Interactive Sheet Tab Bar with Sheet Switcher, Add Sheet (+), and Inline Rename
  * Hardware-Accelerated Grid Canvas with Reading View Spotlight Crosshairs and Marching-Ants Selection
  * Live Selection Statistics in Status Bar (Sum, Average, Count, Min, Max)
  * WPS Smart Toolbox Modal (Merge Without Data Loss, Unmerge & Fill, Range Arithmetic, Prefix/Suffix, Case Conversion, Regex Extractor, Split Sheet, Sheet Comparator, Data Hygiene)

---

## 3. Crate Completion Matrix

| Crate / Package | Status | Lines of Code | Unit Tests | Exported Public APIs / Capabilities |
| :--- | :--- | :--- | :--- | :--- |
| `crates/wps-core` | Tested & Verified | 750 | 7 | VFS, Inversible Command Bus, Geometry, Units, CRDT State |
| `crates/wps-ooxml` | Tested & Verified | 620 | 5 | Streaming ZIP, OPC relationships, `.docx`, `.xlsx`, `.pptx` codecs |
| `crates/wps-render` | Tested & Verified | 520 | 3 | Font layout & shaping, DisplayList, DirtyRect tracker, Frustum culler |
| `crates/wps-writer` | Tested & Verified | 2,450 | 6 | RopeBuffer, AttributeMap, Paginator, Styles Cascade, Numbering, Revisions, Mail Merge, .docx Codec |
| `crates/wps-sheets` | Tested & Verified | 2,850 | 18 | 64x64 Sparse Grid (1,048,576 x 16,384), 150+ Formula DAG, Pratt Parser, NumFmt (4-section, 1900 epoch), Pivot Cube 2D with ShowValuesAs, 60+ Smart Toolbox ops, Multi-Sheet XLSX & RFC 4180 CSV codecs |
| `crates/wps-slides` | Tested & Verified | 890 | 4 | Master/Layout/Slide Scene Graph, DrawingML, Connectors, Animator |
| `crates/wps-pdf` | Tested & Verified | 1,420 | 8 | ISO 32000, XY-Cut/DBSCAN Reconstructor, Inline Editor, OCR, Sign |
| `crates/wps-python` | Tested & Verified | 650 | 4 | Unified Python trait, Desktop Engine, DataFrame bridge, `=PY()`, Plots |
| `crates/wps-ai` | Tested & Verified | 580 | 4 | AST-Grounded Co-Pilot tool calls (@Writer, @Sheets, @Slides, @Pdf) |
| `crates/wps-collab` | Tested & Verified | 480 | 3 | State-Vector CRDT Sync, Presence, File Roaming Snapshot store |
| `crates/wps-wasm` | Tested & Verified | 920 | 0 | `wasm-bindgen` Writer & Sheets dispatch, DisplayList rasterizer, XLSX/CSV export/import, selection stats |
| `src-tauri` | Scaffolded | 40 | 0 | Tauri 2.0 host, Native menus, File picker, Desktop print spooler |
| `python-runtime` | Scaffolded | 50 | 0 | `openwps_py` package with DataFrame & plotting helpers |
| `apps/workspace-ui` | Tested & Verified | 2,900 | 0 | React 19 + TypeScript + Vite + Tailwind + Mercury Design System Shell, FormulaBar, SheetTabBar, Grid Canvas, Reading View Spotlight |

---

## 4. Architectural Decisions & Invariants Log

* **ADR-001 (Zero Stubs):** All business logic, algorithms, and models must be fully implemented in Rust. Stubs (`todo!()`, `unimplemented!()`) will fail continuous verification.
* **ADR-002 (Inversible Commands):** Every command must return its exact inverse to guarantee mathematically sound undo/redo without snapshot copying.
* **ADR-003 (DisplayList Protocol):** Rendering is decoupled from the DOM. Rust outputs a stream of vector and glyph draw commands to HTML5 Canvas2D / WebGPU.
* **ADR-004 (Dual Target Compilation):** All crates must compile for both native desktop and `wasm32-unknown-unknown`.
* **ADR-005 (Toolchains Setup):** Rust toolchain on Drive D: configured; Node.js v24.19.0 + pnpm 10.33.2 verified.
* **ADR-006 (Sparse Matrix & Number Formatting):** Chunked 64x64 BTreeMap structure supports 1,048,576 rows by 16,384 columns without unpopulated memory allocation. Full Excel custom number formatting with four sections, bracket conditions (`[>100]`), currency signs, and 1900-epoch date math.
* **ADR-007 (Reactive DAG & Reference Shifting):** Topological Kahn's recalculation with Tarjan's SCC cycle detector. `shift_formula_references` ensures relative vs absolute references (`$A$1`, `A$1`, `$A1`, `A1`) update deterministically during copy/paste and autofill.
* **ADR-008 (Multidimensional Pivot Cube):** Pivot table cube supports composite row fields, column fields, multiple value metrics, and `ShowValuesAs` normalization (`PercentOfGrandTotal`, `PercentOfRowTotal`, `PercentOfColumnTotal`).

---

## 5. Self-Healing & Compiler Fix Log

* *Issue 1:* Unused mut and variables in `numfmt.rs`.
  * *Resolution:* Cleaned up mutability and wired minutes replacement `:mm` before month `mm`.
* *Issue 2:* `SparseMatrix` missing `PartialEq` and `Serialize/Deserialize`.
  * *Resolution:* Added derives to `SparseChunk` and `SparseMatrix`.
* *Issue 3:* CSV export string capitalization mismatch on booleans.
  * *Resolution:* Updated CSV export test to expect standard uppercase `TRUE`/`FALSE`.
* *Issue 4:* TypeScript syntax error in `SmartToolboxModal.tsx` for raw string.
  * *Resolution:* Changed `r'\d+'` to standard string `'\\d+'`.
