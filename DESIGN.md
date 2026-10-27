# Mercury Design System Specification for OpenWPS (Design.md)

*Synthesized from `npx getdesign@latest add mercury` and adapted for High-Density Office, Multi-Tab Canvas & Viewport Workspaces.*

---

## 1. Executive Philosophy: Financial-Grade Office Productivity

Mercury for OpenWPS merges the tranquility, precision, and architectural discipline of private fintech software with the high-density requirements of a professional office suite (Word Processing, Spreadsheets, Presentations, PDF Studio, Python Data Science, and AI Co-Pilot).

### Key Architectural Pillars:
1. **Fluid Precision & High-Craft Minimalist Density:**
   * Traditional office suites suffer from cluttered, bloated ribbon interfaces with dozens of low-contrast, visually noisy icons.
   * OpenWPS uses Mercury's **progressive disclosure**: high-signal contextual toolbars, micro-toolbars that materialize at the cursor on text or cell selection, and an instant universal `Cmd/Ctrl + K` Command Palette.
2. **Dual-World Surface Atmosphere:**
   * **Light Mode (The Daytime Canvas):** A near-white canvas (`#fbfcfd`), deep plum-ink text (`#28253b`), crisp 1px borders (`#c3c3cc` / `#ededf3`), and pale lavender feature/inspector surfaces (`#edecfb`).
   * **Night Mode (The Precision Darkroom):** Deep ink-violet night workspace (`#171721`), elevated card surfaces (`#1e1e2a`), pale lavender-white typography (`#ededf3`), and subtle indigo glow accents (`#5266eb`).
3. **Restrained Accent Hierarchy:**
   * Primary action calls (Save, Share, Run Python, AI Generate) use the singular, confident **Mercury Indigo** (`#5266eb`).
   * Secondary actions exist as translucent pills (`rgba(82, 102, 235, 0.1)` on light, `rgba(156, 180, 232, 0.2)` on dark) with 40px capsule rounding, eliminating visual fatigue.

---

## 2. Complete Design Tokens Manifest

### 2.1 CSS Variables Root Definition

```css
:root {
  /* Core Base Colors */
  --mercury-primary: #5266eb;
  --mercury-on-primary: #ffffff;
  --mercury-periwinkle: #9cb4e8;
  --mercury-canvas: #fbfcfd;
  --mercury-surface: #ffffff;
  --mercury-surface-tint: #fbfbff;
  --mercury-lavender: #edecfb;
  --mercury-night: #171721;
  --mercury-night-raised: #1e1e2a;
  --mercury-ink: #28253b;
  --mercury-ink-secondary: #554e7b;
  --mercury-on-dark: #ededf3;
  --mercury-fog: #c3c3cc;
  --mercury-slate: #70707d;

  /* Office Chrome Layout Tokens (Light Mode) */
  --mercury-bg-canvas: #f4f5f8;
  --mercury-bg-surface-1: #ffffff;
  --mercury-bg-surface-2: #fbfbff;
  --mercury-bg-surface-3: #edecfb;
  --mercury-border-hairline: 1px solid rgba(195, 195, 204, 0.45);
  --mercury-border-active: 1px solid #5266eb;

  /* Module Semantic Accents (Mercury-Tinted High-Craft) */
  --mercury-accent-writer: #3b5bdb;
  --mercury-accent-writer-tint: rgba(59, 91, 219, 0.08);
  --mercury-accent-sheets: #0ca678;
  --mercury-accent-sheets-tint: rgba(12, 166, 120, 0.08);
  --mercury-accent-slides: #e8590c;
  --mercury-accent-slides-tint: rgba(232, 89, 12, 0.08);
  --mercury-accent-pdf: #e03131;
  --mercury-accent-pdf-tint: rgba(224, 49, 49, 0.08);
  --mercury-accent-ai-python: #7048e8;
  --mercury-accent-ai-python-tint: rgba(112, 72, 232, 0.08);
  --mercury-accent-ai-glow: 0 0 16px rgba(112, 72, 232, 0.25);

  /* Spacing Scale */
  --mercury-space-xs: 4px;
  --mercury-space-sm: 8px;
  --mercury-space-md: 12px;
  --mercury-space-lg: 16px;
  --mercury-space-xl: 20px;
  --mercury-space-xxl: 32px;

  /* Border Radii */
  --mercury-radius-none: 0px;
  --mercury-radius-xs: 4px;
  --mercury-radius-sm: 6px;
  --mercury-radius-md: 10px;
  --mercury-radius-lg: 14px;
  --mercury-radius-xl: 20px;
  --mercury-radius-pill: 40px;
  --mercury-radius-full: 9999px;

  /* Typography */
  --mercury-font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --mercury-font-mono: 'JetBrains Mono', 'Fira Code', Menlo, monospace;
}

[data-theme='dark'] {
  --mercury-bg-canvas: #12121a;
  --mercury-bg-surface-1: #171721;
  --mercury-bg-surface-2: #1e1e2a;
  --mercury-bg-surface-3: #28253b;
  --mercury-border-hairline: 1px solid rgba(112, 112, 125, 0.35);
  --mercury-ink: #ededf3;
  --mercury-ink-secondary: #c3c3cc;
}
```

---

## 3. Typography & Numeric Precision

### 3.1 UI Chrome & Navigation
* **Primary Family:** `Inter`, with `-0.011em` tracking for UI micro-labels, tabs, menu items, and tooltip badges.
* **Weights:** Strict light weights matching Mercury specifications:
  * Body & Menu Labels: `font-weight: 400`
  * Action Buttons & Ribbon Tabs: `font-weight: 420` (or `450` on platforms without variable axes)
  * Eyebrows & Section Headers: `font-weight: 500`, uppercase, `letter-spacing: 0.05em`

### 3.2 Numeric & Code Discipline
* **Tabular Alignment:** All spreadsheet row/column headers, status bar metrics (word count, page number, selection coordinates), and PDF dimension callouts MUST declare `font-variant-numeric: tabular-nums lining-nums`.
* **Formula Bar & Python Console:**
  * Strict monospace typography (`JetBrains Mono` / `--mercury-font-mono`).
  * Syntax highlighted with Mercury semantic tokens: functions in Indigo (`#5266eb`), ranges in Jade (`#0ca678`), string literals in Coral (`#e8590c`), numbers in Iris (`#7048e8`).

---

## 4. Signature OpenWPS + Mercury Layout Architecture

### 4.1 Unified Polymorphic Multi-Document Top Tab Bar (Height: 40px)
* **Visual Treatment:** Sits flush on `--mercury-bg-surface-1` with 1px bottom border `--mercury-border-hairline`.
* **Tab Characteristics:**
  * Active tab lifts to surface color with a 2px top accent indicator colored by document type (Cobalt for Writer, Emerald for Sheets, Coral for Slides, Crimson for PDF).
  * Inactive tabs have translucent hover pills (`rgba(82, 102, 235, 0.06)`).
  * Tab metadata contains: Document Type Icon Badge (16x16), Title, Dirty-State Dot (6px subtle dot), Real-Time Collaborator Avatars (18px circular stack), and Close affordance (`✕`).
  * Split-Screen Toggle Button (`◫` / `⊟`) on the far right allows one-click side-by-side layout.

### 4.2 Contextual Mercury Ribbon Bar (Dual-Mode: Grouped & Compact)
1. **Mode A: Classic High-Density Ribbon (Height: 88px):**
   * Divided into clear semantic clusters separated by 1px vertical hairline dividers (`--mercury-border-hairline`).
   * Primary actions styled with Mercury icon-over-label buttons; subtle translucent hover indicators.
2. **Mode B: Mercury Single-Line Compact Ribbon (Height: 44px):**
   * High-craft horizontal strip prioritizing primary actions, dropdown triggers, and dynamic micro-pills.
3. **Floating Contextual Micro-Toolbar:**
   * Triggers within 80ms of any text selection (Writer/PDF) or cell block selection (Sheets).
   * Renders as an elevated Mercury capsule (`--mercury-radius-pill`, `box-shadow: 0 4px 20px rgba(23, 23, 33, 0.12)`).

### 4.3 Dockable Right Inspector & AI/Python Drawer (Width: 320px–420px)
* Slides out with a smooth 200ms `cubic-bezier(0.16, 1, 0.3, 1)` transition.
* Houses contextual panes:
  * **Writer:** Styles Gallery, Navigator, Track Changes Revisions Tree, Comments.
  * **Spreadsheets:** Pivot Table Field Builder, Conditional Formatting Manager, Sparkline Inspector.
  * **Presentation:** Slide Master Tree, Animation Timeline, Shape Properties.
  * **PDF:** Page Thumbnail Grid, Reconstructed Table Inspector, Digital Signatures, OCR Inspector.
  * **Python & AI:** Live Python DataFrame Console (`=PY()`) and WPS AI Co-Pilot Chat with interactive action cards.

### 4.4 Canvas Workspaces & Viewport Frustum Host
* The canvas viewport container sits on `--mercury-bg-canvas`.
* Document pages, slide canvases, and PDF spreads render with a gentle 1px neutral border and subtle 2px layered blur shadow (`0 2px 8px rgba(40, 37, 59, 0.06)`).
* Grid viewport in Spreadsheets displays 1px solid hairline grid lines (`#e2e4ea` in light, `#262636` in dark).
