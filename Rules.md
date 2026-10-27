# OpenWPS Engineering Constitution & Verification Laws (Rules.md)

## Status: ACTIVE & ENFORCED

---

### Rule 1: Zero-Stub & Zero-Mock Law
1. **Absolute Prohibition of Stubs:**
   * Developers and agents must NEVER write `todo!()`, `unimplemented!()`, `// TODO: implement`, `// Placeholder`, empty stub functions returning arbitrary defaults, or mock UI elements disconnected from the underlying Rust computational engine.
   * Every function, method, struct, and enum declared across `crates/*` must be fully implemented with concrete, production-grade logic.
2. **End-to-End Architectural Continuity:**
   * Every Ribbon tool button, keyboard shortcut, menu item, formula parser node, PDF annotation tool, and Python bridge call must perform a real, deterministic mutation or query on the Rust document AST and propagate changes to the viewport render pipeline.
3. **No Decorative or Hollow UI:**
   * Frontend components in `apps/workspace-ui` must never simulate functionality with dummy state variables. All state transformations must flow through the Rust Command Dispatcher and undo/redo history.

---

### Rule 2: Single Source of Truth in Rust (`crates/*`)
1. **Rust Monolith for Domain Logic:**
   * All document models, OOXML parsers, PDF stream decoders, formatting engines, formula evaluation DAGs, spatial layout reconstruction algorithms, and CRDT synchronization states must reside exclusively inside the Rust workspace (`crates/*`).
2. **Frontend as Hardware-Accelerated Viewport Host:**
   * The TypeScript/React frontend (`apps/workspace-ui`) is strictly an ergonomic presentation layer and canvas viewport host styled with the Mercury Design System.
   * The frontend MUST NEVER contain duplicated document calculation logic, layout re-computation, or file parsing heuristics.
3. **Determinism:**
   * Given identical document state and viewport dimensions, the Rust rendering engine must generate bit-identical `DisplayList` draw calls across Desktop and WebAssembly targets.

---

### Rule 3: Deterministic Canvas/GPU Rendering (No `contenteditable` DOM Hacks)
1. **Banned Technologies:**
   * OpenWPS explicitly forbids `contenteditable`, raw HTML `<div>` content flow for document pages, DOM-based table cell grids for millions of rows, or native browser selection hacks.
2. **Direct Virtualized Rasterization Pipeline:**
   * Writer document pages, Spreadsheet cell grids, Presentation slide stages, and PDF pages must be rendered directly onto hardware-accelerated Canvas2D / WebGPU contexts using Rust display lists generated via `cosmic-text` and vector geometry primitives.
3. **Virtual Frustum Culling:**
   * Only elements within the active viewport (plus a defined overdraw buffer) may be processed for rendering, keeping memory usage constant and rendering smooth at 120 FPS.
4. **First-Class Text Subsystem:**
   * Caret positioning, multi-line selection bounding boxes, BiDi text handling, font fallback, and IME composition must be managed by the engine with sub-pixel precision.

---

### Rule 4: Dual-Target Parity (Desktop Native + Web Wasm)
1. **Compilation Invariance:**
   * Every core crate in `crates/*` must compile cleanly without warnings for:
     * Native Desktop (`x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`) via Tauri 2.0 IPC.
     * WebAssembly (`wasm32-unknown-unknown`) via `wasm-bindgen` and `web-sys`.
2. **Platform Abstraction Layer:**
   * Any platform-specific system integration—including file dialogs, clipboard access, multi-threading, and Python runtime execution—must be abstracted behind clean Rust traits with conditional compilation:
     * Desktop: `pyo3`, native OS threadpools, direct filesystem I/O.
     * Web: `Pyodide` Web Workers, Web Locks API, Origin Private File System (OPFS), and WebAssembly message passing.
3. **No Feature Degradation:**
   * All 150+ spreadsheet formulas, OOXML import/export codecs, PDF spatial text reconstruction, and track changes operations must function identically on both web and desktop.

---

### Rule 5: Mercury Design System Compliance (`npx getdesign@latest add mercury`)
1. **Design System Token Integrity:**
   * Every visual component, ribbon bar, floating toolbar, palette, drawer, dialog, and status bar must strictly adhere to `./Design.md` and the design tokens installed from `npx getdesign@latest add mercury`.
2. **Prohibited Styling Practices:**
   * Arbitrary hardcoded hex colors, random padding/margin values, custom drop shadows, or generic button styles are strictly forbidden.
   * All color references must map to `--mercury-*` tokens (`canvas`, `surface-1/2/3`, `night`, `night-raised`, `ink`, `ink-secondary`, `primary`, module-specific semantic tints).
3. **Progressive Disclosure:**
   * UI surfaces must maintain financial-grade clarity and high-craft minimalist density. Advanced controls must remain accessible through contextual ribbons, floating selection popovers, or the `Cmd/Ctrl + K` Command Palette without cluttering the primary canvas.

---

### Rule 6: Mandatory Verification Gates
1. **Automated Verification Before Commit:**
   * A phase or task in `Phases.md` cannot be marked complete (`[x]`) unless the following suite of gates passes completely:
     1. `cargo check --workspace` passes with 0 errors.
     2. `cargo test --workspace` passes with 0 failures.
     3. `cargo check --target wasm32-unknown-unknown -p wps-wasm` passes.
     4. `pnpm --filter workspace-ui run typecheck` passes with 0 TypeScript errors.
     5. `pnpm --filter workspace-ui run build` completes successfully.
2. **Regression Prevention:**
   * New functionality must always be accompanied by deterministic unit and integration tests asserting AST transformations, serialization parity, and rendering coordinates.
