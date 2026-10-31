import React, { useState, useMemo } from 'react';
import './styles/mercury-tokens.css';

// ============================================================================
// TYPES & WORKSPACE STATE MODELS
// ============================================================================

export type ModuleKind = 'writer' | 'sheets' | 'slides' | 'pdf';
export type SurfaceWorld = 'night-launchpad' | 'light-editor';
export type InspectorTab = 'ai-copilot' | 'python-studio' | 'roaming-history';

export interface WorkspaceTab {
  id: string;
  title: string;
  module: ModuleKind;
  dirty: boolean;
  roamingHash: string;
}

export interface WriterParagraph {
  id: number;
  style: 'Title' | 'Heading 1' | 'Heading 2' | 'Normal' | 'List Paragraph';
  text: string;
  aiRedlineDelete?: string;
  aiRedlineInsert?: string;
}

export interface PdfSpatialBlock {
  blockId: number;
  page: number;
  role: 'Heading' | 'Paragraph' | 'Table';
  bbox: [number, number, number, number];
  text: string;
}

export interface RoamingSnapshot {
  versionId: number;
  timestamp: string;
  author: string;
  label: string;
  sha256Short: string;
}

// ============================================================================
// INITIAL DETERMINISTIC WORKSPACE DATA
// ============================================================================

const INITIAL_TABS: WorkspaceTab[] = [
  { id: 'tab-writer-1', title: 'Q4_Architecture_Spec.docx', module: 'writer', dirty: false, roamingHash: '8f91a2c4' },
  { id: 'tab-sheets-1', title: 'APAC_Revenue_Model.xlsx', module: 'sheets', dirty: false, roamingHash: '3d7e40b1' },
  { id: 'tab-slides-1', title: 'Board_Strategy_Deck.pptx', module: 'slides', dirty: false, roamingHash: 'c20918ef' },
  { id: 'tab-pdf-1', title: 'Master_Enterprise_MSA.pdf', module: 'pdf', dirty: false, roamingHash: '7a115de9' },
];

const INITIAL_WRITER_PARAS: WriterParagraph[] = [
  {
    id: 1,
    style: 'Title',
    text: 'MPS Unified Productivity Engine Specification',
  },
  {
    id: 2,
    style: 'Heading 1',
    text: '1. Executive Architecture Overview',
  },
  {
    id: 3,
    style: 'Normal',
    text: 'We utilize a single Rust and WebAssembly core engine in order to render documents, spreadsheets, presentations, and PDF streams at 120 frames per second.',
    aiRedlineDelete: 'utilize a single Rust and WebAssembly core engine in order to',
    aiRedlineInsert: 'use a unified Rust and WebAssembly core engine to',
  },
  {
    id: 4,
    style: 'List Paragraph',
    text: 'Zero-copy viewport virtualization with sub-pixel cosmic-text glyph shaping.',
  },
];

const INITIAL_SHEET_GRID: string[][] = [
  ['Region', 'Quarter', 'Units', 'Revenue', 'Margin %', 'AI / Formula Output'],
  ['APAC', 'Q1', '1,420', '$426,000', '34.2%', '=SUMIFS(D2:D6, A2:A6, "APAC")'],
  ['EMEA', 'Q1', '1,180', '$354,000', '31.8%', '$1,452,000'],
  ['APAC', 'Q2', '1,650', '$495,000', '36.5%', ''],
  ['AMER', 'Q2', '1,910', '$573,000', '38.0%', ''],
  ['APAC', 'Q3', '1,770', '$531,000', '37.1%', ''],
];

const INITIAL_PDF_BLOCKS: PdfSpatialBlock[] = [
  {
    blockId: 101,
    page: 1,
    role: 'Heading',
    bbox: [50, 72, 540, 104],
    text: 'MASTER ENTERPRISE SERVICES AGREEMENT (ISO 32000 VERIFIED)',
  },
  {
    blockId: 102,
    page: 1,
    role: 'Paragraph',
    bbox: [50, 124, 540, 178],
    text: 'Section 4.2 Data Residency & Encryption: All customer archives are encrypted using AES-256 Revision 6 (V=5, R=6) with cryptographic PKCS#7 /ByteRange digital signatures.',
  },
  {
    blockId: 205,
    page: 1,
    role: 'Table',
    bbox: [50, 200, 540, 275],
    text: 'Section 8.4 Early Termination Fee: In the event of termination prior to the 24-month commitment, Customer shall remit a fixed buyout fee of $45,000 within 30 calendar days.',
  },
];

const INITIAL_SNAPSHOTS: RoamingSnapshot[] = [
  { versionId: 4, timestamp: 'Today, 20:55 IST', author: 'Fiza Syed', label: 'Applied @WriterAI Track Changes Redline', sha256Short: '8f91a2c4' },
  { versionId: 3, timestamp: 'Today, 19:40 IST', author: 'Fiza Syed', label: 'Executed =PY() APAC GroupBy Spill', sha256Short: '3d7e40b1' },
  { versionId: 2, timestamp: 'Today, 18:15 IST', author: 'Fiza Syed', label: 'PDF XY-Cut Table Reconstruction', sha256Short: '7a115de9' },
  { versionId: 1, timestamp: 'Today, 16:00 IST', author: 'Fiza Syed', label: 'Initial ECMA-376 Workspace Import', sha256Short: '1b44c902' },
];

// ============================================================================
// MAIN MERCURY WORKSPACE SHELL COMPONENT
// ============================================================================

export const MercuryWorkspaceShell: React.FC = () => {
  const [surfaceWorld, setSurfaceWorld] = useState<SurfaceWorld>('light-editor');
  const [tabs, setTabs] = useState<WorkspaceTab[]>(INITIAL_TABS);
  const [activeTabId, setActiveTabId] = useState<string>('tab-writer-1');
  const [inspectorOpen, setInspectorOpen] = useState<boolean>(true);
  const [inspectorTab, setInspectorTab] = useState<InspectorTab>('ai-copilot');

  // Editor Interactive States
  const [writerParas, setWriterParas] = useState<WriterParagraph[]>(INITIAL_WRITER_PARAS);
  const [trackChangesOn, setTrackChangesOn] = useState<boolean>(true);
  const [sheetGrid, setSheetGrid] = useState<string[][]>(INITIAL_SHEET_GRID);
  const [selectedCell, setSelectedCell] = useState<[number, number]>([1, 5]);
  const [readingSpotlight, setReadingSpotlight] = useState<boolean>(true);
  const [capsuleInput, setCapsuleInput] = useState<string>(
    '=PY("df = wp.range(\'A1:E6\').to_df(); return df.groupby(\'Region\')[\'Revenue\'].sum()")'
  );
  const [activeSlideIndex, setActiveSlideIndex] = useState<number>(0);
  const [morphPreviewActive, setMorphPreviewActive] = useState<boolean>(false);
  const [highlightedPdfBlockId, setHighlightedPdfBlockId] = useState<number | null>(205);
  const [pdfInlineEditMode, setPdfInlineEditMode] = useState<boolean>(true);
  const [roamingSnapshots, setRoamingSnapshots] = useState<RoamingSnapshot[]>(INITIAL_SNAPSHOTS);

  const activeTab = useMemo(
    () => tabs.find((t) => t.id === activeTabId) || tabs[0],
    [tabs, activeTabId]
  );

  // Accept @WriterAI Track Changes Redline
  const handleAcceptWriterRedlines = () => {
    setWriterParas((prev) =>
      prev.map((p) => {
        if (!p.aiRedlineDelete || !p.aiRedlineInsert) return p;
        return {
          ...p,
          text: p.text.replace(p.aiRedlineDelete, p.aiRedlineInsert),
          aiRedlineDelete: undefined,
          aiRedlineInsert: undefined,
        };
      })
    );
    setRoamingSnapshots((prev) => [
      {
        versionId: prev.length + 1,
        timestamp: 'Just now',
        author: 'Fiza Syed',
        label: 'Accepted @WriterAI Redline Revisions',
        sha256Short: 'a492e11c',
      },
      ...prev,
    ]);
  };

  // Convert PDF directly into a live Writer or Spreadsheets Tab
  const handleConvertPdfToOfficeTab = (targetModule: 'writer' | 'sheets') => {
    const newId = `tab-converted-${Date.now()}`;
    const newTab: WorkspaceTab = {
      id: newId,
      title:
        targetModule === 'writer'
          ? 'Master_Enterprise_MSA_Converted.docx'
          : 'Master_Enterprise_MSA_Tables.xlsx',
      module: targetModule,
      dirty: true,
      roamingHash: 'e9014fa2',
    };
    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newId);
    setSurfaceWorld('light-editor');
  };

  // Execute Formula or =PY() or /ai Capsule Command
  const handleExecuteCapsule = (e: React.FormEvent) => {
    e.preventDefault();
    if (activeTab.module === 'sheets') {
      const [r, c] = selectedCell;
      setSheetGrid((prev) => {
        const copy = prev.map((row) => [...row]);
        if (copy[r] && copy[r][c] !== undefined) {
          copy[r][c] = capsuleInput.startsWith('=PY(') ? '[🐍 DataFrame Spill: $1,452,000]' : capsuleInput;
        }
        return copy;
      });
    }
  };

  return (
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        backgroundColor:
          surfaceWorld === 'night-launchpad' ? 'var(--mercury-night)' : 'var(--mercury-canvas)',
        color:
          surfaceWorld === 'night-launchpad' ? 'var(--mercury-on-dark)' : 'var(--mercury-ink)',
      }}
    >
      {/* ─── 1. MERCURY NAV BAR (72px Night Bar on Every Page) ─── */}
      <header className="mercury-nav-bar">
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-lg)' }}>
          <button
            type="button"
            onClick={() =>
              setSurfaceWorld((w) =>
                w === 'night-launchpad' ? 'light-editor' : 'night-launchpad'
              )
            }
            className="mercury-button-glass-dark ty-ui"
            title="Switch between Mercury Night Launchpad and Light Framed Editor"
          >
            <span
              style={{
                width: 10,
                height: 10,
                borderRadius: 'var(--rounded-full)',
                backgroundColor: 'var(--mercury-primary)',
                display: 'inline-block',
              }}
            />
            <span>MPS Mercury</span>
            <span className="ty-legal" style={{ color: 'var(--mercury-fog)' }}>
              {surfaceWorld === 'night-launchpad' ? 'Night World' : 'Light Canvas'}
            </span>
          </button>

          {/* Polymorphic Multi-Document Capsule Tabs (`nav-link`) */}
          <nav
            aria-label="Open Document Tabs"
            style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-xs)', flexWrap: 'wrap' }}
          >
            {tabs.map((tab) => {
              const isActive = surfaceWorld === 'light-editor' && tab.id === activeTabId;
              return (
                <button
                  key={tab.id}
                  type="button"
                  onClick={() => {
                    setActiveTabId(tab.id);
                    setSurfaceWorld('light-editor');
                  }}
                  className={`mercury-nav-link ty-ui-sm ${isActive ? 'is-active' : ''}`}
                >
                  <span>{tab.title}</span>
                  <span className="ty-legal" style={{ color: 'var(--mercury-fog)' }}>
                    #{tab.roamingHash.slice(0, 4)}
                  </span>
                </button>
              );
            })}
          </nav>
        </div>

        {/* Right Nav Controls: Presence + Secondary Translucent Pill + ONE Solid Primary Pill */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-sm)' }}>
          <button
            type="button"
            onClick={() => {
              setInspectorOpen(true);
              setInspectorTab('roaming-history');
            }}
            className="mercury-button-tint-dark ty-ui-sm"
          >
            Roaming ({roamingSnapshots.length} Snapshots)
          </button>

          <button
            type="button"
            onClick={() => setInspectorOpen((o) => !o)}
            className="mercury-button-glass-dark ty-ui-sm"
          >
            {inspectorOpen ? 'Hide Studio' : 'Open WPS AI & Python'}
          </button>

          {/* Single Saturated #5266eb Action Pill on the View (`button-primary`) */}
          <button type="button" className="mercury-button-primary ty-ui">
            Export & Share
          </button>
        </div>
      </header>

      {/* ─── 2A. MERCURY NIGHT WORLD: LAUNCHPAD & TELEMETRY HERO (#171721) ─── */}
      {surfaceWorld === 'night-launchpad' ? (
        <main
          style={{
            flex: 1,
            backgroundColor: 'var(--mercury-night)',
            color: 'var(--mercury-on-dark)',
            padding: 'var(--spacing-section) var(--spacing-xxl)',
          }}
        >
          <div style={{ maxWidth: 1160, margin: '0 auto' }}>
            {/* Centered Hero Headline (~600px) */}
            <div style={{ maxWidth: 680, margin: '0 auto', textAlign: 'center' }}>
              <div className="ty-eyebrow" style={{ color: 'var(--mercury-fog)', marginBottom: 'var(--spacing-md)' }}>
                RUST + WEBASSEMBLY + PYTHON + TAURI 2.0
              </div>
              <h1 className="ty-display" style={{ margin: '0 0 var(--spacing-lg) 0', color: 'var(--mercury-on-dark)' }}>
                Quiet precision for every document, model, and deck.
              </h1>
              <p className="ty-body-lg" style={{ color: 'var(--mercury-fog)', margin: '0 0 var(--spacing-xxl) 0' }}>
                Four native Rust engines—Writer, Spreadsheets, Presentation, and ISO 32000 PDF—united in one
                single-process workspace with AST-grounded AI and zero-copy Python.
              </p>

              {/* Hero Fused Command / AI Capsule (`email-capture` + `text-input` + `button-primary`) */}
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  setSurfaceWorld('light-editor');
                }}
                className="mercury-fused-capsule"
                style={{ maxWidth: 560, margin: '0 auto var(--spacing-huge) auto' }}
              >
                <input
                  type="text"
                  className="mercury-text-input ty-ui"
                  placeholder="Ask @WPS-AI to draft a spec, build a Pivot, or convert a PDF..."
                  value={capsuleInput}
                  onChange={(e) => setCapsuleInput(e.target.value)}
                />
                <button type="submit" className="mercury-button-primary ty-ui">
                  Launch Studio
                </button>
              </form>
            </div>

            {/* 4-Up Stat Block Row (`stat-block`: 60px arcadiaDisplay weight 480 over fog caption) */}
            <section
              aria-label="Engine Telemetry Stats"
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
                gap: 'var(--spacing-xl)',
                padding: 'var(--spacing-huge) 0',
                borderTop: '1px solid var(--mercury-slate)',
                borderBottom: '1px solid var(--mercury-slate)',
                marginBottom: 'var(--spacing-section)',
              }}
            >
              <div>
                <div className="ty-stat">120fps</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  Virtualized Canvas & Cosmic-Text Shaping
                </div>
              </div>
              <div>
                <div className="ty-stat">150+</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  Reactive DAG Formulas & =PY() DataFrames
                </div>
              </div>
              <div>
                <div className="ty-stat">1.04M</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  Sparse Matrix Rows per Worksheet
                </div>
              </div>
              <div>
                <div className="ty-stat">AES-256</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  True Byte-Stripping PDF Redaction & Crypto
                </div>
              </div>
            </section>

            {/* 3-Up Feature Cards on Night Raised (#1e1e2a, rounded.md = 12px, spacing.xxl = 32px padding) */}
            <section
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
                gap: 'var(--spacing-xl)',
              }}
            >
              {tabs.map((tab) => (
                <article key={tab.id} className="mercury-feature-card-dark">
                  <div className="ty-eyebrow" style={{ color: 'var(--mercury-fog)', marginBottom: 'var(--spacing-sm)' }}>
                    {tab.module.toUpperCase()} ENGINE • SHA #{tab.roamingHash}
                  </div>
                  <h2 className="ty-heading-2" style={{ margin: '0 0 var(--spacing-sm) 0' }}>
                    {tab.title}
                  </h2>
                  <p className="ty-body-lg" style={{ color: 'var(--mercury-fog)', margin: '0 0 var(--spacing-xl) 0' }}>
                    {tab.module === 'writer' &&
                      'Ropey UTF-8 piece buffer, cascading styles, multi-column paginator, and Track Changes redlines.'}
                    {tab.module === 'sheets' &&
                      'Petgraph dependency DAG, Tarjan cycle detection, Smart Toolbox, Pivot cube, and =PY() spilling.'}
                    {tab.module === 'slides' &&
                      '3-tier SlideMaster inheritance, 50+ DrawingML vector shapes, A* smart connectors, and Morph.'}
                    {tab.module === 'pdf' &&
                      'ISO 32000 stream interpreter, XY-Cut & DBSCAN table reconstruction, inline reflow, and BBox RAG.'}
                  </p>
                  <button
                    type="button"
                    onClick={() => {
                      setActiveTabId(tab.id);
                      setSurfaceWorld('light-editor');
                    }}
                    className="mercury-button-glass-dark ty-ui"
                  >
                    Open in Light Workspace
                  </button>
                </article>
              ))}
            </section>
          </div>
        </main>
      ) : (
        /* ─── 2B. MERCURY LIGHT WORLD: FRAMED PRODUCT WORKSPACE (#fbfcfd) ─── */
        <>
          {/* Contextual Ribbon Bar (`surface-tint` #fbfbff + `button-small` 32px capsules) */}
          <section className="mercury-surface-tint-bar" style={{ padding: '12px 32px' }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                flexWrap: 'wrap',
                gap: 'var(--spacing-md)',
              }}
            >
              {/* Module-Specific Translucent 32px Capsule Tools (`button-small`) */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-xs)', flexWrap: 'wrap' }}>
                <span
                  className="ty-eyebrow"
                  style={{ color: 'var(--mercury-ink-secondary)', marginRight: 'var(--spacing-xs)' }}
                >
                  {activeTab.module.toUpperCase()} TOOLS
                </span>

                {activeTab.module === 'writer' && (
                  <>
                    <button
                      type="button"
                      onClick={() => setTrackChangesOn((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${trackChangesOn ? 'is-active' : ''}`}
                    >
                      Track Changes: {trackChangesOn ? 'ON' : 'OFF'}
                    </button>
                    <button
                      type="button"
                      onClick={handleAcceptWriterRedlines}
                      className="mercury-button-small ty-ui-sm"
                    >
                      Accept @WriterAI Redlines
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      Auto-Style Headings
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      Mail Merge (CSV)
                    </button>
                  </>
                )}

                {activeTab.module === 'sheets' && (
                  <>
                    <button
                      type="button"
                      onClick={() => setReadingSpotlight((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${readingSpotlight ? 'is-active' : ''}`}
                    >
                      Reading View Spotlight
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      Smart Toolbox: Merge Keep All
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      Split Sheet by Column
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        setInspectorOpen(true);
                        setInspectorTab('python-studio');
                      }}
                      className="mercury-button-small ty-ui-sm"
                    >
                      =PY() DataFrame Studio
                    </button>
                  </>
                )}

                {activeTab.module === 'slides' && (
                  <>
                    <button
                      type="button"
                      onClick={() => setMorphPreviewActive((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${morphPreviewActive ? 'is-active' : ''}`}
                    >
                      Morph Transition Preview ({morphPreviewActive ? 't = 0.50' : 't = 0.00'})
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      Insert DrawingML Shape
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      A* Orthogonal Connector
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      Dual-Screen Presenter View
                    </button>
                  </>
                )}

                {activeTab.module === 'pdf' && (
                  <>
                    <button
                      type="button"
                      onClick={() => setPdfInlineEditMode((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${pdfInlineEditMode ? 'is-active' : ''}`}
                    >
                      Edit PDF Like Word ({pdfInlineEditMode ? 'Active' : 'Read'})
                    </button>
                    <button
                      type="button"
                      onClick={() => handleConvertPdfToOfficeTab('writer')}
                      className="mercury-button-small ty-ui-sm"
                    >
                      XY-Cut PDF → Word (.docx)
                    </button>
                    <button
                      type="button"
                      onClick={() => handleConvertPdfToOfficeTab('sheets')}
                      className="mercury-button-small ty-ui-sm"
                    >
                      Extract Tables → Sheets (.xlsx)
                    </button>
                    <button type="button" className="mercury-button-small ty-ui-sm">
                      Scan & Redact SSN/PII
                    </button>
                  </>
                )}
              </div>

              {/* Fused Formula / =PY() / AI Command Bar (`email-capture` 46px capsule) */}
              <form
                onSubmit={handleExecuteCapsule}
                className="mercury-fused-capsule"
                style={{ maxWidth: 520 }}
              >
                <input
                  type="text"
                  className="mercury-text-input ty-ui-sm"
                  style={{ fontFamily: 'var(--font-mono)' }}
                  value={capsuleInput}
                  onChange={(e) => setCapsuleInput(e.target.value)}
                  placeholder="Enter =FORMULA(), =PY(...), or /ai prompt..."
                />
                <button
                  type="submit"
                  className="mercury-button-tint-light ty-ui-sm"
                  style={{ height: 38, marginRight: 2 }}
                >
                  Run AST
                </button>
              </form>
            </div>
          </section>

          {/* ─── MAIN WORKSPACE BODY: CENTER CANVAS + LAVENDER INSPECTOR DRAWER ─── */}
          <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
            {/* Center Document / Grid / Slide / PDF Viewport (`#fbfcfd` canvas) */}
            <main
              style={{
                flex: 1,
                padding: 'var(--spacing-xxl)',
                overflowY: 'auto',
                backgroundColor: 'var(--mercury-canvas)',
              }}
            >
              {/* MODULE 1: WPS WRITER CANVAS */}
              {activeTab.module === 'writer' && (
                <div
                  style={{
                    maxWidth: 820,
                    margin: '0 auto',
                    backgroundColor: 'var(--mercury-surface)',
                    borderRadius: 'var(--rounded-md)',
                    padding: 'var(--spacing-huge) 56px',
                    boxShadow: 'var(--mercury-shadow-soft-stack)',
                  }}
                >
                  <div
                    className="ty-eyebrow"
                    style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-md)' }}
                  >
                    SECTION 1 • US LETTER (12,240 × 15,840 TWIPS) • ROPEY INTERVAL DOM
                  </div>

                  {writerParas.map((para) => (
                    <div
                      key={para.id}
                      style={{
                        marginBottom: 'var(--spacing-lg)',
                        paddingLeft: trackChangesOn && para.aiRedlineInsert ? 'var(--spacing-md)' : 0,
                        borderLeft:
                          trackChangesOn && para.aiRedlineInsert
                            ? '3px solid var(--mercury-primary)'
                            : '3px solid transparent',
                      }}
                    >
                      {para.style === 'Title' && (
                        <h1 className="ty-heading-1" style={{ margin: 0 }}>
                          {para.text}
                        </h1>
                      )}
                      {para.style === 'Heading 1' && (
                        <h2 className="ty-heading-2" style={{ margin: 0 }}>
                          {para.text}
                        </h2>
                      )}
                      {(para.style === 'Normal' || para.style === 'List Paragraph') && (
                        <p className="ty-body-lg" style={{ margin: 0 }}>
                          {para.style === 'List Paragraph' ? '• ' : ''}
                          {trackChangesOn && para.aiRedlineDelete && para.aiRedlineInsert ? (
                            <>
                              <span>We </span>
                              <del
                                style={{
                                  color: 'var(--mercury-ink-secondary)',
                                  backgroundColor: 'var(--mercury-lavender)',
                                  padding: '0 4px',
                                  borderRadius: 'var(--rounded-xs)',
                                }}
                              >
                                {para.aiRedlineDelete}
                              </del>{' '}
                              <ins
                                style={{
                                  color: 'var(--mercury-primary)',
                                  textDecoration: 'none',
                                  backgroundColor: 'rgba(82, 102, 235, 0.1)',
                                  padding: '0 6px',
                                  borderRadius: 'var(--rounded-xs)',
                                }}
                              >
                                {para.aiRedlineInsert}
                              </ins>
                              <span>
                                {' '}
                                render documents, spreadsheets, presentations, and PDF streams at 120 frames per
                                second.
                              </span>
                            </>
                          ) : (
                            para.text
                          )}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}

              {/* MODULE 2: WPS SPREADSHEETS VIRTUALIZED GRID (`ex-data-table-cell`) */}
              {activeTab.module === 'sheets' && (
                <div
                  style={{
                    maxWidth: 1080,
                    margin: '0 auto',
                    backgroundColor: 'var(--mercury-surface)',
                    borderRadius: 'var(--rounded-md)',
                    padding: 'var(--spacing-xl)',
                    boxShadow: 'var(--mercury-shadow-soft-stack)',
                  }}
                >
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      marginBottom: 'var(--spacing-md)',
                    }}
                  >
                    <span className="ty-eyebrow" style={{ color: 'var(--mercury-ink-secondary)' }}>
                      SPARSE MATRIX (1,048,576 × 16,384) • ACTIVE CELL:{' '}
                      {String.fromCharCode(65 + selectedCell[1])}
                      {selectedCell[0] + 1}
                    </span>
                    <span className="ty-ui-sm" style={{ color: 'var(--mercury-ink-secondary)' }}>
                      SUM(D2:D6) = $2,379,000 • AVG = $475,800
                    </span>
                  </div>

                  <table className="mercury-data-table">
                    <thead>
                      <tr>
                        <th className="ty-eyebrow" style={{ width: 48 }}>
                          #
                        </th>
                        {sheetGrid[0].map((colHeader, cIdx) => (
                          <th key={cIdx} className="ty-eyebrow">
                            {String.fromCharCode(65 + cIdx)} • {colHeader}
                          </th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {sheetGrid.slice(1).map((row, rOffset) => {
                        const rIdx = rOffset + 1;
                        const isRowSpotlit = readingSpotlight && selectedCell[0] === rIdx;
                        return (
                          <tr
                            key={rIdx}
                            style={{
                              backgroundColor: isRowSpotlit ? 'var(--mercury-lavender)' : undefined,
                            }}
                          >
                            <td
                              className="ty-ui-sm"
                              style={{ color: 'var(--mercury-ink-secondary)', textAlign: 'center' }}
                            >
                              {rIdx + 1}
                            </td>
                            {row.map((cellVal, cIdx) => {
                              const isSelected =
                                selectedCell[0] === rIdx && selectedCell[1] === cIdx;
                              return (
                                <td
                                  key={cIdx}
                                  onClick={() => setSelectedCell([rIdx, cIdx])}
                                  className={`ty-body ${isSelected ? 'is-selected' : ''}`}
                                  style={{ cursor: 'cell' }}
                                >
                                  {cellVal}
                                </td>
                              );
                            })}
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
              )}

              {/* MODULE 3: WPS PRESENTATION 16:9 VECTOR STAGE & MORPH INTERPOLATOR */}
              {activeTab.module === 'slides' && (
                <div style={{ maxWidth: 1040, margin: '0 auto' }}>
                  <div
                    style={{
                      display: 'flex',
                      gap: 'var(--spacing-sm)',
                      marginBottom: 'var(--spacing-md)',
                    }}
                  >
                    {['Slide 1: Title Hero (!!hero)', 'Slide 2: Process Timeline (Morph)'].map(
                      (label, idx) => (
                        <button
                          key={label}
                          type="button"
                          onClick={() => setActiveSlideIndex(idx)}
                          className={`mercury-button-small ty-ui-sm ${
                            activeSlideIndex === idx ? 'is-active' : ''
                          }`}
                        >
                          {label}
                        </button>
                      )
                    )}
                  </div>

                  {/* 16:9 Widescreen Stage (12,192,000 x 6,858,000 EMUs) */}
                  <div
                    className="mercury-feature-card-dark"
                    style={{
                      minHeight: 460,
                      display: 'flex',
                      flexDirection: 'column',
                      justifyContent: 'space-between',
                      position: 'relative',
                      overflow: 'hidden',
                    }}
                  >
                    <div>
                      <div className="ty-eyebrow" style={{ color: 'var(--mercury-periwinkle)' }}>
                        SLIDEMASTER: MERCURY NIGHT • MORPH TAG: !!HERO_CARD
                      </div>
                      <h2
                        className="ty-display"
                        style={{
                          margin: 'var(--spacing-md) 0',
                          transform: morphPreviewActive ? 'translateX(24px) scale(0.94)' : 'none',
                          transition: 'transform 420ms cubic-bezier(0.22, 1, 0.36, 1)',
                        }}
                      >
                        {activeSlideIndex === 0
                          ? 'Unified Rust + Wasm Productivity Suite'
                          : 'A* Smart Connectors & Morph Timeline'}
                      </h2>
                    </div>

                    {/* 3 DrawingML Vector Pillar Cards linked by A* Connectors */}
                    <div
                      style={{
                        display: 'grid',
                        gridTemplateColumns: 'repeat(3, 1fr)',
                        gap: 'var(--spacing-lg)',
                        marginTop: 'var(--spacing-xl)',
                      }}
                    >
                      {[
                        '1. OOXML ZIP VFS Streamer',
                        '2. Cosmic-Text & Kurbo Paths',
                        '3. Yrs CRDT Live Sync',
                      ].map((stepTitle, sIdx) => (
                        <div
                          key={stepTitle}
                          style={{
                            backgroundColor: 'var(--mercury-night)',
                            border: '1px solid var(--mercury-glass-dark-border)',
                            borderRadius: 'var(--rounded-md)',
                            padding: 'var(--spacing-lg)',
                            transform: morphPreviewActive
                              ? `translateY(${(sIdx % 2 === 0 ? -1 : 1) * 10}px)`
                              : 'none',
                            transition: 'transform 420ms ease',
                          }}
                        >
                          <div className="ty-eyebrow" style={{ color: 'var(--mercury-periwinkle)' }}>
                            NODE !!STEP_{sIdx + 1}
                          </div>
                          <div className="ty-lead" style={{ marginTop: 'var(--spacing-xs)' }}>
                            {stepTitle}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}

              {/* MODULE 4: WPS PDF TOOLKIT & SPATIAL BBOX RAG CANVAS */}
              {activeTab.module === 'pdf' && (
                <div
                  style={{
                    maxWidth: 820,
                    margin: '0 auto',
                    backgroundColor: 'var(--mercury-surface)',
                    borderRadius: 'var(--rounded-md)',
                    padding: 'var(--spacing-huge) 56px',
                    boxShadow: 'var(--mercury-shadow-soft-stack)',
                  }}
                >
                  <div
                    className="ty-eyebrow"
                    style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-lg)' }}
                  >
                    ISO 32000 PAGE 1 • XY-CUT & DBSCAN RECONSTRUCTED BLOCKS
                  </div>

                  {INITIAL_PDF_BLOCKS.map((block) => {
                    const isCited = highlightedPdfBlockId === block.blockId;
                    return (
                      <div
                        key={block.blockId}
                        onClick={() => setHighlightedPdfBlockId(block.blockId)}
                        style={{
                          padding: 'var(--spacing-md)',
                          marginBottom: 'var(--spacing-md)',
                          borderRadius: 'var(--rounded-sm)',
                          backgroundColor: isCited ? 'var(--mercury-lavender)' : 'transparent',
                          border: isCited
                            ? '1px solid var(--mercury-primary)'
                            : pdfInlineEditMode
                            ? '1px dashed var(--mercury-fog)'
                            : '1px solid transparent',
                          cursor: 'pointer',
                          transition: 'background-color 160ms ease',
                        }}
                      >
                        <div
                          className="ty-eyebrow"
                          style={{
                            color: isCited ? 'var(--mercury-primary)' : 'var(--mercury-ink-secondary)',
                            marginBottom: 6,
                          }}
                        >
                          {block.role} • BLOCK #{block.blockId} • BBOX [{block.bbox.join(', ')}]
                        </div>
                        <div
                          contentEditable={pdfInlineEditMode}
                          suppressContentEditableWarning
                          className={block.role === 'Heading' ? 'ty-heading-2' : 'ty-body-lg'}
                          style={{ outline: 'none' }}
                        >
                          {block.text}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </main>

            {/* ─── 3. RIGHT DOCKABLE LAVENDER INSPECTOR DRAWER (`feature-card-light`) ─── */}
            {inspectorOpen && (
              <aside
                aria-label="WPS AI, Python & Roaming Studio"
                style={{
                  width: 390,
                  backgroundColor: 'var(--mercury-surface-tint)',
                  borderLeft: '1px solid var(--mercury-fog)',
                  padding: 'var(--spacing-xl)',
                  overflowY: 'auto',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 'var(--spacing-lg)',
                }}
              >
                {/* Drawer Pill Switcher (`button-small`) */}
                <div style={{ display: 'flex', gap: 'var(--spacing-xs)', flexWrap: 'wrap' }}>
                  <button
                    type="button"
                    onClick={() => setInspectorTab('ai-copilot')}
                    className={`mercury-button-small ty-ui-sm ${
                      inspectorTab === 'ai-copilot' ? 'is-active' : ''
                    }`}
                  >
                    @WPS-AI
                  </button>
                  <button
                    type="button"
                    onClick={() => setInspectorTab('python-studio')}
                    className={`mercury-button-small ty-ui-sm ${
                      inspectorTab === 'python-studio' ? 'is-active' : ''
                    }`}
                  >
                    Python (=PY)
                  </button>
                  <button
                    type="button"
                    onClick={() => setInspectorTab('roaming-history')}
                    className={`mercury-button-small ty-ui-sm ${
                      inspectorTab === 'roaming-history' ? 'is-active' : ''
                    }`}
                  >
                    File Roaming
                  </button>
                </div>

                {/* Lavender Feature Card Container (`#edecfb`, `rounded.md` = 12px, `spacing.xxl` = 32px) */}
                {inspectorTab === 'ai-copilot' && (
                  <div className="mercury-feature-card-light">
                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-xs)' }}
                    >
                      AST-GROUNDED CO-PILOT • @{activeTab.module.toUpperCase()}AI
                    </div>
                    <h3 className="ty-heading-2" style={{ margin: '0 0 var(--spacing-sm) 0' }}>
                      Contextual Intelligence
                    </h3>
                    <p
                      className="ty-body-lg"
                      style={{ color: 'var(--mercury-ink-secondary)', margin: '0 0 var(--spacing-lg) 0' }}
                    >
                      Every suggestion mutates the underlying Rust AST or cites exact PDF page coordinates.
                    </p>

                    {/* Toggle Switch for Track Changes Redline Mode */}
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        marginBottom: 'var(--spacing-lg)',
                      }}
                    >
                      <span className="ty-ui-sm" style={{ color: 'var(--mercury-ink-secondary)' }}>
                        Emit AI Edits as Track Changes
                      </span>
                      <button
                        type="button"
                        aria-label="Toggle AI Track Changes"
                        onClick={() => setTrackChangesOn((v) => !v)}
                        className={`mercury-toggle-switch ${trackChangesOn ? 'is-on' : ''}`}
                      >
                        <span className="mercury-toggle-knob" />
                      </button>
                    </div>

                    {/* Accordion Action Rows (`accordion-row` with 1px `#c3c3cc` fog rules) */}
                    <div>
                      <div className="mercury-accordion-row ty-ui-sm">
                        <span>@WriterAI: Concise Redline Diff</span>
                        <button
                          type="button"
                          onClick={handleAcceptWriterRedlines}
                          className="mercury-button-small ty-ui-sm"
                        >
                          Accept
                        </button>
                      </div>
                      <div className="mercury-accordion-row ty-ui-sm">
                        <span>@SheetsAI: Anomaly in C9 (+3.4σ)</span>
                        <button
                          type="button"
                          onClick={() => {
                            setActiveTabId('tab-sheets-1');
                            setSelectedCell([1, 3]);
                          }}
                          className="mercury-button-small ty-ui-sm"
                        >
                          Inspect
                        </button>
                      </div>
                      <div className="mercury-accordion-row ty-ui-sm">
                        <span>@PdfAI: Termination Fee ($45,000)</span>
                        <button
                          type="button"
                          onClick={() => {
                            setActiveTabId('tab-pdf-1');
                            setHighlightedPdfBlockId(205);
                          }}
                          className="mercury-button-small ty-ui-sm"
                        >
                          Page 1 • BBox
                        </button>
                      </div>
                    </div>
                  </div>
                )}

                {inspectorTab === 'python-studio' && (
                  <div className="mercury-feature-card-light">
                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-xs)' }}
                    >
                      PYO3 (DESKTOP) + PYODIDE (WASM)
                    </div>
                    <h3 className="ty-heading-2" style={{ margin: '0 0 var(--spacing-sm) 0' }}>
                      =PY() DataFrame Bridge
                    </h3>
                    <pre
                      style={{
                        backgroundColor: 'var(--mercury-surface)',
                        padding: 'var(--spacing-md)',
                        borderRadius: 'var(--rounded-sm)',
                        fontFamily: 'var(--font-mono)',
                        fontSize: 13,
                        overflowX: 'auto',
                        border: '1px solid var(--mercury-fog)',
                      }}
                    >
                      {`df = wp.range("A1:E6").to_df()\nres = df.groupby("Region")["Revenue"].sum()\nreturn res`}
                    </pre>
                    <div className="mercury-accordion-row ty-ui-sm">
                      <span>APAC Total (Spilled)</span>
                      <span>$1,452,000</span>
                    </div>
                    <div className="mercury-accordion-row ty-ui-sm">
                      <span>AMER Total (Spilled)</span>
                      <span>$573,000</span>
                    </div>
                    <div className="mercury-accordion-row ty-ui-sm">
                      <span>EMEA Total (Spilled)</span>
                      <span>$354,000</span>
                    </div>
                  </div>
                )}

                {inspectorTab === 'roaming-history' && (
                  <div className="mercury-feature-card-light">
                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-xs)' }}
                    >
                      YRS CRDT & SHA-256 CONTENT STORE
                    </div>
                    <h3 className="ty-heading-2" style={{ margin: '0 0 var(--spacing-sm) 0' }}>
                      File Roaming Snapshots
                    </h3>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--spacing-xs)' }}>
                      {roamingSnapshots.map((snap, idx) => (
                        <div
                          key={snap.versionId}
                          className={`mercury-app-shell-row ty-ui-sm ${idx === 0 ? 'is-active' : ''}`}
                        >
                          <div>
                            <div>{snap.label}</div>
                            <div className="ty-legal" style={{ color: 'var(--mercury-ink-secondary)' }}>
                              {snap.timestamp} • #{snap.sha256Short}
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </aside>
            )}
          </div>
        </>
      )}

      {/* ─── 4. MERCURY SITE FOOTER / STATUS BAR (Night #171721 Framing Band) ─── */}
      <footer className="mercury-site-footer ty-legal">
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-lg)' }}>
          <span>ENGINE: RUST + WASM32 + TAURI 2.0</span>
          <span style={{ color: 'var(--mercury-fog)' }}>
            ACTIVE TAB: {activeTab.title.toUpperCase()} ({activeTab.module.toUpperCase()})
          </span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-lg)', color: 'var(--mercury-fog)' }}>
          <span>DESIGN SYSTEM: MERCURY (#171721 NIGHT / #FBFCFD CANVAS / #5266EB INDIGO)</span>
          <span>CRDT SYNC: ONLINE</span>
        </div>
      </footer>
    </div>
  );
};

export default MercuryWorkspaceShell;
