# OpenWPS Product Requirements Document (PRD.md)

## Pinpoint WPS Office Feature Matrix & Functional Specifications

---

## 1. Executive Product Overview
**OpenWPS** is a high-performance, enterprise-grade, memory-safe alternative to the WPS Office Suite, running with bit-for-bit layout determinism across **Desktop (Windows, macOS, Linux via Tauri 2.0)** and **Web (WebAssembly + WebGPU/Canvas2D)**. Styled exclusively with the **Mercury Design System**, OpenWPS delivers financial-grade minimalism, low visual clutter, and zero-compromise office calculation and layout engines.

---

## 2. Pillar 1: Unified Multi-Tab Workspace & File Roaming

### 1.1 All-in-One Multi-Document Shell
* **Single Window Experience:** Open `.docx`, `.xlsx`, `.pptx`, `.pdf`, `.csv`, `.tsv`, and `.md` files concurrently within a single unified window.
* **Polymorphic Tab Engine:** Each tab reflects its document type with a dedicated Mercury-tinted icon badge, live uncommitted changes indicator (dirty dot), real-time remote collaborator presence badges, and context-sensitive keyboard navigation (`Ctrl/Cmd + Tab`, `Ctrl/Cmd + W`, `Ctrl/Cmd + T`).
* **Session Persistence:** State and open tab workspaces are persisted locally in IndexedDB / local SQLite store with automatic crash recovery.

### 1.2 Synchronous & Asynchronous Split-Screen View
* **Side-by-Side Dual Pane:** View two documents side-by-side (e.g., PDF reference document on left, Word Processor or Spreadsheet on right).
* **Synchronized Scroll Option:** Lock step scrolling across two identical or translated documents for comparison and translation proofreading.

### 1.3 Local Content-Addressed File Roaming
* **Snapshot Versioning:** Every document save automatically records a cryptographic content hash (`blake3`) snapshot in a local workspace history vault.
* **Point-in-Time Recovery:** Time-travel scrubber allowing users to inspect and restore any previous version without overwriting the current state.

---

## 3. Pillar 2: WPS Writer (Word Processor)

### 2.1 File Format Interoperability
* **Read / Write Parity:** Full bidirectional support for `.docx` (ECMA-376 / ISO/IEC 29500 OOXML) packages.
* **Import Support:** `.rtf`, `.txt`, `.md`, legacy OLE2 `.doc` (via structured compound document reader), and direct vector PDF export.

### 2.2 Rich Document AST & Layout Architecture
* **Structural Hierarchy:** Document -> Section -> Header/Footer -> Paragraph -> Character Run -> Inline/Floating DrawingML Objects.
* **Section Attributes:** Independent page dimensions (A4, Letter, Legal, Executive, Custom), orientation (Portrait/Landscape), margins (Top, Bottom, Left, Right, Gutter), multi-column text flows with balancing, and different First Page / Odd / Even headers & footers.
* **Rich Paragraph & Text Formatting:** Indentation (First Line, Hanging, Left, Right), line spacing (Single, 1.15, 1.5, Double, Multiple, At Least, Exactly), space before/after, alignment (Left, Center, Right, Justified, Distributed), drop caps, hyphenation, font family, size (in half-points), color, highlight, bold, italic, underline (single, double, dotted, wavy), strikethrough, subscript, superscript, letter-spacing, and small caps.
* **Style Inheritance Cascade:** Built-in Normal, Heading 1–9, Title, Subtitle, Quote, Code, with explicit `based_on` inheritance and `next_paragraph_style` links.

### 2.3 Advanced Table Engine
* **Cell Topologies:** Arbitrary grid layouts with horizontal and vertical cell merging (`gridSpan`, `vMerge`), split cells, cell margins, and custom cell background shading.
* **Dynamic Table Flow:** Automatic table row splitting across page boundaries with repeating header rows (`tblHeader`) on subsequent pages.
* **Calculated Table Cells:** In-table formulas (`=SUM(ABOVE)`, `=AVERAGE(LEFT)`).

### 2.4 Floating Objects & Wrap Polygons
* **Wrap Modes:** In-line with text, Square, Tight, Through, Top and Bottom, Behind Text, and In Front of Text.
* **Vector Guides:** Live smart magnetic snapping to page margins, paragraph centers, and other layout shapes during object manipulation.

### 2.5 Collaborative Track Changes (Redlining) & Threaded Comments
* **Revision Tracking:** AST revision tree logging `InsertRun`, `DeleteRun`, and `FormatChange` with exact user UUID, timestamp, and optional revision notes.
* **Review UI:** Inline strikethrough/underline revisions or margin balloon view; one-click Accept/Reject for individual changes or batch "Accept All / Reject All".
* **Threaded Comments:** Anchor comments to exact character offsets (`(start, end)` rope ranges) with status (`Open`, `Resolved`).

### 2.6 Dynamic Fields & Mail Merge
* **Dynamic Fields:** Live evaluated fields including `PAGE`, `NUMPAGES`, `DATE`, `TIME`, `AUTHOR`, and dynamic `TOC` (Table of Contents) with page number dot leaders.
* **Mail Merge Engine:** Connect `.docx` templates with fields (`«FirstName»`, `«Company»`, `«Balance»`) to any OpenWPS Spreadsheet or CSV data source with batch PDF / DOCX generation.

---

## 4. Pillar 3: WPS Spreadsheets (Calculation, Grid & Smart Toolbox)

### 3.1 Sparse Matrix Grid Engine
* **Grid Capacity:** Full `1,048,576` rows × `16,384` columns (`A` through `XFD`).
* **Memory Architecture:** Chunked 64×64 sparse tile array storing typed cell values (`Empty`, `Text(InternedId)`, `Number(f64)`, `Boolean`, `Error`, `DateTime`, `Array(Vec<Vec<CellValue>>)`) and formatting descriptors.
* **Viewport Virtualization:** Seamless scrolling at 120 FPS across 1M rows with frozen rows/columns, split panes, and row/column grouping/outlines.

### 3.2 Reactive Dependency DAG & 150+ Built-In Functions
* **Lexer & Pratt Formula Parser:** Parsing standard Excel/WPS formula grammar, absolute/relative references (`$A$1`, `B2:D10`), cross-sheet syntax (`'Q3 Budget'!C5`), named ranges, structured table references, and dynamic array operators (`#`).
* **Directed Acyclic Graph (DAG):** Fine-grained dependency tracking powered by `petgraph`, with Tarjan's Strongly Connected Components algorithm for circular reference detection.
* **Dynamic Array Spilling:** Multi-cell spill ranges with `#SPILL!` collision detection.
* **150+ Core Functions across 8 categories:**
  1. **Math & Trig:** `SUM`, `SUMIF`, `SUMIFS`, `SUMPRODUCT`, `ABS`, `ROUND`, `ROUNDUP`, `ROUNDDOWN`, `MROUND`, `CEILING`, `FLOOR`, `INT`, `MOD`, `POWER`, `SQRT`, `EXP`, `LN`, `LOG10`, `PI`, `SIN`, `COS`, `TAN`, `RAND`, `RANDBETWEEN`, `SUBTOTAL`, `AGGREGATE`.
  2. **Statistical:** `AVERAGE`, `AVERAGEA`, `AVERAGEIF`, `AVERAGEIFS`, `COUNT`, `COUNTA`, `COUNTBLANK`, `COUNTIF`, `COUNTIFS`, `MAX`, `MAXA`, `MAXIFS`, `MIN`, `MINA`, `MINIFS`, `MEDIAN`, `MODE.SNGL`, `STDEV.S`, `STDEV.P`, `VAR.S`, `VAR.P`, `PERCENTILE.INC`, `QUARTILE.INC`, `RANK.EQ`, `LARGE`, `SMALL`, `CORREL`, `SLOPE`, `INTERCEPT`, `FORECAST.LINEAR`.
  3. **Lookup & Dynamic Arrays:** `VLOOKUP`, `HLOOKUP`, `XLOOKUP`, `XMATCH`, `INDEX`, `MATCH`, `OFFSET`, `INDIRECT`, `CHOOSE`, `ROW`, `ROWS`, `COLUMN`, `COLUMNS`, `ADDRESS`, `TRANSPOSE`, `UNIQUE`, `FILTER`, `SORT`, `SORTBY`, `SEQUENCE`, `TAKE`, `DROP`, `VSTACK`, `HSTACK`.
  4. **Logical & Information:** `IF`, `IFS`, `AND`, `OR`, `XOR`, `NOT`, `TRUE`, `FALSE`, `IFERROR`, `IFNA`, `SWITCH`, `ISBLANK`, `ISNUMBER`, `ISTEXT`, `ISERROR`, `ISNA`, `ISFORMULA`, `TYPE`, `N`.
  5. **Text & Regex:** `CONCAT`, `CONCATENATE`, `TEXTJOIN`, `LEFT`, `RIGHT`, `MID`, `LEN`, `FIND`, `SEARCH`, `REPLACE`, `SUBSTITUTE`, `TRIM`, `CLEAN`, `UPPER`, `LOWER`, `PROPER`, `EXACT`, `REPT`, `TEXT`, `VALUE`, `NUMBERVALUE`, `CHAR`, `CODE`, `REGEXMATCH`, `REGEXEXTRACT`, `REGEXREPLACE`.
  6. **Date & Time:** `TODAY`, `NOW`, `DATE`, `DATEVALUE`, `TIME`, `YEAR`, `MONTH`, `DAY`, `HOUR`, `MINUTE`, `SECOND`, `WEEKDAY`, `WEEKNUM`, `ISOWEEKNUM`, `EDATE`, `EOMONTH`, `DATEDIF`, `DAYS`, `NETWORKDAYS`, `NETWORKDAYS.INTL`, `WORKDAY`.
  7. **Financial (Decimal-Exact):** `PMT`, `IPMT`, `PPMT`, `PV`, `FV`, `NPV`, `XNPV`, `IRR`, `XIRR`, `MIRR`, `RATE`, `NPER`, `SLN`, `SYD`, `DB`, `DDB`, `EFFECT`, `NOMINAL`.

### 3.3 The WPS Smart Toolbox (Pinpoint Signature Feature Set)
* **Merge Without Data Loss:** Merge multi-cell ranges while concatenating all non-empty values using configurable delimiters (comma, space, semicolon, newline) instead of discarding lower-right cells.
* **Split Sheet Wizard:** Decompose a master sheet into multiple individual sheets or standalone `.xlsx` files partitioned by unique values in a chosen column.
* **Combine Workbooks & Sheets:** Intelligently merge tables across selected sheets or external files into a unified master ledger with automatic column alignment and source tracing.
* **Sheet Comparator:** Side-by-side visual diffing engine that highlights cells with added values, deleted values, formula mutations, or formatting discrepancies.
* **One-Click Data Hygiene Tools:** Batch deduplication across custom column sets, trim whitespace, strip unprintable ASCII, regex extraction of email addresses/phone numbers/URLs, text-to-number/date casting, and batch row insertion.

### 3.4 Pivot Tables, Charts & Formatting
* **Pivot Table Engine:** Multi-dimensional OLAP-style aggregation cube (`Rows`, `Columns`, `Values`, `Filters`) supporting `SUM`, `COUNT`, `AVERAGE`, `MIN`, `MAX`, `VAR`, `STDDEV`, `% of Total`, and calculated fields.
* **Vector Charts & Sparklines:** Native Canvas2D / WebGPU rendered charts (`Column`, `Stacked Column`, `Bar`, `Line`, `Area`, `Pie`, `Doughnut`, `Scatter`, `Combo`) and inline cell Sparklines (`Line`, `Column`, `Win/Loss`).
* **Conditional Formatting:** 2-Color & 3-Color Scales, Data Bars, Icon Sets (Arrows, Traffic Lights, Stars), and Custom Formula Rules.
* **Data Validation:** Dropdown range selection, numeric constraints, date boundaries, and custom formula validation with input messages and error dialogs.

---

## 5. Pillar 4: WPS Presentation (Slides & Animation Stage)

### 4.1 3-Tier Master Slide Inheritance
* **Inheritance Topology:** `SlideMaster` -> `SlideLayout` -> `Slide`.
* **Theme System:** Theme color palettes (12 semantic slots: Dark 1/2, Light 1/2, Accent 1–6, Hyperlink, Followed Hyperlink), major/minor font schemes, background styles, and reusable placeholder slots (Title, Body, Subtitle, Date, Slide Number, Footer).

### 4.2 DrawingML Vector Engine & Smart Connectors
* **50+ Parametric Shapes:** Rectangles, Rounded Rectangles, Ellipses, Trapezoids, Stars (4-32 points), Arrows, Callouts, Flowchart symbols with draggable adjustment handles.
* **A* Orthogonal & Curved Smart Connectors:** Dynamic auto-routing lines that attach to shape connection points and automatically navigate around obstacles when shapes are moved.
* **Vector Boolean Operations:** Shape Union, Combine, Fragment, Intersect, and Subtract.

### 4.3 Animation Timeline & Transitions
* **Granular Timeline:** Sequence Entrance, Emphasis, Exit, and Motion Path animations with timing triggers (`OnClick`, `WithPrevious`, `AfterPrevious`), duration, delay, and cubic-bezier easing.
* **Morph Transition Engine:** Automatically detects identical or named (`!!shape_name`) shapes and text across adjacent slides and interpolates position, size, color, and rotation seamlessly.
* **Presenter View:** Dual-screen interface displaying active slide, next slide preview, speaker notes, live elapsed timer, virtual laser pointer, and annotation pen.

---

## 6. Pillar 5: WPS PDF Toolkit (Render, Direct Edit, PDF-to-Office, OCR, Sign)

### 6.1 ISO 32000 Conforming PDF Engine
* **High-Speed Rasterizer:** Multi-page continuous, two-page book spread, and mobile Reflow modes.
* **Spatial Layout Reconstruction Engine:**
  * Uses recursive XY-Cut and baseline DBSCAN clustering on positioned glyphs to reconstruct semantic hierarchy (`TextLine`, `ParagraphBox`, `TableGrid`, `ImageBlock`).
* **Direct "Edit PDF Like Word":** Click any text block in a rendered PDF to edit text with automatic line-wrap reflow and embedded font substitution, or drag/resize embedded images.
* **High-Fidelity PDF-to-Office Conversion:**
  * PDF -> `.docx`: Converts reconstructed paragraphs, headings, columns, and embedded images into a clean Word document.
  * PDF -> `.xlsx`: Detects table grids (both bordered lines and white-space aligned tables) and exports clean spreadsheets.
  * PDF -> `.pptx`: Extracts slides and vector graphics into presentation decks.

### 6.2 11+ Annotation Tools & Visual Page Organizer
* **Annotation Suite:** Text Highlight, Underline, Squiggly, Strikethrough, Sticky Notes, Freehand Ink Pen, Text Boxes, Callout Clouds, Geometric Shapes, Dynamic Stamps, and Distance/Area Calibration rulers.
* **Page Organizer:** Visual thumbnail grid to reorder, rotate, crop (`CropBox`), delete, extract, split, or insert pages, plus watermarking and Bates numbering.
* **OCR & Security:** Offline text recognition via `ocrs`, AcroForm generation/filling, digital PKCS#7 signatures, permanent text redaction (stripping glyph bytes from PDF stream), and AES-256 encryption.

---

## 7. Pillar 6: Embedded Python Runtime (`=PY()`) & AST-Native WPS AI + Collaboration

### 7.1 Embedded Python Runtime (`wps-python`)
* **Dual Execution Bridge:** `PyO3` native embedded interpreter for Desktop; `Pyodide` Web Worker with SharedArrayBuffer for Web.
* **Spreadsheet `=PY()` Integration:** Type `=PY("df = xl('A1:C50'); df.describe()")` inside any cell to compute statistics, manipulate data with `pandas` / `polars`, and spill results back into the grid.
* **Visual Plot Spilling:** Output `matplotlib` or `seaborn` plots directly into cell canvases or floating slide frames.

### 7.2 AST-Native WPS AI Co-Pilot (`wps-ai`)
* **AST-Grounded Tools:**
  * `@WriterAI`: Rewrite selection, summarize section, change tone, generate complete outline, auto-format headings.
  * `@SheetsAI`: Natural language to complex formulas (`XLOOKUP`, `SUMIFS`, `=PY()`), generate automated Pivot Tables, and detect data anomalies.
  * `@SlidesAI`: One-prompt presentation generation, generating master layouts and content slides.
  * `@PdfAI`: Vector + structural RAG over open PDFs with clickable citation badges that scroll to and highlight exact bounding boxes on the PDF page canvas.

### 7.3 Real-Time CRDT Collaboration (`wps-collab`)
* **State Machine:** Built on `yrs` (Rust Yjs implementation) over WebSockets with offline-first synchronization.
* **Presence:** Live remote carets, real-time selection highlights, and user avatar indicators in the Mercury top tab bar.
