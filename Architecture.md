# OpenWPS Deep System Architecture & Topology (Architecture.md)

---

## 1. Monorepo Topology & Crate Boundaries

```text
openwps/
├── PRD.md                          # Product Requirements Document
├── Architecture.md                 # System Architecture & Topology (This File)
├── Rules.md                        # Zero-Stub Engineering Constitution
├── Phases.md                       # Sequential Execution Roadmap & Verification Gates
├── Design.md                       # Mercury Design System Specification
├── Memory.md                       # Live Agent State & Progress Ledger
├── Cargo.toml                      # Rust Monorepo Workspace Manifest
├── package.json                    # Monorepo Workspace (pnpm)
├── pnpm-workspace.yaml             # pnpm workspace configuration
├── crates/
│   ├── wps-core/                   # VFS, Command Bus, Inverse Undo/Redo, Units, Color, Geometry, CRDT
│   ├── wps-ooxml/                  # Streaming ZIP + XML codecs (.docx, .xlsx, .pptx) & OLE2 (.doc)
│   ├── wps-render/                 # Typography (cosmic-text), Vector Paths, DisplayList, Viewport Frustum
│   ├── wps-writer/                 # Word Processor: Rope + Interval Tree DOM, Paginator, Redlines, Mail Merge
│   ├── wps-sheets/                 # Spreadsheet: 64x64 Sparse Matrix, Formula Parser, petgraph DAG, Smart Toolbox
│   ├── wps-slides/                 # Presentation: Master/Layout/Slide Scene Graph, DrawingML, Connectors, Animator
│   ├── wps-pdf/                    # PDF Toolkit: ISO 32000 Parser, Spatial XY-Cut/DBSCAN Reconstructor, Editor, OCR
│   ├── wps-python/                 # Python Bridge: Unified trait for Desktop PyO3 + Web Pyodide Worker (=PY())
│   ├── wps-ai/                     # AST-Grounded Agent Router for Writer, Sheets, Slides, and PDF RAG
│   ├── wps-collab/                 # WebSocket CRDT Sync Server/Client, Presence Protocol & Roaming Snapshots
│   └── wps-wasm/                   # wasm-bindgen ABI surface for Web & Shared Canvas rendering
├── src-tauri/                      # Tauri 2.0 Native Host (OS File System, Printing, Menus, Desktop PyO3)
├── python-runtime/                 # Embedded Python package (`openwps_py`: DataFrame bridge, Smart Toolbox)
└── apps/
    └── workspace-ui/               # React 19 + TypeScript + Tailwind + Mercury Design System Shell & Canvas Host
```

---

## 2. Zero-Copy Viewport Rendering Protocol

To achieve 120 FPS rendering without DOM or `contenteditable` overhead, OpenWPS implements a high-performance **DisplayList** protocol.

### 2.1 The Render Pipeline Architecture
```text
+-------------------------------------------------------+
|                 Document AST in Rust                  |
|     (Writer Rope, Sheets Sparse Grid, Slides Scene)   |
+-------------------------------------------------------+
                           │
                           ▼
+-------------------------------------------------------+
|             Layout Engine & Frustum Culler            |
| - Line breaker / Paginator (cosmic-text)              |
| - Active viewport window culling (e.g. Rows 40-120)   |
+-------------------------------------------------------+
                           │
                           ▼
+-------------------------------------------------------+
|             DisplayList Generation (Rust)             |
| - Vec<DrawCommand> (Glyphs, Rects, Borders, Images)   |
| - Dirty-rect tracking for partial frame invalidation  |
+-------------------------------------------------------+
                           │
            ┌──────────────┴──────────────┐
            ▼                             ▼
  [Desktop Native (Tauri)]        [WebAssembly Target]
  - wgpu / tiny-skia Direct       - Serialized Flat Buffer /
  - Native Window Surface           SharedArrayBuffer to Canvas2D
```

### 2.2 DrawCommand Serialization Primitives
```rust
#[repr(C)]
pub enum DrawCommand {
    FillRect { rect: Rect, color: Color },
    StrokeRect { rect: Rect, color: Color, width: f32 },
    DrawTextRun {
        glyphs: Vec<PositionedGlyph>,
        font_id: FontId,
        color: Color,
        baseline: Point,
    },
    DrawPath { path: VectorPath, fill: Option<Color>, stroke: Option<Stroke> },
    DrawImage { image_id: ImageId, dest_rect: Rect, clip: Option<Rect> },
    PushClip { rect: Rect },
    PopClip,
}
```

---

## 3. Deep Crate Internal Data Structures

### 3.1 `wps-core` — Foundation, VFS & Inversible Command Engine
* **Virtual File System (VFS):** Unified trait `VirtualFileSystem` with async read/write/stat operations backed by OS filesystem on Desktop and OPFS/IndexedDB on Web.
* **Command Bus & Inverse Undo/Redo:**
  ```rust
  pub trait Command<T>: Send + Sync {
      fn execute(&self, target: &mut T) -> Result<Box<dyn Command<T>>, CoreError>;
      fn description(&self) -> &'static str;
  }
  ```
  Executing a command returns its exact mathematical inverse command, ensuring zero-loss linear undo/redo without document cloning.
* **Geometry & Units:** Strongly-typed units (`Twips`, `Emu`, `Pt`, `Px`, `Mm`, `Inches`, `Percent`) with explicit conversions preventing floating-point inaccuracies.

### 3.2 `wps-writer` — High-Performance Word Processing Subsystem
* **Document DOM & Rope Representation:**
  * Uses `ropey::Rope` for $O(\log N)$ character insertions and deletions at arbitrarily large scales.
  * Formatted character attributes (bold, italic, font, color, link) stored in a B-Tree Attribute Interval Tree over rope byte-ranges.
* **Incremental Line-Breaking & Pagination Cache:**
  * Paragraphs are tokenized into shaping runs and cached using a key `(ParagraphId, AvailableWidthTwips, StyleHash)`.
  * Multi-column balance and widow/orphan suppression algorithm computes section breaks and page allocations incrementally without full document re-flow.
* **Revision Tree (Track Changes):**
  * Revisions form an immutable delta log with user identity, timestamp, and edit type (`Insert`, `Delete`, `PropertyChange`), supporting seamless redline visualization and one-click reconciliation.

### 3.3 `wps-sheets` — Sparse Matrix & Reactive Dependency Engine
* **Sparse Matrix Architecture:**
  * Grid partitioned into 64×64 cell chunks stored inside `BTreeMap<(u32, u16), Box<SparseChunk64x64>>`.
  * Completely empty tiles consume 0 bytes of cell payload.
  * All cell text strings are interned through an immutable String Interner (`StringId`), minimizing memory allocation.
* **Dependency DAG & Topological Calculation:**
  * Built on `petgraph::stable_graph::StableDiGraph<CellCoordinate, ()>`.
  * Cell mutations trigger a reverse-topological pass over dependents.
  * Tarjan's Strongly Connected Components (SCC) identifies self-referencing or circular chains and emits `#CIRCULAR!` error states.
* **Dynamic Array Spilling:**
  * Functions returning 2D arrays (`FILTER`, `UNIQUE`, `SORT`, `SEQUENCE`) reserve bounding boxes in the spill tracker. If occupied cells overlap, the source evaluates to `#SPILL!`.

### 3.4 `wps-slides` — Vector Scene Graph & Presentation Engine
* **3-Tier Cascade Hierarchy:**
  * `SlideMaster` defines default layouts, color themes, and slide background vectors.
  * `SlideLayout` inherits master properties and places named placeholder shapes.
  * `Slide` contains user shapes and overrides layout placeholders.
* **A* Smart Connectors:**
  * Connectors between shapes query an obstacle grid using an A* grid router to create orthogonal lines with automatic obstacle avoidance.
* **Morph Transition Interpolator:**
  * Compares matching shape trees across two slides, computes interpolation paths for bounding boxes, corner radii, fill colors, and rotation angles, rendering smooth 60–120 FPS transitions.

### 3.5 `wps-pdf` — Spatial Layout Reconstruction & OCR
* **PDF Object Stream Parser:** ISO 32000 compliant parser handling xref tables, flate/dct/ccitt stream filters, Type1/TrueType/CFF embedded fonts, and AcroForms.
* **Spatial Layout Reconstruction Engine:**
  1. Positioned glyph extraction with transformation matrix resolution.
  2. Recursive XY-Cut algorithm separates columns and blocks by analyzing horizontal and vertical white-space projection profiles.
  3. DBSCAN clustering on baseline angles and line-heights groups glyph runs into semantic paragraphs.
  4. Vector line intersection detection reconstructs tabular grid matrices (`Row` × `Col`).
* **Direct PDF Inline Editing:**
  * Modifying text within a reconstructed paragraph reflows the paragraph boundaries and generates replacement PDF content streams with matching font descriptors.

### 3.6 `wps-python` & `wps-ai` — Computation & Intelligence
* **Unified Python Runtime Trait:**
  ```rust
  pub trait PythonEngine: Send + Sync {
      fn eval_cell(&mut self, script: &str, context: &SheetContext) -> Result<PyOutput, PyError>;
      fn run_macro(&mut self, script: &str, app: &mut OfficeContext) -> Result<(), PyError>;
  }
  ```
  Implemented on Desktop using `pyo3` and on Web using a Pyodide Web Worker message bus.
* **WPS AI Tool Routing:**
  * Structured JSON-RPC / tool-call interface that compiles natural language prompts into typed AST mutations (`InsertSection`, `UpdateFormula`, `CreatePivotTable`, `AddSlide`, `HighlightPdfBBox`).

---

## 4. Frontend & Tauri 2.0 Topology

* **Frontend:** Built with React 19, TypeScript, Tailwind CSS, and the Mercury Design System tokens.
* **Tauri 2.0 Host:**
  * Native OS Menus (File, Edit, View, Insert, Format, Tools, Help).
  * System file picker, local printing spooler, OS notifications.
  * High-speed Tauri IPC bridge (`invoke('execute_wps_command', payload)`) mapping directly to the Rust Command Bus.
