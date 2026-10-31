# Mercury Design System Specification (`DESIGN-mercury.md`)

*Unified High-Craft Office Design Specification for OpenWPS (`veronica-medusa/MPS`)*

---

## 1. Executive Philosophy & The Two Worlds

OpenWPS is structured into two native surface worlds:
1. **The `night` World (`#171721`):** Home launchpad, file roaming version history time-machine, and universal top/bottom workspace framing chrome.
2. **The `canvas` Light World (`#fbfcfd`):** Crisp, high-contrast, sub-pixel document pages (Writer), spreadsheet matrices (Spreadsheets), presentation vector stages (Presentation), and ISO 32000 PDF reading/reflow viewports.

### Architectural Laws
* **Light Variable Typography:** Typography weights NEVER exceed `500`. Arcadia Display uses weight `480`, Arcadia Sans uses weights `360`, `400`, `420`, and `500`.
* **Single Primary Action Law:** Never render more than one solid `#5266eb` primary button inside any toolbar, modal footer, or card group. All secondary actions exist as translucent pills (`button-glass-dark`, `button-tint-dark`, `button-tint-light`, `button-outline-light`).
* **Zero Chrome Color Bloat:** Eliminate distracting status reds, oranges, and greens from navigation and ribbon chrome. Use restrained Mercury semantic tints and slate/fog hairlines.

---

## 2. Complete Token Manifest

### 2.1 CSS Custom Properties

```css
:root {
  /* ─── Mercury Brand & Surface Palette ─── */
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

  /* ─── Translucent Capsule Fills & Hairlines ─── */
  --mercury-glass-dark-bg: rgba(175, 178, 206, 0.20);
  --mercury-glass-dark-border: rgba(175, 178, 206, 0.36);
  --mercury-tint-dark-bg: rgba(156, 180, 232, 0.20);
  --mercury-tint-light-bg: rgba(82, 102, 235, 0.10);
  --mercury-outline-light-bg: rgba(112, 115, 147, 0.10);
  --mercury-outline-light-border: rgba(112, 115, 147, 0.22);
  --mercury-soft-stack-shadow: 0 2px 12px rgba(40, 37, 59, 0.05), 0 1px 2px rgba(40, 37, 59, 0.04);

  /* ─── Typography Families & Light Variable Weights (NEVER > 500) ─── */
  --font-arcadia-display: 'arcadiaDisplay', 'Inter Variable', 'Inter', -apple-system, sans-serif;
  --font-arcadia-sans: 'arcadia', 'Inter Variable', 'Inter', -apple-system, sans-serif;
  --font-mercury-mono: 'JetBrains Mono', ui-monospace, SFMono-Regular, monospace;

  /* ─── Border Radius Scale ─── */
  --radius-none: 0px;
  --radius-xs: 4px;
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-xl: 32px;
  --radius-pill: 40px;
  --radius-full: 9999px;

  /* ─── 8px Spacing System ─── */
  --space-xs: 8px;
  --space-sm: 12px;
  --space-md: 16px;
  --space-lg: 20px;
  --space-xl: 24px;
  --space-xxl: 32px;
  --space-huge: 40px;
  --space-section: 80px;
  --space-section-lg: 128px;
}
```

---

## 3. Strict Typography Scale Classes

| Class | Font Family | Size | Weight | Line Height | Tracking | Usage |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `.mercury-stat` | `var(--font-arcadia-display)` | 60px | 480 | 1.1 | 0 | Stat block numerals in Home Launchpad |
| `.mercury-display` | `var(--font-arcadia-display)` | 52px | 480 | 1.1 | 0 | Main hero titles & welcome headings |
| `.mercury-h1` | `var(--font-arcadia-display)` | 42px | 480 | 1.15 | 0.42px | Document title & major headers |
| `.mercury-h2` | `var(--font-arcadia-display)` | 28px | 480 | 1.2 | 0.42px | Section headers & modal titles |
| `.mercury-lead` | `var(--font-arcadia-display)` | 24px | 480 | 1.2 | 0.48px | Sub-headers & feature lead-ins |
| `.mercury-body-lg` | `var(--font-arcadia-sans)` | 18px | 360 | 1.35 | 0 | Document body lead text |
| `.mercury-body` | `var(--font-arcadia-sans)` | 16px | 400 | 1.4 | 0 | Standard document & reading text |
| `.mercury-ui` | `var(--font-arcadia-sans)` | 16px | 420 | 1.0 | 0 | Nav links & primary button labels |
| `.mercury-ui-sm` | `var(--font-arcadia-sans)` | 14px | 420 | 1.0 | 0.07px | Ribbon pills, status pills & menu items |
| `.mercury-eyebrow` | `var(--font-arcadia-sans)` | 14px | 500 | 1.0 | 0.28px | Ribbon section headers (UPPERCASE) |
| `.mercury-legal` | `var(--font-arcadia-sans)` | 12px | 400 | 1.4 | 0.24px | Status bar counters & footnote captions |

---

## 4. Component Recipes

### 4.1 Mercury Action Pills (`MercuryButton`)
* **`button-primary`:** `#5266eb` background, `#ffffff` label, `32px` (`--radius-xl`) corners, `40px` height, `0 20px` padding. (Limit: 1 per group).
* **`button-glass-dark`:** `rgba(175, 178, 206, 0.20)` fill, `1px solid rgba(175, 178, 206, 0.36)`, `#ededf3` text, `40px` radius, `40px` height.
* **`button-tint-dark`:** `rgba(156, 180, 232, 0.20)` fill, `#ededf3` text, `40px` radius, `40px` height.
* **`button-tint-light`:** `rgba(82, 102, 235, 0.10)` fill, `#28253b` text, `40px` radius, `40px` height.
* **`button-outline-light`:** `rgba(112, 115, 147, 0.10)` fill, `1px solid rgba(112, 115, 147, 0.22)`, `#28253b` text, `40px` radius, `40px` height.
* **`button-small`:** `rgba(82, 102, 235, 0.10)` fill, `#28253b` text, `14px` (`ui-sm`), `40px` radius, `0 16px` padding, `32px` height.
* **`button-icon-round`:** `border-radius: 9999px` circular controls.

### 4.2 Fused Input Capsule (`MercuryInputCapsule`)
* **Shell:** `46px` tall, `#ffffff` surface fill, rounded-full (`9999px`), `1px solid rgba(112, 115, 147, 0.22)`.
* **Input:** `#28253b` text, `20px` left padding, `32px` left radius, transparent border, outline-none.
* **Trailing Action:** Fused `#5266eb` `button-primary` (`40px` tall, `0 20px` padding, `32px` right radius).
* **Application:** Formula Bar, Command Palette (`Cmd+K`), Inline `/ai` Prompt Capsule, WPS AI Chat Input.
