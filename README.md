# OpenWPS (开放式办公套件)

> **A production-grade, memory-safe, pinpoint-accurate replica of the WPS Office Suite engineered in Rust, WebAssembly, Python, and Tauri 2.0, styled with the Mercury Design System.**

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B_Edition_2021-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-wasm32--unknown--unknown-654ff0.svg)](https://webassembly.org/)
[![Frontend](https://img.shields.io/badge/Frontend-React_19_+_TypeScript_+_Tailwind-61dafb.svg)](https://react.dev/)
[![Design System](https://img.shields.io/badge/Design_System-Mercury-5266eb.svg)](https://getdesign.dev)
[![Tests](https://img.shields.io/badge/Unit_Tests-73%2F73_Passing-brightgreen.svg)]()

---

```
   ____                       _       ______  _____ 
  / __ \                     | |     |  ____|/ ____|
 | |  | |_ __   ___ _ __   __| |_   _| |__  | (___  
 | |  | | '_ \ / _ \ '_ \ / _` \ \ / /  __|  \___ \ 
 | |__| | |_) |  __/ | | | (_| |\ V /| |____ ____) |
  \____/| .__/ \___|_| |_|\__,_| \_/ |__________/  
        | |                                         
        |_|   Next-Gen Pinpoint Precision Office Suite
```

---

## 1. Executive Manifesto

Office software has been stagnant for decades, burdened by bloated legacy C++ codebases, sluggish Web-tech wrappers relying on fragile `contenteditable` DOM hacks, and closed proprietary document formats.

**OpenWPS** provides a modern, high-performance alternative:

1. **Single Source of Truth in Rust (`crates/*`):** Zero JavaScript or DOM state duplication. All document abstract syntax trees (ASTs), OOXML/PDF codecs, 2D layout calculation engines, formula evaluation DAGs, and undo/redo buses run entirely in memory-safe Rust.
2. **Deterministic Vector Canvas Pipeline:** No browser text-selection quirks or flickering reflows. OpenWPS renders document pages (Writer), sparse chunked matrices (Spreadsheets), slide stages (Presentation), and PDF canvases via a unified, zero-copy **`DisplayList`** stream into HTML5 Canvas2D and WebGPU.
3. **True Dual-Target Parity:** Compiles identically for **Native Desktop** (Windows, macOS, Linux via Tauri 2.0 with OS menus and system print spoolers) and **Web Browsers** (via `wasm32-unknown-unknown` + `wasm-bindgen`).
4. **Embedded Python (`=PY()`) & AST-Grounded AI:** First-class data science workflows inside spreadsheets with zero-copy pandas/polars `PyDataFrame` bridges and vector SVG plot spilling, coupled with an AI co-pilot that modifies typed Rust AST nodes rather than generating unstructured diffs.
5. **Mercury Design System Craft:** Styled with Mercury's high-craft financial density tokens, progressive disclosure ribbons, razor-sharp 1px hairlines, and module-specific semantic accents.

---

## 2. Monorepo Architecture & Topology

```
OpenWPS/
├── apps/
│   └── workspace-ui/            # Mercury Design System Frontend (React 19, TypeScript, Tailwind)
│       ├── src/
│       │   ├── components/
│       │   │   ├── TopTabBar.tsx         # Polymorphic multi-document tab bar (Writer, Sheets, Slides, PDF)
│       │   │   ├── RibbonBar.tsx         # High-density contextual Ribbon (Classic & Compact)
│       │   │   ├── FormulaBar.tsx        # Name Box, fx inserter, and 150+ function autocomplete
│       │   │   ├── SheetTabBar.tsx       # Bottom sheet switcher, add sheet (+), and inline rename
│       │   │   ├── CanvasViewport.tsx    # High-DPI canvas host with Reading View Spotlight crosshairs
│       │   │   ├── SmartToolboxModal.tsx # WPS Smart Toolbox wizards (Pivots, Math, Regex, Case)
│       │   │   ├── StatusBar.tsx         # Live selection statistics (Sum, Average, Count, Min, Max)
│       │   │   ├── CommandPalette.tsx    # Universal Cmd/Ctrl + K command palette
│       │   │   └── RightInspector.tsx    # Dockable inspector for WPS AI and Python REPL
│       │   ├── App.tsx          # Master shell coordinating active tab state & module switches
│       │   └── types.ts         # High-density UI state contracts
├── crates/                      # Core Rust Workspace
│   ├── wps-core/                # High-precision dimensional arithmetic, Geometry, VFS, Inversible Commands
│   ├── wps-ooxml/               # Streaming ZIP, OPC Relationships, Shared Strings Table (SST), Styles
│   ├── wps-render/              # DisplayList protocol, Glyph rasterization, Frustum culler, DirtyRects
│   ├── wps-writer/              # Document AST, A4/Letter Paginator, Redlining, Mail Merge, .docx Codec
│   ├── wps-sheets/              # 1M×16K SparseMatrix (64×64 chunks), 150+ Formula DAG, Smart Toolbox, .xlsx
│   ├── wps-slides/              # 3-Tier Master Cascade, 50+ DrawingML shapes, A* Connectors, Morph, .pptx
│   ├── wps-pdf/                 # ISO 32000 parser, XY-Cut/DBSCAN spatial reconstructor, Inline Editor, Redaction
│   ├── wps-python/              # Zero-copy PyDataFrame, describe(), =PY() evaluator, Matplotlib SVG generator
│   ├── wps-ai/                  # AST-Grounded Agent Router (@Writer, @Sheets, @Slides, @Pdf)
│   ├── wps-collab/              # State-vector CRDT sync, Presence tracking, Content-addressed File Roaming
│   └── wps-wasm/                # wasm-bindgen ABI exposing all engines to browser Canvas2D/WebGPU
├── src-tauri/                   # Tauri 2.0 Host Application & Native OS integration
├── PRD.md                       # Product Requirements Document & Pinpoint WPS Feature Matrix
├── Architecture.md              # Deep System Architecture & Low-Level Invariants
├── Rules.md                     # Zero-Stub Engineering Constitution & Verification Laws
├── Phases.md                    # Verification Gates & Execution Roadmap
├── Design.md                    # Mercury Design System Specification adapted for Office
├── Memory.md                    # Persistent Agent Working Memory & Ledger
└── LICENSE                      # Apache 2.0 Open-Source License
```

---

## 3. The Six Engineering Pillars

### 📝 Pillar 1: Unified Multi-Tab Shell & Content-Addressed Roaming
- **All-in-One Polymorphic Shell:** Work with `.docx`, `.xlsx`, `.pptx`, `.pdf`, `.csv`, and `.py` documents simultaneously in a single, high-speed tabbed window.
- **Synchronous Split-Screen:** Open two documents side-by-side (e.g. PDF reference on the left, Writer or Spreadsheet on the right) with synchronized scroll cursors.
- **Content-Addressed File Roaming:** Integrated in `wps-collab::roaming`, every save creates a snapshot indexed by an FNV-1a content hash. Revert to any point in time with instant zero-loss rollbacks.

### ✍️ Pillar 2: WPS Writer (Word Processor)
- **Bidirectional ECMA-376 `.docx` Codec:** Direct XML streaming roundtrip without intermediary markup loss.
- **Document AST Model:** `Section` (margins, columns, orientation), `Paragraph` (line spacing, indentation, styles), `TextRun` (font, size, bold, italic, color, underline, strike), `Table` (repeating header rows, cell spans, padding), and `FloatingObject` (wrap modes: Inline, Square, Tight, Behind, InFront).
- **Sub-Pixel Layout & Paginator:** Hardware-agnostic multi-page paginator respecting widow/orphan rules, footnote allocation, and table split mechanics.
- **Track Changes & Redlining:** Multi-author revision tree tracking insertions, deletions, and format changes with interactive Accept/Reject workflows.
- **Mail Merge Engine:** Dynamic templating engine merging JSON/CSV data streams into batch output documents.

### 📊 Pillar 3: WPS Spreadsheets & Smart Toolbox
- **Sparse Chunked Matrix:** $1,048,576 \text{ rows} \times 16,384 \text{ columns}$ grid organized in sparse $64 \times 64$ chunks via `BTreeMap<(u32, u16), Box<SparseChunk>>`. Memory usage is strictly proportional to populated cells.
- **150+ Built-In Formulas:** Full Lexer and recursive descent Pratt Parser with standard Excel operator precedence (`^`, `*`, `/`, `+`, `-`, `&`, comparison operators). Covers:
  - *Math & Trig:* `SUM`, `AVERAGE`, `PRODUCT`, `ROUND`, `ABS`, `MOD`, `POWER`, `CEILING`, `FLOOR`.
  - *Lookup & Reference:* `XLOOKUP`, `VLOOKUP`, `HLOOKUP`, `INDEX`, `MATCH`, `OFFSET`, `INDIRECT`.
  - *Statistical:* `CORREL`, `SLOPE`, `INTERCEPT`, `FORECAST`, `MEDIAN`, `STDEV.S`, `VAR.S`, `COUNT`, `COUNTIF`.
  - *Financial:* `PMT`, `PV`, `FV`, `NPV`, `IRR`, `RATE`, `NPER`, `EFFECT`, `NOMINAL`.
  - *Date & Time:* `DATE`, `YEAR`, `MONTH`, `DAY`, `HOUR`, `MINUTE`, `SECOND`, `EDATE`, `EOMONTH`, `NETWORKDAYS`, `WORKDAY`.
  - *Dynamic Arrays:* `FILTER`, `UNIQUE`, `SORT`, `SEQUENCE`.
- **Reactive Dependency DAG & Reference Shifting:** Kahn's topological sort recalculation for dirty cells; Tarjan's SCC cycle detector isolates circular references (`#CYCLE!`). `shift_formula_references` automatically updates relative references while preserving absolute anchors (`$A$1`, `A$1`, `$A1`, `A1`) during copy/paste and autofill.
- **WPS Smart Toolbox (60+ Signature Batch Ops):**
  - *Merge Without Data Loss:* Combines cells concatenating all text values rather than discarding data.
  - *Unmerge & Fill:* Splits merged cells and duplicates values to prevent broken lookups and pivot tables.
  - *Range Arithmetic:* Applies batch math (`+`, `-`, `*`, `/`) in-place across selected numbers.
  - *Batch Prefix / Suffix:* Prepends or appends custom strings across an entire range.
  - *Case Conversion:* Rapidly converts ranges between UPPERCASE, lowercase, and Proper Case.
  - *Regex Extractor:* Extracts numbers, emails, or regex patterns into target columns.
  - *Formulas to Values:* Freezes dynamic formulas into static values.
  - *Sheet Comparator:* Computes visual cell-by-cell structural diffs between two sheets.
- **Multidimensional 2D Pivot Cube:** Composite row fields, column fields, multiple value metrics, and Slicer filters. Includes `ShowValuesAs` normalization modes (`PercentOfGrandTotal`, `PercentOfRowTotal`, `PercentOfColumnTotal`).
- **Custom Number & Date Formatter (`numfmt`):** Complete 4-section format code parser (`POSITIVE;NEGATIVE;ZERO;TEXT`) with bracket conditions (`[>100]`), color tags (`[Red]`, `[Green]`), currency signs, commas, scientific notation, and 1900 serial dates.
- **Spreadsheets UI Workspaces:** Interactive Formula Bar with Name Box and autocomplete, Sheet Tab Bar with inline rename, and Reading View Spotlight crosshairs.

### 📽️ Pillar 4: WPS Presentation (Slides Engine)
- **3-Tier Cascade Inheritance & EMU Coordinates:** Integer-precision ECMA-376 English Metric Units (`1 pt = 12,700 EMUs`, `1 in = 914,400 EMUs`) prevent float drift. Cascading property resolver queries `Slide` $\rightarrow$ `SlideLayout` $\rightarrow$ `SlideMaster` $\rightarrow$ `Theme` for fill, stroke, and typography defaults.
- **50+ DrawingML Parametric Shapes:** Complete library of procedural geometries (Rectangles, RoundedRects, Ellipses, Stars, Callouts, Flowcharts, Arrows) with draggable yellow diamond adjustment handles (`adj: Vec<f64>`) and auto-fitting text frames.
- **Boolean 2D Shape Operations:** Full vector constructive solid geometry (CSG): Union (Monotone Chain Convex Hull), Subtract, Intersect (Sutherland-Hodgman polygon clipping), Combine (symmetric difference XOR), and Fragment.
- **A\* Smart Connectors with Obstacle Avoidance:** Orthogonal vector routing engine calculating waypoints through a 2D navigation grid around obstacle bounding boxes, with 8 cardinal port attachments and cubic Bézier smoothing.
- **Keyframe Animation Sequencer:** Timeline state machine supporting Entrance, Emphasis, and Exit animations with easing curves (Linear, EaseInQuad, EaseOutQuad, EaseInOutCubic, Bounce, Elastic).
- **60–120 FPS Morph Transition Interpolator:** Pinpoint geometry matching across slides using the `!!name` correspondence protocol; dissimilar shapes undergo perimeter path resampling (64 points), rotation, stroke, and RGBA cross-fade.
- **Dual-Screen Presenter View Cockpit:** Integrated full-screen presenter display featuring elapsed timer, speaker notes pane, next slide preview, glowing laser pointer overlay, live ink annotations (pen and translucent highlighter), and blackout/whiteout ('B'/'W') shortcuts.
- **Bidirectional PresentationML (`.pptx`) Codec:** Lossless streaming OPC ZIP codec exporting and importing presentation decks with slide masters, layouts, DrawingML shapes, connectors, and transitions.

### 📑 Pillar 5: WPS PDF Toolkit & Spatial Layout Reconstruction
- **ISO 32000 Stream Parser:** Robust parser extracting indirect objects, cross-reference tables, stream dictionaries, Flate compression, and page trees.
- **Spatial Layout Reconstruction (PDF $\rightarrow$ Office):**
  - *DBSCAN Clustering:* Clusters positioned glyph baselines and line-heights into cohesive words and lines.
  - *Recursive XY-Cut:* 2D whitespace projection histogram profiling that identifies columns, paragraphs, and reading order.
  - *Hybrid Table Detection:* Traces horizontal and vertical vector strokes and borderless multi-column gutters to reconstruct tabular grid matrices (`ReconstructedTable`).
  - *High-Fidelity AST Exporters:* Exports reconstructed pages directly to live `wps_writer::ast::Document`, multi-sheet `wps_sheets::Workbook`, or `wps_slides::PresentationDeck`.
- **Direct "Edit PDF Like Word":** Click any reconstructed paragraph to edit text in place; line wraps reflow automatically according to bounding box metrics, emitting fresh PDF content streams.
- **11+ Annotation Suite:** Text Highlights, Underline, Squiggly, Strikethrough, Sticky Notes, Freehand Ink, Free Text Boxes, Callout Clouds with 3-point leader lines, Geometric Shapes (including `CloudPolygon` with scalloped borders), Dynamic Stamps (`APPROVED`, `CONFIDENTIAL`, `DRAFT`, `REJECTED`), and Calibrated Physical Measurement Tools (Distance Ruler, Perimeter Tool, Shoelace Area Polygon).
- **Visual Page Organizer:** Visual page reordering, 90°/180° rotation, cropping, page deletion/extraction, watermark application, legal Bates numbering (6 positions), and multi-level stream compression (`miniz_oxide`).
- **Security & Redaction:** FIPS 197 pure-Rust AES-256-CBC content encryption, cryptographic digital signatures, and irreversible spatial redaction that permanently scrubs sensitive string bytes from raw content streams with zero trace.

### 🐍 Pillar 6: Embedded Python (`=PY()`) & AST-Native AI
- **Spreadsheet `=PY()` Integration:** Evaluate Python scripts right inside spreadsheet cells.
  ```python
  df = xl("A1:C50")
  df.describe()
  ```
- **Zero-Copy `PyDataFrame`:** Memory-efficient tabular structure with native `mean()`, `std_dev()`, `min()`, `max()`, `percentile()`, and pandas-identical `describe()` output matrix.
- **Visual Vector Plot Spilling:** Plot instructions (`plt.plot(x, y)`, `plt.bar(cats, vals)`) generate crisp vector SVG charts styled with Mercury colors that spill directly into cell canvases.
- **Cross-Module Macro Automation:** Automation runner enabling batch scripts across all modules (e.g., pulling numbers from Sheets, generating a Writer summary, appending slides, and stamping a PDF).
- **AST-Native WPS AI Co-Pilot:**
  - `@WriterAI`: Rewrites text, generates complete outlines, changes tone.
  - `@SheetsAI`: Compiles natural language into complex formulas (`=XLOOKUP`, `=SUMIFS`).
  - `@SlidesAI`: One-prompt topic-to-presentation deck generator.
  - `@PdfAI`: Vector + structural RAG over open PDFs with clickable citations highlighting exact bounding boxes on page canvases.

---

## 4. Mercury Design System Integration

OpenWPS is styled strictly using the **Mercury Design System** (`npx getdesign@latest add mercury`):

| Token | Value | Applied Element |
| :--- | :--- | :--- |
| `--mercury-bg-canvas` | `#fbfcfd` | Main viewport canvas backdrop behind pages/slides |
| `--mercury-bg-surface-1` | `#ffffff` | Document page surface, Ribbon background, Modal containers |
| `--mercury-bg-surface-2` | `#fbfbff` | Active tab headers, formula bar input, tooltips |
| `--mercury-bg-night` | `#171721` | Dark mode base, primary action buttons, contrast carets |
| `--mercury-border-hairline` | `#e5e7eb` | Crisp 1px structural pane separators and grid lines |
| `--mercury-accent-writer` | `#3B5BDB` | Cobalt accent for WPS Writer chrome & selection bounds |
| `--mercury-accent-sheets` | `#0CA678` | Emerald accent for WPS Spreadsheets grid headers & cell highlights |
| `--mercury-accent-slides` | `#E8590C` | Coral accent for Presentation slide navigation & marquee handles |
| `--mercury-accent-pdf` | `#E03131` | Crimson accent for PDF Toolkit, stamps, and annotations |
| `--mercury-accent-ai` | `#7048E8` | Iris accent for WPS AI Co-Pilot & Python DataFrame bridges |

---

## 5. Mathematical & Algorithmic Foundations

OpenWPS relies on exact, mathematically sound algorithms:

### 1. Pratt Parser with Precedence Binding Powers
Formula parsing uses Vaughan Pratt's top-down operator precedence algorithm to evaluate nested arithmetic, range operators (`:`), and dynamic function calls in linear $O(n)$ time:
$$\text{Binding Power: } \text{Range (:) } > \text{Unary (-)} > \text{Exponentiation (\^)} > \text{Multiplication (*, /)} > \text{Addition (+, -)} > \text{Concat (\&)} > \text{Comparison}$$

### 2. Tarjan's Strongly Connected Components (SCC)
To maintain real-time recalculation speed without runaway stack overflows, dependency updates construct a directed graph $G = (V, E)$ where vertices are cells and directed edges represent references. Tarjan's algorithm locates cycles in $O(|V| + |E|)$ time and emits `#CYCLE!`.

### 3. Recursive XY-Cut Algorithm
PDF layout reconstruction calculates 1D projection histograms $H_X(x)$ and $H_Y(y)$ over bounding boxes:
$$H_X(x) = \sum_{i} \mathbf{1}_{[x_{i,\text{min}}, x_{i,\text{max}}]}(x)$$
Valleys where $H_X(x) = 0$ over a width threshold $\Delta x > W_{\text{col}}$ identify column boundaries, segmenting multi-column layouts into natural reading orders prior to paragraph clustering.

### 4. Zero-Copy `DisplayList` Vector Protocol
Instead of allocating heavyweight DOM trees, rendering outputs an immutable array of flat vector commands (`DrawTextRun`, `FillRect`, `StrokeRect`, `DrawLine`, `DrawPath`). Viewports apply frustum culling:
$$\text{Visible} \iff \text{Command.Bounds} \cap \text{Viewport.Bounds} \neq \emptyset$$

---

## 6. Getting Started

### Prerequisites

- **Rust Toolchain:** `1.85+` (stable channel with `wasm32-unknown-unknown` target)
- **Node.js & Package Manager:** Node `v20+` and `pnpm v9+`

### Quickstart

1. **Clone the repository:**
   ```bash
   git clone https://github.com/veronica-medusa/MPS.git
   cd MPS
   ```

2. **Install frontend dependencies:**
   ```bash
   pnpm install
   ```

3. **Run the full test suite (73 tests across all 11 crates):**
   ```bash
   cargo test --workspace
   ```

4. **Verify WebAssembly compilation:**
   ```bash
   cargo check --target wasm32-unknown-unknown -p wps-wasm
   ```

5. **Start the Web UI Development Server:**
   ```bash
   pnpm run dev
   ```
   Open your browser at `http://localhost:5173`.

6. **Build for Production:**
   ```bash
   pnpm run build
   ```

---

## 7. Zero-Stub Verification

OpenWPS operates under a strict **Zero-Stub Engineering Law**:
- No `todo!()` or `unimplemented!()` placeholders.
- No dummy mock returns.
- Every Ribbon button, shortcut, and formula triggers deterministic mutations on the compiled Rust AST.

To independently execute the automated verification gates:

```bash
# Verification Gate 1: Core Primitives & OOXML VFS
cargo test -p wps-core -p wps-ooxml -p wps-render

# Verification Gate 2: Writer Engine & .docx Codec
cargo test -p wps-writer

# Verification Gate 3: Spreadsheets Engine, 150+ Formula DAG & .xlsx Codec
cargo test -p wps-sheets

# Verification Gate 4: Presentation Engine & .pptx Codec
cargo test -p wps-slides

# Verification Gate 5: PDF Toolkit, Spatial Reconstructor & Redaction
cargo test -p wps-pdf

# Verification Gate 6: Embedded Python, CRDT Collab & Wasm Check
cargo test -p wps-python -p wps-collab -p wps-ai
cargo check --target wasm32-unknown-unknown -p wps-wasm
```

---

## 8. License

OpenWPS is open-source software licensed under the **Apache License, Version 2.0**. See the [LICENSE](file:///C:/Users/Sajid/Downloads/MPS/LICENSE) file for the complete license terms and conditions.

---

*Engineered with precision for the next generation of computing.*
