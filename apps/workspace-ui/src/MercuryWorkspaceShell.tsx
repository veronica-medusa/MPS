import React, { useState, useMemo, useCallback, useRef, useEffect } from 'react';
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

export interface SlideCard {
  slideIndex: number;
  title: string;
  morphTag: string;
  pillars: string[];
  speakerNotes: string;
}

export interface PdfSpatialBlock {
  blockId: number;
  page: number;
  role: 'Heading' | 'Paragraph' | 'Table';
  bbox: [number, number, number, number];
  text: string;
  redacted?: boolean;
}

export interface RoamingSnapshot {
  versionId: number;
  timestamp: string;
  author: string;
  label: string;
  sha256Short: string;
  savedState?: string;
}

export interface AiChatMessage {
  id: string;
  sender: 'user' | 'assistant';
  content: string;
  timestamp: string;
  actionTag?: string;
}

// ============================================================================
// DETERMINISTIC INITIAL DATASETS
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
  {
    id: 5,
    style: 'Heading 2',
    text: '2. High-Performance Memory Subsystems',
  },
  {
    id: 6,
    style: 'Normal',
    text: 'The Rope UTF-8 piece buffer enables O(log N) insertions, edits, and deletions without re-allocating contiguous strings.',
  },
];

const INITIAL_SHEET_GRID: string[][] = [
  ['Region', 'Quarter', 'Units', 'Revenue', 'Margin %', 'AI / Formula Output'],
  ['APAC', 'Q1', '1,420', '$426,000', '34.2%', '=SUMIFS(D2:D6, A2:A6, "APAC")'],
  ['EMEA', 'Q1', '1,180', '$354,000', '31.8%', '$1,452,000'],
  ['APAC', 'Q2', '1,650', '$495,000', '36.5%', '$495,000'],
  ['AMER', 'Q2', '1,910', '$573,000', '38.0%', '$573,000'],
  ['APAC', 'Q3', '1,770', '$531,000', '37.1%', '$531,000'],
];

const INITIAL_SLIDES: SlideCard[] = [
  {
    slideIndex: 0,
    title: 'Unified Rust + Wasm Productivity Suite',
    morphTag: '!!HERO_CARD',
    pillars: [
      '1. OOXML ZIP VFS Streamer',
      '2. Cosmic-Text & Kurbo Paths',
      '3. Yrs CRDT Live Sync',
    ],
    speakerNotes: 'Welcome the board. Emphasize that OpenWPS eliminates Electron overhead completely, delivering 120fps across all four productivity engines.',
  },
  {
    slideIndex: 1,
    title: 'A* Smart Connectors & Morph Timeline',
    morphTag: '!!HERO_CARD',
    pillars: [
      '1. 3-Tier SlideMaster Graph',
      '2. 50+ Parametric DrawingML Shapes',
      '3. Dual-Screen Presenter Cockpit',
    ],
    speakerNotes: 'Demonstrate real-time 120fps Morph transition interpolation between complex vector node layouts without rasterization artifacts.',
  },
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
    text: 'Section 4.2 Data Residency & Encryption: All customer archives are encrypted using AES-256 Revision 6 (V=5, R=6) with cryptographic PKCS#7 /ByteRange digital signatures. Technical contact SSN: 999-00-1234.',
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
  { versionId: 4, timestamp: 'Today, 20:55', author: 'Fiza Syed', label: 'Applied @WriterAI Track Changes Redline', sha256Short: '8f91a2c4' },
  { versionId: 3, timestamp: 'Today, 19:40', author: 'Fiza Syed', label: 'Executed =PY() APAC GroupBy Spill', sha256Short: '3d7e40b1' },
  { versionId: 2, timestamp: 'Today, 18:15', author: 'Fiza Syed', label: 'PDF XY-Cut Table Reconstruction', sha256Short: '7a115de9' },
  { versionId: 1, timestamp: 'Today, 16:00', author: 'Fiza Syed', label: 'Initial ECMA-376 Workspace Import', sha256Short: '1b44c902' },
];

const INITIAL_AI_MESSAGES: AiChatMessage[] = [
  {
    id: 'msg-1',
    sender: 'assistant',
    content: 'Ready: @WriterAI, @SheetsAI, @SlidesAI, and @PdfAI connected to AST with zero-latency local intelligence.',
    timestamp: '20:50',
  },
];

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export const MercuryWorkspaceShell: React.FC = () => {
  // Navigation & Viewport Mode State
  const [surfaceWorld, setSurfaceWorld] = useState<SurfaceWorld>('light-editor');
  const [tabs, setTabs] = useState<WorkspaceTab[]>(INITIAL_TABS);
  const [activeTabId, setActiveTabId] = useState<string>('tab-writer-1');
  const [inspectorOpen, setInspectorOpen] = useState<boolean>(true);
  const [inspectorTab, setInspectorTab] = useState<InspectorTab>('ai-copilot');

  // Modals & Notifications
  const [presenterModalOpen, setPresenterModalOpen] = useState<boolean>(false);
  const [exportModalOpen, setExportModalOpen] = useState<boolean>(false);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // Writer Engine State
  const [writerParas, setWriterParas] = useState<WriterParagraph[]>(INITIAL_WRITER_PARAS);
  const [trackChangesOn, setTrackChangesOn] = useState<boolean>(true);

  // Sheets Engine State
  const [sheetGrid, setSheetGrid] = useState<string[][]>(INITIAL_SHEET_GRID);
  const [selectedCell, setSelectedCell] = useState<[number, number]>([1, 5]);
  const [readingSpotlight, setReadingSpotlight] = useState<boolean>(true);

  // Dedicated Formula / Command Bar Inputs
  const [formulaInput, setFormulaInput] = useState<string>('=SUMIFS(D2:D6, A2:A6, "APAC")');
  const [launchpadCapsuleInput, setLaunchpadCapsuleInput] = useState<string>(
    '=PY("df = wp.range(\'A1:E6\').to_df(); return df.groupby(\'Region\')[\'Revenue\'].sum()")'
  );

  // Slides Engine State
  const [slides, setSlides] = useState<SlideCard[]>(INITIAL_SLIDES);
  const [activeSlideIndex, setActiveSlideIndex] = useState<number>(0);
  const [morphPreviewActive, setMorphPreviewActive] = useState<boolean>(false);
  const [connectorsVisible, setConnectorsVisible] = useState<boolean>(true);

  // PDF Toolkit Engine State
  const [pdfBlocks, setPdfBlocks] = useState<PdfSpatialBlock[]>(INITIAL_PDF_BLOCKS);
  const [highlightedPdfBlockId, setHighlightedPdfBlockId] = useState<number | null>(205);
  const [pdfInlineEditMode, setPdfInlineEditMode] = useState<boolean>(true);

  // File Roaming & CRDT Snapshots
  const [roamingSnapshots, setRoamingSnapshots] = useState<RoamingSnapshot[]>(INITIAL_SNAPSHOTS);

  // Python Studio State
  const [pythonCode, setPythonCode] = useState<string>(
    `df = wp.range("A1:E6").to_df()\nres = df.groupby("Region")["Revenue"].sum()\nreturn res`
  );
  const [pythonSpillData, setPythonSpillData] = useState<Array<{ region: string; total: string }>>([
    { region: 'APAC', total: '$1,452,000' },
    { region: 'AMER', total: '$573,000' },
    { region: 'EMEA', total: '$354,000' },
  ]);
  const [pythonConsoleLog, setPythonConsoleLog] = useState<string>(
    'PyO3 execution: 1.2ms | Zero-copy arrow buffers mounted'
  );

  // WPS AI Interactive Chat State
  const [aiChatMessages, setAiChatMessages] = useState<AiChatMessage[]>(INITIAL_AI_MESSAGES);
  const [aiChatInput, setAiChatInput] = useState<string>('');

  // Active Tab Derived Reference
  const activeTab = useMemo(
    () => tabs.find((t) => t.id === activeTabId) || tabs[0],
    [tabs, activeTabId]
  );

  // Active Slide Derived Reference
  const activeSlide = useMemo(
    () => slides[activeSlideIndex] || slides[0],
    [slides, activeSlideIndex]
  );

  // Active Cell Coordinate Label (e.g., "F2")
  const activeCellCoord = useMemo(() => {
    const colLetter = String.fromCharCode(65 + selectedCell[1]);
    const rowNum = selectedCell[0] + 1;
    return `${colLetter}${rowNum}`;
  }, [selectedCell]);

  // Keep Formula Bar synced when active cell changes
  useEffect(() => {
    if (activeTab.module === 'sheets') {
      const [r, c] = selectedCell;
      if (sheetGrid[r] && sheetGrid[r][c] !== undefined) {
        setFormulaInput(sheetGrid[r][c]);
      }
    }
  }, [selectedCell, sheetGrid, activeTab.module]);

  // Toast Helper
  const showToast = useCallback((msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3200);
  }, []);

  // Record a deterministic Roaming Snapshot
  const recordSnapshot = useCallback(
    (label: string) => {
      const newVersion = roamingSnapshots.length + 1;
      const hex = Math.floor(Math.random() * 0xffffff)
        .toString(16)
        .padStart(6, '0');
      const shortSha = `${hex.slice(0, 4)}e9`;
      const snap: RoamingSnapshot = {
        versionId: newVersion,
        timestamp: 'Just now',
        author: 'Fiza Syed',
        label,
        sha256Short: shortSha,
      };
      setRoamingSnapshots((prev) => [snap, ...prev]);
      showToast(`Snapshot v${newVersion} recorded: ${label}`);
    },
    [roamingSnapshots.length, showToast]
  );

  // ============================================================================
  // TAB LIFECYCLE HANDLERS
  // ============================================================================

  const handleSelectTab = (tabId: string) => {
    setActiveTabId(tabId);
    setSurfaceWorld('light-editor');
  };

  const handleCloseTab = (tabId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (tabs.length <= 1) return;
    const remaining = tabs.filter((t) => t.id !== tabId);
    setTabs(remaining);
    if (activeTabId === tabId) {
      setActiveTabId(remaining[0].id);
    }
  };

  const handleCreateTab = (module: ModuleKind) => {
    const count = tabs.length + 1;
    const titles: Record<ModuleKind, string> = {
      writer: `Document_${count}.docx`,
      sheets: `Worksheet_${count}.xlsx`,
      slides: `Presentation_${count}.pptx`,
      pdf: `Document_${count}.pdf`,
    };
    const newId = `tab-${module}-${Date.now()}`;
    const newTab: WorkspaceTab = {
      id: newId,
      title: titles[module],
      module,
      dirty: false,
      roamingHash: Math.floor(Math.random() * 0xffffffff).toString(16).slice(0, 8),
    };
    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newId);
    setSurfaceWorld('light-editor');
    recordSnapshot(`Created New ${module.toUpperCase()} Document (${titles[module]})`);
  };

  // ============================================================================
  // WRITER ENGINE ACTIONS
  // ============================================================================

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
    recordSnapshot('Accepted @WriterAI Track Changes Redlines');
  };

  const handleRejectWriterRedlines = () => {
    setWriterParas((prev) =>
      prev.map((p) => ({
        ...p,
        aiRedlineDelete: undefined,
        aiRedlineInsert: undefined,
      }))
    );
    recordSnapshot('Rejected @WriterAI Track Changes Redlines');
  };

  const handleAutoStyleHeadings = () => {
    setWriterParas((prev) =>
      prev.map((p, idx) => {
        const trimmed = p.text.trim();
        let newStyle = p.style;
        if (idx === 0 && trimmed.split(/\s+/).length <= 8) {
          newStyle = 'Title';
        } else if (/^\d+\.\s+/.test(trimmed)) {
          newStyle = 'Heading 1';
        } else if (/^[•\-\*]\s+/.test(trimmed)) {
          newStyle = 'List Paragraph';
        }
        return { ...p, style: newStyle };
      })
    );
    recordSnapshot('Auto-Styled Document Headings via AST Intelligence');
  };

  const handleMailMerge = () => {
    showToast('Mail Merge (CSV): Injected 24 recipient records into section template.');
    recordSnapshot('Mail Merge: Applied CSV Customer Fields');
  };

  const handleUpdateWriterPara = (id: number, newText: string) => {
    setWriterParas((prev) =>
      prev.map((p) => (p.id === id ? { ...p, text: newText } : p))
    );
  };

  // ============================================================================
  // SHEETS ENGINE ACTIONS
  // ============================================================================

  const handleSelectCell = (rIdx: number, cIdx: number) => {
    setSelectedCell([rIdx, cIdx]);
  };

  const handleApplyFormula = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    const [r, c] = selectedCell;
    const trimmed = formulaInput.trim();
    let displayVal = trimmed;

    if (trimmed.startsWith('=PY(')) {
      displayVal = '$1,452,000';
    } else if (trimmed.startsWith('=SUM')) {
      displayVal = '$2,379,000';
    }

    setSheetGrid((prev) => {
      const next = prev.map((row) => [...row]);
      if (next[r] && next[r][c] !== undefined) {
        next[r][c] = displayVal;
      }
      return next;
    });

    recordSnapshot(`Updated Cell ${activeCellCoord} → ${displayVal}`);
  };

  const handleSmartToolboxMergeKeepAll = () => {
    const [r] = selectedCell;
    if (r >= 1 && r < sheetGrid.length) {
      const merged = `${sheetGrid[r][0]} (${sheetGrid[r][1]})`;
      setSheetGrid((prev) => {
        const next = prev.map((row) => [...row]);
        next[r][5] = merged;
        return next;
      });
      recordSnapshot(`Smart Toolbox: Merged Cells Without Loss into Row ${r + 1}`);
    }
  };

  const handleSmartToolboxSplitByRegion = () => {
    const newId = `tab-sheets-apac-${Date.now()}`;
    const newTab: WorkspaceTab = {
      id: newId,
      title: 'APAC_Split_Sheet.xlsx',
      module: 'sheets',
      dirty: true,
      roamingHash: '91c4e20b',
    };
    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newId);
    recordSnapshot('Smart Toolbox: Split Sheet by Region Column');
  };

  const handleAddSheetRow = () => {
    const newRow = ['AMER', 'Q3', '2,040', '$612,000', '39.1%', '$612,000'];
    setSheetGrid((prev) => [...prev, newRow]);
    recordSnapshot('Inserted Calculation Row into Sparse Matrix');
  };

  // ============================================================================
  // SLIDES ENGINE ACTIONS
  // ============================================================================

  const handleAddSlide = () => {
    const newIndex = slides.length;
    const newSlide: SlideCard = {
      slideIndex: newIndex,
      title: `Executive Roadmap & Milestones (Phase ${newIndex + 1})`,
      morphTag: '!!ROADMAP_CARD',
      pillars: [
        '1. Distributed Sharded Storage',
        '2. Zero-Copy WebGPU Pipelines',
        '3. Enterprise SSO & ISO Redaction',
      ],
      speakerNotes: 'Outline phase milestones and highlight sub-millisecond AST synchronization.',
    };
    setSlides((prev) => [...prev, newSlide]);
    setActiveSlideIndex(newIndex);
    recordSnapshot(`Added Vector Slide #${newIndex + 1} to Presentation Stage`);
  };

  const handleAddPillarNode = () => {
    setSlides((prev) =>
      prev.map((s, idx) => {
        if (idx !== activeSlideIndex) return s;
        return {
          ...s,
          pillars: [...s.pillars, `${s.pillars.length + 1}. Auto-Generated AST Vector Node`],
        };
      })
    );
    recordSnapshot(`Added DrawingML Vector Shape to Slide #${activeSlideIndex + 1}`);
  };

  const handleUpdateSpeakerNotes = (text: string) => {
    setSlides((prev) =>
      prev.map((s, idx) => (idx === activeSlideIndex ? { ...s, speakerNotes: text } : s))
    );
  };

  // ============================================================================
  // PDF TOOLKIT ENGINE ACTIONS
  // ============================================================================

  const handleConvertPdfToOfficeTab = (targetModule: 'writer' | 'sheets') => {
    const newId = `tab-conv-${Date.now()}`;
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
    recordSnapshot(`Converted ISO 32000 PDF → Native ${targetModule.toUpperCase()} Tab`);
  };

  const handlePdfScanAndRedactPii = () => {
    setPdfBlocks((prev) =>
      prev.map((b) => {
        if (b.text.includes('999-00-1234')) {
          return {
            ...b,
            text: b.text.replace('999-00-1234', '[REDACTED-SSN - AES-256 PURGED]'),
            redacted: true,
          };
        }
        return b;
      })
    );
    recordSnapshot('Executed Permanent Byte-Stripping PII & SSN Redaction');
    showToast('PDF Redaction: 1 SSN occurrence permanently wiped from raw content stream.');
  };

  const handleUpdatePdfBlockText = (blockId: number, newText: string) => {
    setPdfBlocks((prev) =>
      prev.map((b) => (b.blockId === blockId ? { ...b, text: newText } : b))
    );
  };

  // ============================================================================
  // PYTHON STUDIO EXECUTION
  // ============================================================================

  const handleRunPythonScript = () => {
    const t0 = performance.now();
    // Simulate real execution of groupby
    const t1 = performance.now();
    const elapsed = Math.max(0.8, Number((t1 - t0).toFixed(2)));
    setPythonConsoleLog(
      `PyO3 Desktop + Pyodide Wasm execution finished in ${elapsed}ms.\nDataFrame spill: 3 rows × 2 cols. Zero-copy buffer pinned.`
    );
    setPythonSpillData([
      { region: 'APAC', total: '$1,452,000' },
      { region: 'AMER', total: '$573,000' },
      { region: 'EMEA', total: '$354,000' },
    ]);
    recordSnapshot('Executed =PY() Python DataFrame Studio');
    showToast('Python: Script executed. DataFrame spill updated.');
  };

  // ============================================================================
  // WPS AI INTERACTIVE CHAT & COPILOT
  // ============================================================================

  const handleSendAiChat = (customPrompt?: string) => {
    const prompt = customPrompt || aiChatInput;
    if (!prompt.trim()) return;

    const userMsg: AiChatMessage = {
      id: `usr-${Date.now()}`,
      sender: 'user',
      content: prompt,
      timestamp: 'Now',
    };

    let reply = `Analyzed "${prompt}" against the active ${activeTab.module.toUpperCase()} AST. All structural constraints verified.`;

    if (activeTab.module === 'writer') {
      reply = `@WriterAI: Found opportunities to enhance executive clarity. Emitted Track Changes redlines for paragraph #3 with zero lexical regression.`;
    } else if (activeTab.module === 'sheets') {
      reply = `@SheetsAI: Calculated APAC aggregate revenue across Q1-Q3: $1,452,000 (35.9% margin). Cell anomaly in D5 verified benign.`;
    } else if (activeTab.module === 'slides') {
      reply = `@SlidesAI: Interpolated 120fps Morph curve tags across 3 DrawingML nodes. Layout matches 16:9 SlideMaster constraints.`;
    } else if (activeTab.module === 'pdf') {
      reply = `@PdfAI: Located termination clause in Section 8.4 (Page 1, BBox [50, 200, 540, 275]). Early buyout fee is $45,000.`;
    }

    const aiMsg: AiChatMessage = {
      id: `ai-${Date.now() + 1}`,
      sender: 'assistant',
      content: reply,
      timestamp: 'Now',
    };

    setAiChatMessages((prev) => [...prev, userMsg, aiMsg]);
    if (!customPrompt) setAiChatInput('');
    recordSnapshot(`Prompted @WPS-AI: "${prompt.slice(0, 32)}..."`);
  };

  // ============================================================================
  // FILE ROAMING TIME-TRAVEL RESTORATION
  // ============================================================================

  const handleRestoreSnapshot = (versionId: number) => {
    if (versionId === 1) {
      setWriterParas(INITIAL_WRITER_PARAS);
      setSheetGrid(INITIAL_SHEET_GRID);
      setPdfBlocks(INITIAL_PDF_BLOCKS);
      setSlides(INITIAL_SLIDES);
    }
    recordSnapshot(`Restored Workspace to Roaming Snapshot v${versionId}`);
    showToast(`Time Machine: Restored state from Snapshot v${versionId}`);
  };

  return (
    <div
      style={{
        minHeight: '100vh',
        height: '100vh',
        display: 'flex',
        flexDirection: 'column',
        backgroundColor:
          surfaceWorld === 'night-launchpad' ? 'var(--mercury-night)' : 'var(--mercury-canvas)',
        color:
          surfaceWorld === 'night-launchpad' ? 'var(--mercury-on-dark)' : 'var(--mercury-ink)',
        overflow: 'hidden',
      }}
    >
      {/* ─── TOAST NOTIFICATION BANNER ─── */}
      {toastMessage && (
        <div
          style={{
            position: 'fixed',
            bottom: 48,
            left: '50%',
            transform: 'translateX(-50%)',
            backgroundColor: 'var(--mercury-night)',
            color: 'var(--mercury-on-dark)',
            border: '1px solid var(--mercury-glass-dark-border)',
            borderRadius: 'var(--rounded-pill)',
            padding: '10px 24px',
            fontSize: 14,
            fontWeight: 420,
            boxShadow: '0 8px 32px rgba(23, 23, 33, 0.35)',
            zIndex: 100,
            display: 'flex',
            alignItems: 'center',
            gap: 10,
          }}
        >
          <span
            style={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              backgroundColor: 'var(--mercury-primary)',
            }}
          />
          <span>{toastMessage}</span>
        </div>
      )}

      {/* ─── 1. MERCURY NAVIGATION BAR (Exact 72px Fixed Night Bar) ─── */}
      <header className="mercury-nav-bar">
        {/* Left: Brand & World Switcher Capsule */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-md)', flexShrink: 0 }}>
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
              {surfaceWorld === 'night-launchpad' ? 'Night Launchpad' : 'Light Canvas'}
            </span>
          </button>
        </div>

        {/* Center: Polymorphic Document Tabs (Strict No-Wrap, Horizontally Scrollable) */}
        <nav aria-label="Open Document Tabs" className="mercury-nav-tabs">
          {tabs.map((tab) => {
            const isActive = surfaceWorld === 'light-editor' && tab.id === activeTabId;
            return (
              <button
                key={tab.id}
                type="button"
                onClick={() => handleSelectTab(tab.id)}
                className={`mercury-nav-link ty-ui-sm ${isActive ? 'is-active' : ''}`}
                style={{ flexShrink: 0 }}
              >
                <span>{tab.title}</span>
                <span className="ty-legal" style={{ color: 'var(--mercury-fog)' }}>
                  #{tab.roamingHash.slice(0, 4)}
                </span>
                {tabs.length > 1 && (
                  <span
                    onClick={(e) => handleCloseTab(tab.id, e)}
                    style={{
                      marginLeft: 4,
                      opacity: 0.6,
                      fontSize: 14,
                      lineHeight: 1,
                      cursor: 'pointer',
                    }}
                    title="Close Tab"
                  >
                    ×
                  </span>
                )}
              </button>
            );
          })}

          {/* Quick Add Tab Buttons */}
          <button
            type="button"
            onClick={() => handleCreateTab('writer')}
            className="mercury-button-glass-dark ty-ui-sm"
            style={{ height: 32, padding: '0 10px', flexShrink: 0 }}
            title="Create New Writer Document"
          >
            + DOCX
          </button>
          <button
            type="button"
            onClick={() => handleCreateTab('sheets')}
            className="mercury-button-glass-dark ty-ui-sm"
            style={{ height: 32, padding: '0 10px', flexShrink: 0 }}
            title="Create New Spreadsheets Sheet"
          >
            + XLSX
          </button>
          <button
            type="button"
            onClick={() => handleCreateTab('slides')}
            className="mercury-button-glass-dark ty-ui-sm"
            style={{ height: 32, padding: '0 10px', flexShrink: 0 }}
            title="Create New Slide Deck"
          >
            + PPTX
          </button>
        </nav>

        {/* Right Controls: Roaming + Studio Drawer Toggle + Single Primary Button */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spacing-sm)', flexShrink: 0 }}>
          <button
            type="button"
            onClick={() => {
              setInspectorOpen(true);
              setInspectorTab('roaming-history');
            }}
            className="mercury-button-tint-dark ty-ui-sm"
          >
            🕒 Roaming ({roamingSnapshots.length})
          </button>

          <button
            type="button"
            onClick={() => setInspectorOpen((o) => !o)}
            className="mercury-button-glass-dark ty-ui-sm"
          >
            {inspectorOpen ? 'Hide Studio' : 'Open Studio'}
          </button>

          {/* Exactly ONE solid #5266eb primary action button in this chrome row */}
          <button
            type="button"
            onClick={() => setExportModalOpen(true)}
            className="mercury-button-primary ty-ui"
          >
            Export & Share
          </button>
        </div>
      </header>

      {/* ─── 2A. NIGHT WORLD: LAUNCHPAD & TELEMETRY HERO (#171721) ─── */}
      {surfaceWorld === 'night-launchpad' ? (
        <main
          style={{
            flex: 1,
            backgroundColor: 'var(--mercury-night)',
            color: 'var(--mercury-on-dark)',
            padding: '24px 32px 32px 32px',
            overflowY: 'auto',
          }}
        >
          <div style={{ maxWidth: 1180, margin: '0 auto' }}>
            {/* Centered Hero Headline */}
            <div style={{ maxWidth: 720, margin: '0 auto', textAlign: 'center' }}>
              <div
                className="ty-eyebrow"
                style={{ color: 'var(--mercury-periwinkle)', marginBottom: 'var(--spacing-sm)' }}
              >
                RUST + WEBASSEMBLY + PYTHON + TAURI 2.0
              </div>
              <h1
                className="ty-display"
                style={{ margin: '0 0 var(--spacing-sm) 0', color: 'var(--mercury-on-dark)', fontSize: 44 }}
              >
                Quiet precision for every document, model, and deck.
              </h1>
              <p
                className="ty-body-lg"
                style={{ color: 'var(--mercury-fog)', margin: '0 0 var(--spacing-lg) 0', fontSize: 16 }}
              >
                Four native Rust engines—Writer, Spreadsheets, Presentation, and ISO 32000 PDF—united in one
                single-process workspace with AST-grounded AI and zero-copy Python.
              </p>

              {/* Fused Launchpad AI / Command Capsule */}
              <form
                onSubmit={(e) => {
                  e.preventDefault();
                  handleSendAiChat(launchpadCapsuleInput);
                  setSurfaceWorld('light-editor');
                }}
                className="mercury-fused-capsule"
                style={{ maxWidth: 620, margin: '0 auto var(--spacing-lg) auto' }}
              >
                <input
                  type="text"
                  className="mercury-text-input ty-ui"
                  placeholder="Ask @WPS-AI to draft a spec, build a Pivot, or convert a PDF..."
                  value={launchpadCapsuleInput}
                  onChange={(e) => setLaunchpadCapsuleInput(e.target.value)}
                />
                <button type="submit" className="mercury-button-primary ty-ui">
                  Launch Studio
                </button>
              </form>
            </div>

            {/* 4-Up Stat Block Row */}
            <section
              aria-label="Engine Telemetry Stats"
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(4, minmax(0, 1fr))',
                gap: 'var(--spacing-lg)',
                padding: 'var(--spacing-md) 0',
                borderTop: '1px solid var(--mercury-glass-dark-border)',
                borderBottom: '1px solid var(--mercury-glass-dark-border)',
                marginBottom: 'var(--spacing-lg)',
              }}
            >
              <div>
                <div className="ty-stat" style={{ fontSize: 48 }}>120fps</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  Virtualized Canvas & Cosmic-Text
                </div>
              </div>
              <div>
                <div className="ty-stat" style={{ fontSize: 48 }}>150+</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  Reactive DAG Formulas & =PY()
                </div>
              </div>
              <div>
                <div className="ty-stat" style={{ fontSize: 48 }}>1.04M</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  Sparse Matrix Rows per Worksheet
                </div>
              </div>
              <div>
                <div className="ty-stat" style={{ fontSize: 48 }}>AES-256</div>
                <div className="ty-ui-sm" style={{ color: 'var(--mercury-fog)', marginTop: 'var(--spacing-xs)' }}>
                  True Byte-Stripping PDF Redaction
                </div>
              </div>
            </section>

            {/* Balanced 4-Up Feature Cards Grid (Strict 4-Column on Desktop) */}
            <section className="mercury-launchpad-grid">
              {tabs.slice(0, 4).map((tab) => (
                <article
                  key={tab.id}
                  className="mercury-feature-card-dark"
                  style={{ display: 'flex', flexDirection: 'column', justifyContent: 'space-between' }}
                >
                  <div>
                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-periwinkle)', marginBottom: 'var(--spacing-sm)' }}
                    >
                      {tab.module.toUpperCase()} ENGINE • #{tab.roamingHash.slice(0, 4)}
                    </div>
                    <h2
                      className="ty-heading-2"
                      style={{
                        margin: '0 0 var(--spacing-sm) 0',
                        fontSize: 20,
                        whiteSpace: 'nowrap',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                      }}
                      title={tab.title}
                    >
                      {tab.title}
                    </h2>
                    <p
                      className="ty-body"
                      style={{ color: 'var(--mercury-fog)', margin: '0 0 var(--spacing-lg) 0', fontSize: 14 }}
                    >
                      {tab.module === 'writer' &&
                        'Ropey UTF-8 piece buffer, cascading styles, and Track Changes redlines.'}
                      {tab.module === 'sheets' &&
                        'Petgraph dependency DAG, Tarjan cycle detection, and =PY() spilling.'}
                      {tab.module === 'slides' &&
                        '3-tier SlideMaster inheritance, 50+ DrawingML shapes, and Morph.'}
                      {tab.module === 'pdf' &&
                        'ISO 32000 stream interpreter, XY-Cut table reconstruction, and BBox RAG.'}
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => {
                      setActiveTabId(tab.id);
                      setSurfaceWorld('light-editor');
                    }}
                    className="mercury-button-glass-dark ty-ui-sm"
                    style={{ width: '100%' }}
                  >
                    Open in Light Canvas
                  </button>
                </article>
              ))}
            </section>
          </div>
        </main>
      ) : (
        /* ─── 2B. LIGHT CANVAS WORLD: FRAMED WORKSPACE (#fbfcfd) ─── */
        <>
          {/* Contextual Ribbon Bar (`surface-tint` #fbfbff + `button-small` 32px capsules) */}
          <section className="mercury-surface-tint-bar" style={{ padding: '8px 24px', flexShrink: 0 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                flexWrap: 'nowrap',
                gap: 'var(--spacing-md)',
              }}
            >
              {/* Module-Specific Translucent 32px Capsule Tools */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 'var(--spacing-xs)',
                  flexWrap: 'nowrap',
                  overflowX: 'auto',
                }}
              >
                <span
                  className="ty-eyebrow"
                  style={{ color: 'var(--mercury-ink-secondary)', marginRight: 'var(--spacing-xs)', flexShrink: 0 }}
                >
                  {activeTab.module.toUpperCase()} TOOLS
                </span>

                {/* WRITER TOOLS */}
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
                      title="Accept all @WriterAI redline revisions into Rope buffer"
                    >
                      Accept Redlines
                    </button>
                    <button
                      type="button"
                      onClick={handleRejectWriterRedlines}
                      className="mercury-button-small ty-ui-sm"
                      title="Reject AI revisions and preserve original"
                    >
                      Reject Redlines
                    </button>
                    <button
                      type="button"
                      onClick={handleAutoStyleHeadings}
                      className="mercury-button-small ty-ui-sm"
                      title="Auto-detect and format headings"
                    >
                      Auto-Style Headings
                    </button>
                    <button
                      type="button"
                      onClick={handleMailMerge}
                      className="mercury-button-small ty-ui-sm"
                      title="Merge customer fields from CSV"
                    >
                      Mail Merge (CSV)
                    </button>
                  </>
                )}

                {/* SHEETS TOOLS */}
                {activeTab.module === 'sheets' && (
                  <>
                    <button
                      type="button"
                      onClick={() => setReadingSpotlight((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${readingSpotlight ? 'is-active' : ''}`}
                    >
                      Spotlight: {readingSpotlight ? 'ON' : 'OFF'}
                    </button>
                    <button
                      type="button"
                      onClick={handleSmartToolboxMergeKeepAll}
                      className="mercury-button-small ty-ui-sm"
                      title="Combine Region + Quarter without losing cell content"
                    >
                      Merge Keep All
                    </button>
                    <button
                      type="button"
                      onClick={handleSmartToolboxSplitByRegion}
                      className="mercury-button-small ty-ui-sm"
                      title="Split sheet by Region column into a new tab"
                    >
                      Split Sheet by Region
                    </button>
                    <button
                      type="button"
                      onClick={handleAddSheetRow}
                      className="mercury-button-small ty-ui-sm"
                      title="Insert a new calculation row"
                    >
                      + Insert Row
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        setInspectorOpen(true);
                        setInspectorTab('python-studio');
                      }}
                      className="mercury-button-small ty-ui-sm"
                    >
                      🐍 Python (=PY) Studio
                    </button>
                  </>
                )}

                {/* SLIDES TOOLS */}
                {activeTab.module === 'slides' && (
                  <>
                    <button
                      type="button"
                      onClick={() => setMorphPreviewActive((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${morphPreviewActive ? 'is-active' : ''}`}
                    >
                      Morph Preview: {morphPreviewActive ? 't = 0.50' : 't = 0.00'}
                    </button>
                    <button
                      type="button"
                      onClick={handleAddPillarNode}
                      className="mercury-button-small ty-ui-sm"
                    >
                      + DrawingML Shape
                    </button>
                    <button
                      type="button"
                      onClick={() => setConnectorsVisible((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${connectorsVisible ? 'is-active' : ''}`}
                    >
                      A* Connectors: {connectorsVisible ? 'ON' : 'OFF'}
                    </button>
                    <button
                      type="button"
                      onClick={() => setPresenterModalOpen(true)}
                      className="mercury-button-small ty-ui-sm"
                    >
                      Presenter View
                    </button>
                  </>
                )}

                {/* PDF TOOLS */}
                {activeTab.module === 'pdf' && (
                  <>
                    <button
                      type="button"
                      onClick={() => setPdfInlineEditMode((v) => !v)}
                      className={`mercury-button-small ty-ui-sm ${pdfInlineEditMode ? 'is-active' : ''}`}
                    >
                      Edit PDF: {pdfInlineEditMode ? 'Active' : 'Locked'}
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
                    <button
                      type="button"
                      onClick={handlePdfScanAndRedactPii}
                      className="mercury-button-small ty-ui-sm"
                    >
                      Scan & Redact SSN/PII
                    </button>
                  </>
                )}
              </div>

              {/* Right: Quick AI / Command Capsule (For non-Sheets tabs) */}
              {activeTab.module !== 'sheets' && (
                <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexShrink: 0 }}>
                  <form
                    onSubmit={(e) => {
                      e.preventDefault();
                      handleSendAiChat(launchpadCapsuleInput);
                    }}
                    className="mercury-fused-capsule"
                    style={{ width: 340, height: 36 }}
                  >
                    <input
                      type="text"
                      className="mercury-text-input ty-ui-sm"
                      placeholder="Ask @WPS-AI..."
                      value={launchpadCapsuleInput}
                      onChange={(e) => setLaunchpadCapsuleInput(e.target.value)}
                    />
                    <button
                      type="submit"
                      className="mercury-button-tint-light ty-ui-sm"
                      style={{ height: 30, marginRight: 2 }}
                    >
                      Run
                    </button>
                  </form>
                </div>
              )}
            </div>
          </section>

          {/* ─── DEDICATED FULL-WIDTH EXCEL-GRADE FORMULA BAR (ONLY IN SHEETS) ─── */}
          {activeTab.module === 'sheets' && (
            <div className="mercury-formula-bar">
              {/* Name Box (e.g., F2) */}
              <div className="mercury-name-box" title="Active Cell Coordinate">
                {activeCellCoord}
              </div>

              {/* fx Badge */}
              <div className="mercury-fx-badge" title="Insert Function">
                fx
              </div>

              {/* Formula / Cell Content Input */}
              <form
                onSubmit={handleApplyFormula}
                style={{ flex: 1, display: 'flex', gap: 8, alignItems: 'center' }}
              >
                <div className="mercury-formula-input-wrap">
                  <input
                    type="text"
                    className="mercury-formula-input"
                    value={formulaInput}
                    onChange={(e) => setFormulaInput(e.target.value)}
                    placeholder="Enter formula =SUM(), =PY(), or cell value..."
                  />
                </div>
                <button
                  type="submit"
                  className="mercury-button-tint-light ty-ui-sm"
                  style={{ height: 32, padding: '0 16px', flexShrink: 0 }}
                >
                  Run AST
                </button>
              </form>
            </div>
          )}

          {/* ─── MAIN WORKSPACE BODY: CENTER CANVAS + RIGHT STUDIO DRAWER ─── */}
          <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
            {/* Center Document / Grid / Slide / PDF Viewport */}
            <main
              style={{
                flex: 1,
                padding: 'var(--spacing-xl)',
                overflowY: 'auto',
                backgroundColor: 'var(--mercury-canvas)',
              }}
            >
              {/* ─── MODULE 1: WPS WRITER CANVAS ─── */}
              {activeTab.module === 'writer' && (
                <div className="mercury-paper-sheet">
                  {/* Margin Guide Hairline */}
                  <div className="mercury-paper-margin-ruler" />

                  <div
                    className="ty-eyebrow"
                    style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-xl)' }}
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
                        <h1
                          className="ty-heading-1"
                          contentEditable
                          suppressContentEditableWarning
                          onBlur={(e) => handleUpdateWriterPara(para.id, e.currentTarget.innerText)}
                          style={{ margin: 0, outline: 'none' }}
                        >
                          {para.text}
                        </h1>
                      )}
                      {para.style === 'Heading 1' && (
                        <h2
                          className="ty-heading-2"
                          contentEditable
                          suppressContentEditableWarning
                          onBlur={(e) => handleUpdateWriterPara(para.id, e.currentTarget.innerText)}
                          style={{ margin: 0, outline: 'none' }}
                        >
                          {para.text}
                        </h2>
                      )}
                      {para.style === 'Heading 2' && (
                        <h3
                          className="ty-lead"
                          contentEditable
                          suppressContentEditableWarning
                          onBlur={(e) => handleUpdateWriterPara(para.id, e.currentTarget.innerText)}
                          style={{ margin: 0, outline: 'none' }}
                        >
                          {para.text}
                        </h3>
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
                                  padding: '2px 6px',
                                  borderRadius: 'var(--rounded-xs)',
                                  textDecoration: 'line-through',
                                }}
                              >
                                {para.aiRedlineDelete}
                              </del>{' '}
                              <ins
                                style={{
                                  color: 'var(--mercury-primary)',
                                  textDecoration: 'none',
                                  backgroundColor: 'rgba(82, 102, 235, 0.12)',
                                  padding: '2px 8px',
                                  borderRadius: 'var(--rounded-xs)',
                                  borderBottom: '2px solid var(--mercury-primary)',
                                }}
                              >
                                {para.aiRedlineInsert}
                              </ins>
                              <span>
                                {' '}
                                render documents, spreadsheets, presentations, and PDF streams at 120 frames per
                                second.
                              </span>
                              <span className="mercury-redline-callout">
                                <button
                                  type="button"
                                  onClick={handleAcceptWriterRedlines}
                                  className="mercury-button-small ty-ui-sm"
                                  style={{ height: 24, padding: '0 8px', fontSize: 11 }}
                                >
                                  ✓ Accept
                                </button>
                                <button
                                  type="button"
                                  onClick={handleRejectWriterRedlines}
                                  className="mercury-button-small ty-ui-sm"
                                  style={{ height: 24, padding: '0 8px', fontSize: 11 }}
                                >
                                  ✗ Reject
                                </button>
                              </span>
                            </>
                          ) : (
                            <span
                              contentEditable
                              suppressContentEditableWarning
                              onBlur={(e) => handleUpdateWriterPara(para.id, e.currentTarget.innerText)}
                              style={{ outline: 'none' }}
                            >
                              {para.text}
                            </span>
                          )}
                        </p>
                      )}
                    </div>
                  ))}
                </div>
              )}

              {/* ─── MODULE 2: WPS SPREADSHEETS VIRTUALIZED GRID ─── */}
              {activeTab.module === 'sheets' && (
                <div style={{ maxWidth: 1140, margin: '0 auto' }}>
                  <div className="mercury-grid-viewport">
                    <table className="mercury-grid-table">
                      <thead>
                        <tr>
                          {/* Top-Left Corner Cell */}
                          <th className="mercury-grid-corner-cell" />
                          {sheetGrid[0].map((colTitle, cIdx) => {
                            const isColSpotlit = readingSpotlight && selectedCell[1] === cIdx;
                            return (
                              <th
                                key={cIdx}
                                className={`mercury-grid-col-header ${isColSpotlit ? 'is-spotlit' : ''}`}
                              >
                                {String.fromCharCode(65 + cIdx)} • {colTitle}
                              </th>
                            );
                          })}
                        </tr>
                      </thead>
                      <tbody>
                        {sheetGrid.slice(1).map((row, rOffset) => {
                          const rIdx = rOffset + 1;
                          const isRowSpotlit = readingSpotlight && selectedCell[0] === rIdx;
                          return (
                            <tr key={rIdx}>
                              {/* Sticky Row Number Header */}
                              <td
                                className={`mercury-grid-row-header ${isRowSpotlit ? 'is-spotlit' : ''}`}
                              >
                                {rIdx + 1}
                              </td>
                              {row.map((cellVal, cIdx) => {
                                const isSelected =
                                  selectedCell[0] === rIdx && selectedCell[1] === cIdx;
                                return (
                                  <td
                                    key={cIdx}
                                    onClick={() => handleSelectCell(rIdx, cIdx)}
                                    className={`mercury-grid-cell ${
                                      isRowSpotlit ? 'is-row-spotlit' : ''
                                    } ${isSelected ? 'is-selected' : ''}`}
                                  >
                                    {cellVal}
                                    {isSelected && <span className="mercury-cell-handle" />}
                                  </td>
                                );
                              })}
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>

                    {/* Bottom Sheet Tab Bar */}
                    <div className="mercury-sheet-tab-bar">
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span
                          className="mercury-button-small ty-ui-sm is-active"
                          style={{ height: 26, padding: '0 12px' }}
                        >
                          Sheet1
                        </span>
                        <button
                          type="button"
                          onClick={() => showToast('Created new Sheet2 tab in workbook')}
                          className="mercury-button-small ty-ui-sm"
                          style={{ height: 26, padding: '0 8px' }}
                        >
                          +
                        </button>
                      </div>
                      <div className="ty-legal" style={{ color: 'var(--mercury-ink-secondary)' }}>
                        SUM = $2,379,000 • AVERAGE = $475,800 • COUNT = 5 • 100% Zoom
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* ─── MODULE 3: WPS PRESENTATION VECTOR STAGE & FILMSTRIP ─── */}
              {activeTab.module === 'slides' && (
                <div className="mercury-slides-container">
                  {/* Left Filmstrip */}
                  <div className="mercury-filmstrip">
                    <div className="ty-eyebrow" style={{ color: 'var(--mercury-slate)', fontSize: 11 }}>
                      SLIDES ({slides.length})
                    </div>
                    {slides.map((s, idx) => (
                      <div
                        key={s.slideIndex}
                        onClick={() => setActiveSlideIndex(idx)}
                        className={`mercury-slide-thumb ${activeSlideIndex === idx ? 'is-active' : ''}`}
                      >
                        <div className="ty-legal" style={{ color: 'var(--mercury-periwinkle)' }}>
                          Slide {idx + 1}
                        </div>
                        <div
                          className="ty-ui-sm"
                          style={{
                            color: 'var(--mercury-on-dark)',
                            fontSize: 12,
                            whiteSpace: 'nowrap',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                          }}
                        >
                          {s.title}
                        </div>
                      </div>
                    ))}
                    <button
                      type="button"
                      onClick={handleAddSlide}
                      className="mercury-button-tint-light ty-ui-sm"
                      style={{ width: '100%', height: 32 }}
                    >
                      + Add Slide
                    </button>
                  </div>

                  {/* Main Stage & Notes */}
                  <div className="mercury-slide-main-stage">
                    <div className="mercury-16-9-viewport">
                      <div>
                        <div className="ty-eyebrow" style={{ color: 'var(--mercury-periwinkle)' }}>
                          SLIDEMASTER: MERCURY NIGHT • MORPH TAG: {activeSlide.morphTag}
                        </div>
                        <h2
                          className="ty-display"
                          style={{
                            margin: 'var(--spacing-md) 0',
                            fontSize: 36,
                            transform: morphPreviewActive ? 'translateX(20px) scale(0.96)' : 'none',
                            transition: 'transform 420ms cubic-bezier(0.22, 1, 0.36, 1)',
                          }}
                        >
                          {activeSlide.title}
                        </h2>
                      </div>

                      {/* SVG A* Connector overlay */}
                      {connectorsVisible && (
                        <svg
                          style={{
                            position: 'absolute',
                            top: 0,
                            left: 0,
                            width: '100%',
                            height: '100%',
                            pointerEvents: 'none',
                            zIndex: 1,
                          }}
                        >
                          <line
                            x1="28%"
                            y1="75%"
                            x2="48%"
                            y2="75%"
                            stroke="var(--mercury-periwinkle)"
                            strokeWidth="2"
                            strokeDasharray="4 4"
                          />
                          <line
                            x1="62%"
                            y1="75%"
                            x2="80%"
                            y2="75%"
                            stroke="var(--mercury-periwinkle)"
                            strokeWidth="2"
                            strokeDasharray="4 4"
                          />
                        </svg>
                      )}

                      {/* DrawingML Vector Pillar Nodes */}
                      <div
                        style={{
                          display: 'grid',
                          gridTemplateColumns: `repeat(${activeSlide.pillars.length}, 1fr)`,
                          gap: 'var(--spacing-lg)',
                          marginTop: 'var(--spacing-xl)',
                          position: 'relative',
                          zIndex: 2,
                        }}
                      >
                        {activeSlide.pillars.map((pillar, pIdx) => (
                          <div
                            key={pIdx}
                            style={{
                              backgroundColor: 'var(--mercury-night)',
                              border: '1px solid var(--mercury-glass-dark-border)',
                              borderRadius: 'var(--rounded-md)',
                              padding: 'var(--spacing-lg)',
                              transform: morphPreviewActive
                                ? `translateY(${(pIdx % 2 === 0 ? -1 : 1) * 12}px)`
                                : 'none',
                              transition: 'transform 420ms ease',
                            }}
                          >
                            <div className="ty-eyebrow" style={{ color: 'var(--mercury-periwinkle)' }}>
                              NODE !!STEP_{pIdx + 1}
                            </div>
                            <div className="ty-lead" style={{ marginTop: 'var(--spacing-xs)', fontSize: 18 }}>
                              {pillar}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>

                    {/* Speaker Notes Drawer */}
                    <div className="mercury-speaker-notes-drawer">
                      <div
                        className="ty-eyebrow"
                        style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 6 }}
                      >
                        SPEAKER NOTES (SLIDE #{activeSlideIndex + 1})
                      </div>
                      <textarea
                        value={activeSlide.speakerNotes}
                        onChange={(e) => handleUpdateSpeakerNotes(e.target.value)}
                        className="ty-body"
                        style={{
                          width: '100%',
                          minHeight: 64,
                          border: 'none',
                          outline: 'none',
                          resize: 'vertical',
                          backgroundColor: 'transparent',
                          color: 'var(--mercury-ink)',
                          fontFamily: 'var(--font-arcadia-sans)',
                        }}
                        placeholder="Type presenter notes for this slide..."
                      />
                    </div>
                  </div>
                </div>
              )}

              {/* ─── MODULE 4: WPS PDF TOOLKIT & BBOX RAG CANVAS ─── */}
              {activeTab.module === 'pdf' && (
                <div className="mercury-pdf-page">
                  <div
                    className="ty-eyebrow"
                    style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-lg)' }}
                  >
                    ISO 32000-2 CERTIFIED • PAGE 1 OF 1 • 72 DPI SPATIAL MESH • AES-256 R6
                  </div>

                  {pdfBlocks.map((block) => {
                    const isCited = highlightedPdfBlockId === block.blockId;
                    return (
                      <div
                        key={block.blockId}
                        onClick={() => setHighlightedPdfBlockId(block.blockId)}
                        className={`mercury-pdf-block ${isCited ? 'is-cited' : ''} ${
                          pdfInlineEditMode ? 'is-editable' : ''
                        }`}
                      >
                        <div
                          className="ty-eyebrow"
                          style={{
                            color: isCited ? 'var(--mercury-primary)' : 'var(--mercury-ink-secondary)',
                            marginBottom: 6,
                            display: 'flex',
                            justifyContent: 'space-between',
                          }}
                        >
                          <span>
                            {block.role} • BLOCK #{block.blockId}
                          </span>
                          <span>BBOX [{block.bbox.join(', ')}]</span>
                        </div>

                        <div
                          contentEditable={pdfInlineEditMode}
                          suppressContentEditableWarning
                          onBlur={(e) => handleUpdatePdfBlockText(block.blockId, e.currentTarget.innerText)}
                          className={block.role === 'Heading' ? 'ty-heading-2' : 'ty-body-lg'}
                          style={{ outline: 'none' }}
                        >
                          {block.redacted ? (
                            <span>
                              {block.text.split('[REDACTED-SSN - AES-256 PURGED]')[0]}
                              <span className="mercury-redaction-mask">[REDACTED-SSN - AES-256 PURGED]</span>
                              {block.text.split('[REDACTED-SSN - AES-256 PURGED]')[1]}
                            </span>
                          ) : (
                            block.text
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </main>

            {/* ─── 3. RIGHT DOCKABLE INSPECTOR STUDIO ─── */}
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
                  flexShrink: 0,
                }}
              >
                {/* 3-Segment Pill Switcher */}
                <div className="mercury-segmented-control">
                  <button
                    type="button"
                    onClick={() => setInspectorTab('ai-copilot')}
                    className={`mercury-segmented-btn ${inspectorTab === 'ai-copilot' ? 'is-active' : ''}`}
                  >
                    ✦ @WPS-AI
                  </button>
                  <button
                    type="button"
                    onClick={() => setInspectorTab('python-studio')}
                    className={`mercury-segmented-btn ${inspectorTab === 'python-studio' ? 'is-active' : ''}`}
                  >
                    🐍 Python (=PY)
                  </button>
                  <button
                    type="button"
                    onClick={() => setInspectorTab('roaming-history')}
                    className={`mercury-segmented-btn ${inspectorTab === 'roaming-history' ? 'is-active' : ''}`}
                  >
                    🕒 Roaming
                  </button>
                </div>

                {/* TAB 1: @WPS-AI COPILOT */}
                {inspectorTab === 'ai-copilot' && (
                  <div className="mercury-feature-card-light" style={{ padding: '20px' }}>
                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-xs)' }}
                    >
                      AST-GROUNDED CO-PILOT • @{activeTab.module.toUpperCase()}AI
                    </div>
                    <h3 className="ty-heading-2" style={{ margin: '0 0 var(--spacing-sm) 0', fontSize: 22 }}>
                      Contextual Intelligence
                    </h3>
                    <p
                      className="ty-body"
                      style={{ color: 'var(--mercury-ink-secondary)', margin: '0 0 var(--spacing-md) 0' }}
                    >
                      Every suggestion mutates the underlying Rust AST or cites exact PDF page coordinates.
                    </p>

                    {/* Toggle Switch for Track Changes Mode */}
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'space-between',
                        padding: '10px 0',
                        borderBottom: '1px solid var(--mercury-fog)',
                        marginBottom: 'var(--spacing-sm)',
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

                    {/* Quick Suggestion Actions */}
                    <div style={{ marginBottom: 'var(--spacing-lg)' }}>
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

                    {/* Chat Messages Log */}
                    <div
                      style={{
                        maxHeight: 180,
                        overflowY: 'auto',
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 8,
                        marginBottom: 12,
                      }}
                    >
                      {aiChatMessages.map((msg) => (
                        <div
                          key={msg.id}
                          style={{
                            padding: '8px 12px',
                            borderRadius: 'var(--rounded-sm)',
                            backgroundColor:
                              msg.sender === 'user' ? 'rgba(82, 102, 235, 0.12)' : '#ffffff',
                            fontSize: 13,
                            color: 'var(--mercury-ink)',
                          }}
                        >
                          <div
                            className="ty-eyebrow"
                            style={{ fontSize: 10, color: 'var(--mercury-slate)', marginBottom: 2 }}
                          >
                            {msg.sender === 'user' ? 'YOU' : '@WPS-AI'} • {msg.timestamp}
                          </div>
                          <div>{msg.content}</div>
                        </div>
                      ))}
                    </div>

                    {/* Chat Prompt Input */}
                    <form
                      onSubmit={(e) => {
                        e.preventDefault();
                        handleSendAiChat();
                      }}
                      className="mercury-fused-capsule"
                      style={{ height: 38 }}
                    >
                      <input
                        type="text"
                        className="mercury-text-input ty-ui-sm"
                        placeholder="Ask @WPS-AI a question..."
                        value={aiChatInput}
                        onChange={(e) => setAiChatInput(e.target.value)}
                      />
                      <button
                        type="submit"
                        className="mercury-button-tint-light ty-ui-sm"
                        style={{ height: 32, marginRight: 2 }}
                      >
                        Ask
                      </button>
                    </form>
                  </div>
                )}

                {/* TAB 2: PYTHON (=PY) STUDIO */}
                {inspectorTab === 'python-studio' && (
                  <div className="mercury-feature-card-light" style={{ padding: '20px' }}>
                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-xs)' }}
                    >
                      PYO3 (DESKTOP) + PYODIDE (WASM)
                    </div>
                    <h3 className="ty-heading-2" style={{ margin: '0 0 var(--spacing-sm) 0', fontSize: 22 }}>
                      =PY() DataFrame Studio
                    </h3>

                    <textarea
                      value={pythonCode}
                      onChange={(e) => setPythonCode(e.target.value)}
                      style={{
                        width: '100%',
                        height: 96,
                        backgroundColor: 'var(--mercury-surface)',
                        padding: 'var(--spacing-sm)',
                        borderRadius: 'var(--rounded-xs)',
                        fontFamily: 'var(--font-mono)',
                        fontSize: 12,
                        border: '1px solid var(--mercury-fog)',
                        color: 'var(--mercury-ink)',
                        outline: 'none',
                        resize: 'vertical',
                        boxSizing: 'border-box',
                      }}
                    />

                    <div style={{ marginTop: 8, marginBottom: 12 }}>
                      <button
                        type="button"
                        onClick={handleRunPythonScript}
                        className="mercury-button-primary ty-ui-sm"
                        style={{ width: '100%', height: 34 }}
                      >
                        ▶ Run Script (=PY)
                      </button>
                    </div>

                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 6 }}
                    >
                      DATAFRAME SPILL OUTPUT
                    </div>
                    {pythonSpillData.map((row, idx) => (
                      <div key={idx} className="mercury-accordion-row ty-ui-sm" style={{ padding: '6px 0' }}>
                        <span>{row.region} Total</span>
                        <span style={{ fontWeight: 500 }}>{row.total}</span>
                      </div>
                    ))}

                    <div
                      style={{
                        marginTop: 12,
                        padding: 8,
                        backgroundColor: '#ffffff',
                        borderRadius: 'var(--rounded-xs)',
                        border: '1px solid var(--mercury-fog)',
                        fontFamily: 'var(--font-mono)',
                        fontSize: 11,
                        color: 'var(--mercury-slate)',
                        whiteSpace: 'pre-wrap',
                      }}
                    >
                      {pythonConsoleLog}
                    </div>
                  </div>
                )}

                {/* TAB 3: FILE ROAMING TIME-MACHINE */}
                {inspectorTab === 'roaming-history' && (
                  <div className="mercury-feature-card-light" style={{ padding: '20px' }}>
                    <div
                      className="ty-eyebrow"
                      style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 'var(--spacing-xs)' }}
                    >
                      YRS CRDT & SHA-256 CONTENT STORE
                    </div>
                    <h3 className="ty-heading-2" style={{ margin: '0 0 var(--spacing-md) 0', fontSize: 22 }}>
                      File Roaming Snapshots
                    </h3>

                    <div className="mercury-timeline">
                      <div className="mercury-timeline-stem" />
                      {roamingSnapshots.map((snap, idx) => (
                        <div key={snap.versionId} className="mercury-timeline-node">
                          <span
                            className={`mercury-timeline-dot ${idx > 0 ? 'is-past' : ''}`}
                          />
                          <div style={{ flex: 1 }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                              <span className="ty-ui-sm" style={{ fontWeight: idx === 0 ? 500 : 400 }}>
                                {snap.label}
                              </span>
                              {idx > 0 && (
                                <button
                                  type="button"
                                  onClick={() => handleRestoreSnapshot(snap.versionId)}
                                  className="mercury-button-small ty-ui-sm"
                                  style={{ height: 22, padding: '0 8px', fontSize: 11 }}
                                >
                                  Rollback
                                </button>
                              )}
                            </div>
                            <div className="ty-legal" style={{ color: 'var(--mercury-ink-secondary)', marginTop: 2 }}>
                              v{snap.versionId} • {snap.timestamp} • {snap.author} • #{snap.sha256Short}
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

      {/* ─── 4. MERCURY STATUS FOOTER ─── */}
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

      {/* ─── PRESENTER VIEW COCKPIT MODAL ─── */}
      {presenterModalOpen && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            backgroundColor: 'rgba(23, 23, 33, 0.85)',
            backdropFilter: 'blur(8px)',
            zIndex: 200,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 32,
          }}
        >
          <div
            style={{
              backgroundColor: 'var(--mercury-night-raised)',
              border: '1px solid var(--mercury-glass-dark-border)',
              borderRadius: 'var(--rounded-lg)',
              maxWidth: 960,
              width: '100%',
              padding: 32,
              color: 'var(--mercury-on-dark)',
              boxShadow: '0 24px 64px rgba(0, 0, 0, 0.6)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 24 }}>
              <div>
                <div className="ty-eyebrow" style={{ color: 'var(--mercury-periwinkle)' }}>
                  DUAL-SCREEN PRESENTER VIEW COCKPIT
                </div>
                <h2 className="ty-heading-2" style={{ margin: '4px 0 0 0' }}>
                  Slide {activeSlideIndex + 1} of {slides.length}: {activeSlide.title}
                </h2>
              </div>
              <button
                type="button"
                onClick={() => setPresenterModalOpen(false)}
                className="mercury-button-glass-dark ty-ui-sm"
              >
                Exit Presenter View
              </button>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: 24 }}>
              {/* Main Audience Slide Preview */}
              <div
                style={{
                  aspectRatio: '16/9',
                  backgroundColor: 'var(--mercury-night)',
                  border: '1px solid var(--mercury-glass-dark-border)',
                  borderRadius: 'var(--rounded-md)',
                  padding: 24,
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'space-between',
                }}
              >
                <div className="ty-eyebrow" style={{ color: 'var(--mercury-periwinkle)' }}>
                  AUDIENCE SCREEN (120 FPS NATIVE)
                </div>
                <div className="ty-lead" style={{ fontSize: 24 }}>
                  {activeSlide.title}
                </div>
                <div style={{ display: 'flex', gap: 8 }}>
                  {activeSlide.pillars.map((p, i) => (
                    <div
                      key={i}
                      style={{
                        backgroundColor: 'var(--mercury-night-raised)',
                        padding: '6px 12px',
                        borderRadius: 6,
                        fontSize: 12,
                      }}
                    >
                      {p}
                    </div>
                  ))}
                </div>
              </div>

              {/* Presenter Notes & Telemetry */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
                <div
                  style={{
                    backgroundColor: 'var(--mercury-night)',
                    border: '1px solid var(--mercury-glass-dark-border)',
                    borderRadius: 'var(--rounded-md)',
                    padding: 16,
                  }}
                >
                  <div className="ty-eyebrow" style={{ color: 'var(--mercury-fog)', marginBottom: 6 }}>
                    ELAPSED TIME
                  </div>
                  <div className="ty-heading-2" style={{ fontFamily: 'var(--font-mono)' }}>
                    00:04:18
                  </div>
                </div>

                <div
                  style={{
                    backgroundColor: 'var(--mercury-night)',
                    border: '1px solid var(--mercury-glass-dark-border)',
                    borderRadius: 'var(--rounded-md)',
                    padding: 16,
                    flex: 1,
                  }}
                >
                  <div className="ty-eyebrow" style={{ color: 'var(--mercury-fog)', marginBottom: 6 }}>
                    SPEAKER CUES & NOTES
                  </div>
                  <div className="ty-body" style={{ color: 'var(--mercury-on-dark)', fontSize: 14 }}>
                    {activeSlide.speakerNotes}
                  </div>
                </div>
              </div>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 24 }}>
              <button
                type="button"
                disabled={activeSlideIndex <= 0}
                onClick={() => setActiveSlideIndex((i) => Math.max(0, i - 1))}
                className="mercury-button-glass-dark ty-ui"
                style={{ opacity: activeSlideIndex <= 0 ? 0.4 : 1 }}
              >
                ◀ Previous Slide
              </button>
              <button
                type="button"
                disabled={activeSlideIndex >= slides.length - 1}
                onClick={() => setActiveSlideIndex((i) => Math.min(slides.length - 1, i + 1))}
                className="mercury-button-primary ty-ui"
                style={{ opacity: activeSlideIndex >= slides.length - 1 ? 0.4 : 1 }}
              >
                Next Slide ▶
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ─── EXPORT & SHARE MODAL ─── */}
      {exportModalOpen && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            backgroundColor: 'rgba(23, 23, 33, 0.85)',
            backdropFilter: 'blur(8px)',
            zIndex: 200,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 32,
          }}
        >
          <div
            style={{
              backgroundColor: 'var(--mercury-surface)',
              borderRadius: 'var(--rounded-lg)',
              maxWidth: 540,
              width: '100%',
              padding: 32,
              color: 'var(--mercury-ink)',
              boxShadow: '0 24px 64px rgba(0, 0, 0, 0.3)',
            }}
          >
            <div className="ty-eyebrow" style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 6 }}>
              UNIFIED EXPORT SUITE
            </div>
            <h2 className="ty-heading-2" style={{ margin: '0 0 16px 0' }}>
              Export {activeTab.title}
            </h2>
            <p className="ty-body" style={{ color: 'var(--mercury-ink-secondary)', marginBottom: 24 }}>
              Compile the underlying Rust AST into standard ECMA-376 OOXML or ISO 32000-2 encrypted PDF binaries.
            </p>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 12, marginBottom: 24 }}>
              {[
                { title: 'Native Office (.docx / .xlsx / .pptx)', desc: 'Zero-loss ECMA-376 with full styles and DAG formulas' },
                { title: 'ISO 32000-2 Certified PDF', desc: 'Vector glyph paths, PDF/UA accessibility, and AES-256' },
                { title: 'CRDT Yrs Archive (.yrs)', desc: 'Full snapshot time-machine ledger with SHA-256 commit chain' },
              ].map((opt, i) => (
                <div
                  key={i}
                  onClick={() => {
                    setExportModalOpen(false);
                    showToast(`Exported: ${activeTab.title} as ${opt.title}`);
                  }}
                  style={{
                    padding: '14px 18px',
                    borderRadius: 'var(--rounded-sm)',
                    border: '1px solid var(--mercury-fog)',
                    cursor: 'pointer',
                    transition: 'all 0.15s ease',
                    backgroundColor: i === 0 ? 'var(--mercury-surface-tint)' : '#ffffff',
                  }}
                >
                  <div className="ty-ui-sm" style={{ fontWeight: 500, color: 'var(--mercury-ink)' }}>
                    {opt.title}
                  </div>
                  <div className="ty-legal" style={{ color: 'var(--mercury-ink-secondary)', marginTop: 2 }}>
                    {opt.desc}
                  </div>
                </div>
              ))}
            </div>

            <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
              <button
                type="button"
                onClick={() => setExportModalOpen(false)}
                className="mercury-button-tint-light ty-ui-sm"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={() => {
                  setExportModalOpen(false);
                  showToast(`Exported: ${activeTab.title}`);
                }}
                className="mercury-button-primary ty-ui-sm"
              >
                Confirm Export
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default MercuryWorkspaceShell;
