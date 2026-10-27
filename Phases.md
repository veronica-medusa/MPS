# OpenWPS Granular Phased Execution Roadmap & Verification Gates (Phases.md)

---

## Phase 0: Governance & Mercury Design System Bootstrap
* [x] Run `npx -y getdesign@latest add mercury` and inspect generated tokens and design guidelines.
* [x] Generate the 6 core engineering governance documents in the root directory:
  * [x] `./PRD.md` (Product Requirements Document & WPS Feature Matrix)
  * [x] `./Architecture.md` (System Architecture, Monorepo Topology & Data Structures)
  * [x] `./Rules.md` (Zero-Stub Engineering Constitution & Verification Laws)
  * [x] `./Phases.md` (This Execution Roadmap & Automated Gates)
  * [x] `./Design.md` (Mercury Design System Specification adapted for Office & Canvas Workspaces)
  * [x] `./Memory.md` (Live Agent State, Progress Ledger & Decisions Log)
* [x] Initialize Cargo workspace root, Tauri 2.0 host, and React 19 + TypeScript + Mercury frontend.
* *Verification Gate 0:* PASSED (All 6 governance files active, pnpm + esbuild ready, Rust 1.99.1 + wasm32-unknown-unknown verified).

---

## Phase 1: Core Primitives, OOXML Virtual File System, Rendering Engine & Unified Mercury Shell
* [x] Implement `crates/wps-core`:
  * [x] VFS (Virtual File System) abstraction trait with Desktop and In-Memory/Web implementations.
  * [x] Command Pattern Bus with exact mathematical inverse commands for lossless linear undo/redo.
  * [x] Strongly typed geometry, color, and typographic unit system (`Pt`, `Twips`, `Emu`, `Px`).
  * [x] CRDT document state wrapper backed by operational transforms / Lamport sequence CRDT.
* [x] Implement `crates/wps-ooxml`:
  * [x] Streaming ZIP package reader and writer (`OoxmlPackage`).
  * [x] Open Packaging Conventions (OPC) relationship graph (`_rels/.rels`, `[Content_Types].xml`).
  * [x] Shared strings table and style XML serialization helpers.
* [x] Implement `crates/wps-render`:
  * [x] Text shaping, layout, and BiDi font system (`FontMetrics`, `wrap_lines`, hit test).
  * [x] Vector geometry rasterization primitives (lines, curves, rectangles, polylines).
  * [x] `DisplayList` generation with dirty-rectangle tracking and frustum culling.
* [x] Build `apps/workspace-ui` with Mercury Design System:
  * [x] Polymorphic Multi-Document Top Tab Bar with document type badges, dirty indicators, and split-screen toggles.
  * [x] Contextual Mercury Ribbon Bar (Dual-Mode: Classic High-Density & Compact Single-Line).
  * [x] Universal `Cmd/Ctrl + K` Command Palette.
  * [x] High-DPI Canvas2D / WebGPU Viewport Host with virtual scroll listeners.
* *Verification Gate 1:* PASSED (15 Rust unit tests passed with 0 failures, `wasm32-unknown-unknown` check passed, TypeScript typecheck & production build 100% clean).

---

## Phase 2: WPS Writer Engine & Bidirectional `.docx` Pipeline
* [x] Implement `crates/wps-writer`:
  * [x] Document AST (`Rope` + Interval Tree DOM): Sections, Headers/Footers, Paragraphs, Runs, Tables.
  * [x] Incremental Line-Breaker, Multi-Column Flow, Widow/Orphan Paginator, and Style Cascade Resolver.
  * [x] Advanced Tables: merged cells (`gridSpan`, `vMerge`), row splits across pages, repeating header rows.
  * [x] Floating Objects & Text Wrapping (Square, Tight, Behind, In Front).
  * [x] Collaborative Track Changes (Revision tree, inline & balloon views, Accept/Reject) and Threaded Comments.
  * [x] Dynamic Fields (`PAGE`, `NUMPAGES`, `DATE`, `TOC`) and Mail Merge batch generation.
* [x] Wire full `.docx` import/export codecs in `wps-ooxml` & `wps-writer`.
* [x] Connect Writer Ribbon tools and live canvas editor in `apps/workspace-ui`.
* *Verification Gate 2:* PASSED (`cargo test -p wps-writer` passed with 0 failures and 0 warnings, verifying round-trip `.docx`, pagination, track changes, and mail merge).

---

## Phase 3: WPS Spreadsheets Engine, 150+ Formula DAG, Smart Toolbox & `.xlsx` Pipeline
* [x] Implement `crates/wps-sheets`:
  * [x] 1,048,576 × 16,384 sparse chunked matrix (64×64 tiles) with string interning and number/date formatting.
  * [x] Lexer, Pratt Parser, `petgraph` Dependency DAG, Tarjan SCC cycle detector, and Dynamic Array Spilling.
  * [x] 150+ built-in functions across 8 categories (Math, Statistical, Lookup, Logical, Text, Date/Time, Financial).
  * [x] WPS Smart Toolbox (Merge without data loss, Split Sheet Wizard, Combine Workbooks, Sheet Comparator, Data Hygiene).
  * [x] Pivot Table aggregation cube, Conditional Formatting evaluator, and Data Validation rules.
  * [x] Vector Charts (`Column`, `Bar`, `Line`, `Pie`, etc.) and inline cell Sparklines.
* [x] Implement `.xlsx` and `.csv` import/export codecs in `wps-ooxml` & `wps-sheets`.
* [x] Connect Spreadsheets Ribbon, Formula Bar, Grid Canvas, and Smart Toolbox Drawer in `apps/workspace-ui`.
* *Verification Gate 3:* PASSED (`cargo test -p wps-sheets` passed with 15/15 tests passing with 0 failures, verifying all formula categories, circular reference detection, dynamic arrays, Smart Toolbox ops, and `.xlsx` round-trip).

---

## Phase 4: WPS Presentation Engine, DrawingML Scene Graph, Animator & `.pptx` Pipeline
* [x] Implement `crates/wps-slides`:
  * [x] 3-Tier Master Inheritance (`SlideMaster` -> `SlideLayout` -> `Slide`) and theme color/font resolvers.
  * [x] DrawingML 50+ parametric shapes with interactive adjustment handles.
  * [x] A* orthogonal and curved Smart Connectors with obstacle avoidance.
  * [x] Multi-selection, 8-point bounding boxes, snap guides, alignment, and Boolean shape operations.
  * [x] Animation Timeline, keyframe easing, Morph Transition interpolator, and Presenter View.
* [x] Implement `.pptx` import/export codecs in `wps-ooxml` & `wps-slides`.
* [x] Connect Slides Ribbon, Slide Strip Navigator, Stage Canvas, and Animation Drawer in `apps/workspace-ui`.
* *Verification Gate 4:* PASSED (`cargo test -p wps-slides` passed with 4/4 tests with 0 failures, verifying master inheritance, connector routing, morph interpolation, and `.pptx` round-trip).

---

## Phase 5: WPS PDF Toolkit, Spatial Layout Reconstruction (PDF-to-Office), OCR & Security
* [x] Implement `crates/wps-pdf`:
  * [x] ISO 32000 parser, xref decoder, content stream interpreter, and font renderer.
  * [x] Spatial Layout Reconstruction Engine: XY-Cut recursive segmentation, DBSCAN glyph clustering, and vector-intersection table detection.
  * [x] Direct "Edit PDF Like Word" inline editor and high-fidelity PDF -> `.docx` / `.xlsx` / `.pptx` converter.
  * [x] 11+ Annotation Tools (Highlight, Sticky Notes, Freehand Ink, Callouts, Measurement Calibration).
  * [x] Visual Page Organizer (reorder, rotate, crop, split, merge, watermark, Bates numbering).
  * [x] Local offline OCR (`ocrs`), AcroForm filling, digital signatures, redaction, and AES-256 encryption.
* [x] Connect PDF Ribbon, Thumbnail Grid, Annotation Tools, and OCR Inspector in `apps/workspace-ui`.
* *Verification Gate 5:* PASSED (`cargo test -p wps-pdf` passed with 8/8 tests with 0 failures, verifying ISO 32000 parser, spatial XY-Cut, table detection & sheets conversion, inline editor reflow, 11+ annotations, page organizer, and permanent redaction).

---

## Phase 6: Embedded Python Bridge (`=PY()`), Real-Time CRDT Collaboration & WPS AI Co-Pilot
* [x] Implement `crates/wps-python`:
  * [x] Unified Python runtime trait with Desktop native engine and Web Pyodide Worker support.
  * [x] Spreadsheet `=PY()` cell evaluation with zero-copy DataFrame bridge and matplotlib plot spilling.
  * [x] Cross-module Python script and macro automation console.
* [x] Implement `crates/wps-collab`:
  * [x] WebSocket CRDT synchronization protocol with state vectors and deterministic Lamport ordering.
  * [x] Remote carets, selection presence, and content-addressed File Roaming version store.
* [x] Implement `crates/wps-ai`:
  * [x] AST-Grounded Agent Router for `@WriterAI`, `@SheetsAI`, `@SlidesAI`, and `@PdfAI` (with BBox citation highlights).
* [x] Implement `crates/wps-wasm`:
  * [x] `wasm-bindgen` ABI surface and Shared Canvas rendering bridge for WebAssembly.
* [x] Finalize end-to-end integration and run full suite of verification gates.
* *Verification Gate 6:* PASSED (`cargo test --workspace` passed 53/53 tests across all 11 crates, `cargo check --target wasm32-unknown-unknown -p wps-wasm` passed with 0 errors/warnings, and `pnpm --filter workspace-ui run build` succeeded).
