import React, { useState, useRef } from 'react';
import {
  Undo2,
  Redo2,
  Bold,
  Italic,
  Underline,
  Strikethrough,
  AlignLeft,
  AlignCenter,
  AlignRight,
  AlignJustify,
  List,
  ListOrdered,
  Indent,
  Outdent,
  Table as TableIcon,
  Image as ImageIcon,
  BookOpen,
  FileText,
  Mail,
  CheckCircle,
  XCircle,
  MessageSquare,
  Sparkles,
  ChevronDown,
  Plus,
  Trash2
} from 'lucide-react';
import { WriterParagraph, WriterTable, WriterComment, WriterDocumentState } from '../types';

interface WriterWorkspaceProps {
  initialState?: Partial<WriterDocumentState>;
  onStateChange?: (state: WriterDocumentState) => void;
  onOpenAiAssist?: (prompt: string) => void;
}

export const WriterWorkspace: React.FC<WriterWorkspaceProps> = ({
  initialState,
  onStateChange,
  onOpenAiAssist,
}) => {
  // ─── Document State ───
  const [activeRibbonTab, setActiveRibbonTab] = useState<'home' | 'insert' | 'layout' | 'references' | 'review'>('home');
  const [paragraphs, setParagraphs] = useState<WriterParagraph[]>(
    initialState?.paragraphs || [
      {
        id: 'p1',
        style: 'Title',
        text: 'WPS Office Enterprise Proposal',
        align: 'left',
      },
      {
        id: 'p2',
        style: 'Heading 1',
        text: '1. Executive Summary',
        align: 'left',
      },
      {
        id: 'p3',
        style: 'Normal',
        text: 'This document presents the complete office suite architecture delivering high fidelity document processing, rich spreadsheets, professional presentations, and integrated PDF conversions.',
        align: 'left',
        revision: {
          type: 'insert',
          originalText: '',
          revisedText: ' delivering high fidelity document processing, rich spreadsheets, professional presentations, and integrated PDF conversions.',
          author: 'Alex Morgan',
          date: 'Today, 10:15 AM',
        },
      },
      {
        id: 'p4',
        style: 'Heading 2',
        text: '2. Productivity Highlights',
        align: 'left',
      },
      {
        id: 'p5',
        style: 'Normal',
        text: 'Users can edit rich formatted text, manage tracked revisions, insert dynamic calculation tables, and collaborate seamlessly across devices.',
        align: 'left',
      },
      {
        id: 'p6',
        style: 'Quote',
        text: '“Simplicity is about subtracting the obvious and adding the meaningful.” — John Maeda',
        align: 'left',
      },
    ]
  );

  const [tables, setTables] = useState<WriterTable[]>(
    initialState?.tables || [
      {
        id: 'table-1',
        rows: 3,
        cols: 3,
        data: [
          ['Feature', 'Support Level', 'Notes'],
          ['Track Changes', 'Complete', 'Redlines & Revisions'],
          ['Table of Contents', 'Automated', 'Heading-based scan'],
        ],
      },
    ]
  );

  const [comments, setComments] = useState<WriterComment[]>(
    initialState?.comments || [
      {
        id: 'c1',
        author: 'Sarah Chen',
        date: 'Today, 11:20 AM',
        text: 'Please verify the implementation timeline with engineering.',
        targetText: 'Productivity Highlights',
        resolved: false,
      },
    ]
  );

  const [trackChanges, setTrackChanges] = useState<boolean>(initialState?.trackChanges ?? true);
  const [markupView, setMarkupView] = useState<'all' | 'final'>(initialState?.markupView || 'all');
  const [pageSize, setPageSize] = useState<'Letter' | 'A4'>(initialState?.pageSize || 'Letter');
  const [orientation, setOrientation] = useState<'Portrait' | 'Landscape'>(initialState?.orientation || 'Portrait');
  const [margins, setMargins] = useState<'Normal' | 'Narrow' | 'Wide'>(initialState?.margins || 'Normal');
  const [zoom, setZoom] = useState<number>(initialState?.zoom || 100);

  // Formatting Selection State
  const [selectedParaId, setSelectedParaId] = useState<string>('p1');
  const [fontFamily, setFontFamily] = useState<string>('Inter');
  const [fontSize, setFontSize] = useState<number>(14);
  const [isBold, setIsBold] = useState<boolean>(false);
  const [isItalic, setIsItalic] = useState<boolean>(false);
  const [isUnderline, setIsUnderline] = useState<boolean>(false);
  const [isStrike, setIsStrike] = useState<boolean>(false);
  const [textColor, setTextColor] = useState<string>('#28253b');
  const [highlightColor, setHighlightColor] = useState<string>('transparent');

  // Table Insert Picker Popover
  const [showTablePicker, setShowTablePicker] = useState(false);
  const [hoverGrid, setHoverGrid] = useState({ rows: 3, cols: 3 });

  // Word & Character Counts
  const totalWords = paragraphs.reduce((acc, p) => acc + (p.text.trim() ? p.text.trim().split(/\s+/).length : 0), 0);
  const totalChars = paragraphs.reduce((acc, p) => acc + p.text.length, 0);

  // Update paragraph text
  const handleParagraphChange = (id: string, newText: string) => {
    setParagraphs((prev) =>
      prev.map((p) => {
        if (p.id !== id) return p;
        if (trackChanges && !p.revision) {
          return {
            ...p,
            text: newText,
            revision: {
              type: 'insert',
              originalText: p.text,
              revisedText: newText,
              author: 'Current User',
              date: 'Just now',
            },
          };
        }
        return { ...p, text: newText };
      })
    );
  };

  // Apply style to active paragraph
  const handleApplyStyle = (style: WriterParagraph['style']) => {
    setParagraphs((prev) =>
      prev.map((p) => (p.id === selectedParaId ? { ...p, style } : p))
    );
  };

  // Apply alignment to active paragraph
  const handleApplyAlign = (align: WriterParagraph['align']) => {
    setParagraphs((prev) =>
      prev.map((p) => (p.id === selectedParaId ? { ...p, align } : p))
    );
  };

  // Insert Table
  const handleInsertTable = (rows: number, cols: number) => {
    const data: string[][] = [];
    for (let r = 0; r < rows; r++) {
      const row: string[] = [];
      for (let c = 0; c < cols; c++) {
        row.push(r === 0 ? `Header ${c + 1}` : `Data ${r},${c + 1}`);
      }
      data.push(row);
    }
    const newTable: WriterTable = {
      id: `table-${Date.now()}`,
      rows,
      cols,
      data,
    };
    setTables((prev) => [...prev, newTable]);
    setShowTablePicker(false);
  };

  // Table Cell Editing
  const handleTableCellChange = (tableId: string, r: number, c: number, val: string) => {
    setTables((prev) =>
      prev.map((t) => {
        if (t.id !== tableId) return t;
        const newData = t.data.map((rowArr, rowIdx) =>
          rowIdx === r ? rowArr.map((cellVal, colIdx) => (colIdx === c ? val : cellVal)) : rowArr
        );
        return { ...t, data: newData };
      })
    );
  };

  // Revisions Management
  const handleAcceptRevision = (id: string) => {
    setParagraphs((prev) =>
      prev.map((p) => (p.id === id ? { ...p, revision: undefined } : p))
    );
  };

  const handleRejectRevision = (id: string) => {
    setParagraphs((prev) =>
      prev.map((p) => {
        if (p.id !== id || !p.revision) return p;
        return {
          ...p,
          text: p.revision.originalText || '',
          revision: undefined,
        };
      })
    );
  };

  const handleAcceptAllRevisions = () => {
    setParagraphs((prev) => prev.map((p) => ({ ...p, revision: undefined })));
  };

  const handleRejectAllRevisions = () => {
    setParagraphs((prev) =>
      prev.map((p) => ({
        ...p,
        text: p.revision ? p.revision.originalText : p.text,
        revision: undefined,
      }))
    );
  };

  // Insert Table of Contents
  const handleInsertToc = () => {
    const headings = paragraphs.filter((p) => p.style.startsWith('Heading'));
    const tocText = headings.map((h, idx) => `${idx + 1}. ${h.text}`).join('\n');
    const newPara: WriterParagraph = {
      id: `toc-${Date.now()}`,
      style: 'Normal',
      text: `TABLE OF CONTENTS:\n${tocText}`,
      align: 'left',
    };
    setParagraphs((prev) => [prev[0], newPara, ...prev.slice(1)]);
  };

  // Add Comment
  const handleAddComment = () => {
    const newComment: WriterComment = {
      id: `c-${Date.now()}`,
      author: 'Current User',
      date: 'Just now',
      text: 'Please review this paragraph.',
      targetText: paragraphs.find((p) => p.id === selectedParaId)?.text.slice(0, 30) || 'Document Section',
      resolved: false,
    };
    setComments((prev) => [...prev, newComment]);
  };

  return (
    <div className="flex-1 flex flex-col h-full overflow-hidden bg-mercury-canvas">
      {/* ─── Zone 2: Contextual Ribbon Toolbar ─── */}
      <div className="wps-ribbon-bar">
        {/* Ribbon Tab Strip */}
        <div className="wps-ribbon-tabs-row">
          <div className="flex items-center gap-1">
            <button
              onClick={() => setActiveRibbonTab('home')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'home' ? 'is-active' : ''}`}
            >
              Home
            </button>
            <button
              onClick={() => setActiveRibbonTab('insert')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'insert' ? 'is-active' : ''}`}
            >
              Insert
            </button>
            <button
              onClick={() => setActiveRibbonTab('layout')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'layout' ? 'is-active' : ''}`}
            >
              Page Layout
            </button>
            <button
              onClick={() => setActiveRibbonTab('references')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'references' ? 'is-active' : ''}`}
            >
              References & Mail Merge
            </button>
            <button
              onClick={() => setActiveRibbonTab('review')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'review' ? 'is-active' : ''}`}
            >
              Review & Track Changes
            </button>
          </div>

          <div className="flex items-center gap-2 text-xs text-mercury-slate">
            <span>WPS Writer</span>
          </div>
        </div>

        {/* Ribbon Body Actions */}
        <div className="wps-ribbon-body">
          {/* TAB 1: HOME */}
          {activeRibbonTab === 'home' && (
            <>
              {/* Group: Clipboard */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button className="wps-ribbon-btn-compact" title="Undo (Ctrl+Z)">
                    <Undo2 className="w-4 h-4" />
                  </button>
                  <button className="wps-ribbon-btn-compact" title="Redo (Ctrl+Y)">
                    <Redo2 className="w-4 h-4" />
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Clipboard</span>
              </div>

              {/* Group: Font Family & Size */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <select
                    value={fontFamily}
                    onChange={(e) => setFontFamily(e.target.value)}
                    className="h-7 text-xs bg-white border border-mercury-fog/50 rounded px-2 outline-none"
                  >
                    <option value="Inter">Inter</option>
                    <option value="Georgia">Georgia</option>
                    <option value="Times New Roman">Times New Roman</option>
                    <option value="JetBrains Mono">JetBrains Mono</option>
                  </select>

                  <select
                    value={fontSize}
                    onChange={(e) => setFontSize(Number(e.target.value))}
                    className="h-7 text-xs bg-white border border-mercury-fog/50 rounded px-2 outline-none w-14"
                  >
                    {[10, 11, 12, 14, 16, 18, 20, 24, 28, 32, 36, 48, 72].map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </select>

                  <button
                    onClick={() => setIsBold(!isBold)}
                    className={`wps-ribbon-btn-compact font-bold ${isBold ? 'is-active' : ''}`}
                    title="Bold"
                  >
                    <Bold className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => setIsItalic(!isItalic)}
                    className={`wps-ribbon-btn-compact italic ${isItalic ? 'is-active' : ''}`}
                    title="Italic"
                  >
                    <Italic className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => setIsUnderline(!isUnderline)}
                    className={`wps-ribbon-btn-compact underline ${isUnderline ? 'is-active' : ''}`}
                    title="Underline"
                  >
                    <Underline className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => setIsStrike(!isStrike)}
                    className={`wps-ribbon-btn-compact line-through ${isStrike ? 'is-active' : ''}`}
                    title="Strikethrough"
                  >
                    <Strikethrough className="w-3.5 h-3.5" />
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Font</span>
              </div>

              {/* Group: Paragraph Alignment & Lists */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => handleApplyAlign('left')}
                    className="wps-ribbon-btn-compact"
                    title="Align Left"
                  >
                    <AlignLeft className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => handleApplyAlign('center')}
                    className="wps-ribbon-btn-compact"
                    title="Center"
                  >
                    <AlignCenter className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => handleApplyAlign('right')}
                    className="wps-ribbon-btn-compact"
                    title="Align Right"
                  >
                    <AlignRight className="w-3.5 h-3.5" />
                  </button>
                  <button
                    onClick={() => handleApplyAlign('justify')}
                    className="wps-ribbon-btn-compact"
                    title="Justify"
                  >
                    <AlignJustify className="w-3.5 h-3.5" />
                  </button>
                  <button className="wps-ribbon-btn-compact" title="Bullet List">
                    <List className="w-3.5 h-3.5" />
                  </button>
                  <button className="wps-ribbon-btn-compact" title="Numbered List">
                    <ListOrdered className="w-3.5 h-3.5" />
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Paragraph</span>
              </div>

              {/* Group: Styles Gallery */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => handleApplyStyle('Normal')}
                    className="px-2.5 py-1 text-xs border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Normal
                  </button>
                  <button
                    onClick={() => handleApplyStyle('Title')}
                    className="px-2.5 py-1 text-xs font-bold border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Title
                  </button>
                  <button
                    onClick={() => handleApplyStyle('Heading 1')}
                    className="px-2.5 py-1 text-xs font-semibold text-blue-700 border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Heading 1
                  </button>
                  <button
                    onClick={() => handleApplyStyle('Heading 2')}
                    className="px-2.5 py-1 text-xs font-medium text-blue-600 border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Heading 2
                  </button>
                  <button
                    onClick={() => handleApplyStyle('Quote')}
                    className="px-2.5 py-1 text-xs italic border border-mercury-fog/40 rounded bg-white hover:bg-mercury-lavender"
                  >
                    Quote
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Styles Gallery</span>
              </div>
            </>
          )}

          {/* TAB 2: INSERT */}
          {activeRibbonTab === 'insert' && (
            <>
              {/* Insert Table */}
              <div className="wps-ribbon-group relative">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setShowTablePicker(!showTablePicker)}
                    className="wps-ribbon-btn"
                  >
                    <TableIcon className="w-5 h-5 text-blue-600" />
                    <span>Insert Table</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Tables</span>

                {showTablePicker && (
                  <div className="absolute left-0 top-full mt-1 bg-white border border-mercury-fog rounded shadow-lg p-3 z-50">
                    <div className="text-xs font-semibold mb-2">
                      Insert Table: {hoverGrid.rows} × {hoverGrid.cols}
                    </div>
                    <div className="grid grid-cols-8 gap-1">
                      {Array.from({ length: 8 }).map((_, r) =>
                        Array.from({ length: 8 }).map((_, c) => {
                          const isHovered = r < hoverGrid.rows && c < hoverGrid.cols;
                          return (
                            <div
                              key={`${r}-${c}`}
                              onMouseEnter={() => setHoverGrid({ rows: r + 1, cols: c + 1 })}
                              onClick={() => handleInsertTable(hoverGrid.rows, hoverGrid.cols)}
                              className={`w-4 h-4 border cursor-pointer ${
                                isHovered ? 'bg-mercury-primary border-mercury-primary' : 'bg-gray-100 border-gray-300'
                              }`}
                            />
                          );
                        })
                      )}
                    </div>
                  </div>
                )}
              </div>

              {/* Insert Image */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <label className="wps-ribbon-btn cursor-pointer">
                    <ImageIcon className="w-5 h-5 text-emerald-600" />
                    <span>Insert Image</span>
                    <input type="file" accept="image/*" className="hidden" />
                  </label>
                </div>
                <span className="wps-ribbon-group-title">Illustrations</span>
              </div>

              {/* Comments */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button onClick={handleAddComment} className="wps-ribbon-btn">
                    <MessageSquare className="w-5 h-5 text-amber-600" />
                    <span>Add Comment</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Collaboration</span>
              </div>
            </>
          )}

          {/* TAB 3: PAGE LAYOUT */}
          {activeRibbonTab === 'layout' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <div className="flex flex-col gap-1 text-xs">
                    <label className="text-[10px] text-mercury-slate">Paper Size</label>
                    <select
                      value={pageSize}
                      onChange={(e) => setPageSize(e.target.value as any)}
                      className="h-7 text-xs bg-white border border-mercury-fog/50 rounded px-2"
                    >
                      <option value="Letter">Letter (8.5" × 11")</option>
                      <option value="A4">A4 (210 × 297 mm)</option>
                    </select>
                  </div>

                  <div className="flex flex-col gap-1 text-xs">
                    <label className="text-[10px] text-mercury-slate">Orientation</label>
                    <select
                      value={orientation}
                      onChange={(e) => setOrientation(e.target.value as any)}
                      className="h-7 text-xs bg-white border border-mercury-fog/50 rounded px-2"
                    >
                      <option value="Portrait">Portrait</option>
                      <option value="Landscape">Landscape</option>
                    </select>
                  </div>

                  <div className="flex flex-col gap-1 text-xs">
                    <label className="text-[10px] text-mercury-slate">Margins</label>
                    <select
                      value={margins}
                      onChange={(e) => setMargins(e.target.value as any)}
                      className="h-7 text-xs bg-white border border-mercury-fog/50 rounded px-2"
                    >
                      <option value="Normal">Normal (1.0")</option>
                      <option value="Narrow">Narrow (0.5")</option>
                      <option value="Wide">Wide (1.5")</option>
                    </select>
                  </div>
                </div>
                <span className="wps-ribbon-group-title">Page Setup</span>
              </div>
            </>
          )}

          {/* TAB 4: REFERENCES & MAIL MERGE */}
          {activeRibbonTab === 'references' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button onClick={handleInsertToc} className="wps-ribbon-btn">
                    <BookOpen className="w-5 h-5 text-blue-600" />
                    <span>Table of Contents</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Table of Contents</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => alert('Mail Merge: Placeholders {{Name}}, {{Company}}, and {{Amount}} are ready to bind.')}
                    className="wps-ribbon-btn"
                  >
                    <Mail className="w-5 h-5 text-indigo-600" />
                    <span>Mail Merge (CSV)</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Mailings</span>
              </div>
            </>
          )}

          {/* TAB 5: REVIEW & TRACK CHANGES */}
          {activeRibbonTab === 'review' && (
            <>
              {/* Track Changes Toggle */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setTrackChanges(!trackChanges)}
                    className={`wps-ribbon-btn ${trackChanges ? 'is-active' : ''}`}
                  >
                    <FileText className="w-5 h-5 text-amber-600" />
                    <span>Track Changes: {trackChanges ? 'ON' : 'OFF'}</span>
                  </button>

                  <select
                    value={markupView}
                    onChange={(e) => setMarkupView(e.target.value as any)}
                    className="h-7 text-xs bg-white border border-mercury-fog/50 rounded px-2"
                  >
                    <option value="all">All Markup</option>
                    <option value="final">Final Text</option>
                  </select>
                </div>
                <span className="wps-ribbon-group-title">Tracking</span>
              </div>

              {/* Batch Actions */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={handleAcceptAllRevisions}
                    className="wps-ribbon-btn text-emerald-700"
                  >
                    <CheckCircle className="w-5 h-5 text-emerald-600" />
                    <span>Accept All</span>
                  </button>
                  <button
                    onClick={handleRejectAllRevisions}
                    className="wps-ribbon-btn text-rose-700"
                  >
                    <XCircle className="w-5 h-5 text-rose-600" />
                    <span>Reject All</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Changes</span>
              </div>
            </>
          )}
        </div>
      </div>

      {/* ─── Zone 3: Horizontal Ruler & Canvas Area ─── */}
      <div className="wps-canvas-container">
        <div className="wps-doc-viewport">
          {/* Top Ruler Bar */}
          <div
            className="wps-ruler-bar"
            style={{ width: orientation === 'Landscape' ? '1056px' : '816px' }}
          >
            <span>0"</span>
            <span>1"</span>
            <span>2"</span>
            <span>3"</span>
            <span>4"</span>
            <span>5"</span>
            <span>6"</span>
            <span>7"</span>
            {orientation === 'Landscape' && (
              <>
                <span>8"</span>
                <span>9"</span>
              </>
            )}
          </div>

          {/* Paper Sheet */}
          <div
            className="wps-paper-sheet"
            style={{
              width: orientation === 'Landscape' ? '1056px' : '816px',
              minHeight: orientation === 'Landscape' ? '816px' : '1056px',
              fontFamily,
              fontSize: `${fontSize}px`,
            }}
          >
            {paragraphs.map((p) => {
              const isTitle = p.style === 'Title';
              const isH1 = p.style === 'Heading 1';
              const isH2 = p.style === 'Heading 2';
              const isQuote = p.style === 'Quote';

              return (
                <div
                  key={p.id}
                  onClick={() => setSelectedParaId(p.id)}
                  className={`mb-4 relative group ${
                    selectedParaId === p.id ? 'ring-1 ring-mercury-primary/40 rounded p-1 -m-1' : ''
                  }`}
                >
                  <div
                    contentEditable
                    suppressContentEditableWarning
                    onBlur={(e) => handleParagraphChange(p.id, e.currentTarget.innerText)}
                    className={`outline-none leading-relaxed ${
                      isTitle ? 'text-3xl font-bold tracking-tight text-mercury-ink' : ''
                    } ${isH1 ? 'text-2xl font-semibold text-blue-800 mt-4 mb-2' : ''} ${
                      isH2 ? 'text-xl font-medium text-blue-700 mt-3 mb-1' : ''
                    } ${isQuote ? 'border-l-4 border-mercury-primary pl-4 italic text-mercury-slate' : ''} ${
                      isBold ? 'font-bold' : ''
                    } ${isItalic ? 'italic' : ''} ${isUnderline ? 'underline' : ''} ${
                      isStrike ? 'line-through' : ''
                    }`}
                    style={{ textAlign: p.align || 'left' }}
                  >
                    {p.text}
                  </div>

                  {/* Render Track Changes revision if present and markup view is 'all' */}
                  {markupView === 'all' && p.revision && (
                    <div className="mt-1 flex items-center gap-2 text-xs bg-amber-50 border border-amber-200 rounded px-2.5 py-1 text-amber-900">
                      <span className="font-semibold text-amber-800">
                        Tracked Revision ({p.revision.author}):
                      </span>
                      {p.revision.type === 'delete' ? (
                        <del className="text-rose-700 bg-rose-100 px-1 rounded">
                          {p.revision.originalText}
                        </del>
                      ) : (
                        <ins className="text-emerald-700 bg-emerald-100 px-1 rounded no-underline">
                          {p.revision.revisedText}
                        </ins>
                      )}
                      <div className="ml-auto flex items-center gap-1">
                        <button
                          onClick={() => handleAcceptRevision(p.id)}
                          className="px-2 py-0.5 rounded bg-emerald-600 text-white text-[11px] hover:bg-emerald-700"
                        >
                          Accept
                        </button>
                        <button
                          onClick={() => handleRejectRevision(p.id)}
                          className="px-2 py-0.5 rounded bg-rose-600 text-white text-[11px] hover:bg-rose-700"
                        >
                          Reject
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              );
            })}

            {/* Inserted Tables */}
            {tables.map((table) => (
              <div key={table.id} className="my-6 overflow-x-auto">
                <table className="w-full border-collapse border border-mercury-fog text-xs">
                  <tbody>
                    {table.data.map((row, rIdx) => (
                      <tr key={rIdx}>
                        {row.map((cell, cIdx) => (
                          <td
                            key={cIdx}
                            className={`border border-mercury-fog p-2.5 ${
                              rIdx === 0 ? 'bg-mercury-surface-tint font-semibold text-mercury-ink' : ''
                            }`}
                          >
                            <input
                              type="text"
                              value={cell}
                              onChange={(e) => handleTableCellChange(table.id, rIdx, cIdx, e.target.value)}
                              className="w-full bg-transparent outline-none border-none text-inherit text-xs"
                            />
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ))}
          </div>
        </div>

        {/* Right Margin Comments Drawer */}
        {comments.length > 0 && (
          <aside className="w-72 bg-white border-l border-mercury-fog p-3 flex flex-col gap-3 overflow-y-auto">
            <div className="flex items-center justify-between text-xs font-semibold text-mercury-slate">
              <span>Document Comments ({comments.length})</span>
            </div>

            {comments.map((c) => (
              <div
                key={c.id}
                className="p-3 rounded-mercury-md bg-mercury-surface-tint border border-mercury-fog/40 text-xs flex flex-col gap-1.5"
              >
                <div className="flex items-center justify-between">
                  <span className="font-semibold text-mercury-ink">{c.author}</span>
                  <span className="text-[10px] text-mercury-slate">{c.date}</span>
                </div>
                <div className="text-[11px] text-mercury-slate italic border-l-2 border-amber-400 pl-1.5">
                  "{c.targetText}"
                </div>
                <p className="text-mercury-ink text-xs">{c.text}</p>
                <button
                  onClick={() => setComments((prev) => prev.filter((item) => item.id !== c.id))}
                  className="self-end text-[10px] text-emerald-600 hover:underline mt-1 font-medium"
                >
                  ✓ Resolve
                </button>
              </div>
            ))}
          </aside>
        )}
      </div>

      {/* ─── Zone 5: Bottom Status Bar (28px) ─── */}
      <footer className="wps-status-bar">
        <div className="flex items-center gap-4">
          <span>Page 1 of 1</span>
          <span>{totalWords} words</span>
          <span>{totalChars} characters</span>
          <span className="flex items-center gap-1.5">
            <span
              className={`w-2 h-2 rounded-full ${
                trackChanges ? 'bg-amber-400 animate-pulse' : 'bg-gray-400'
              }`}
            />
            <span>Track Changes: {trackChanges ? 'ON' : 'OFF'}</span>
          </span>
        </div>

        <div className="flex items-center gap-3">
          <span className="text-[11px] text-mercury-fog">Zoom: {zoom}%</span>
          <input
            type="range"
            min="50"
            max="200"
            value={zoom}
            onChange={(e) => setZoom(Number(e.target.value))}
            className="w-24 h-1 accent-mercury-primary cursor-pointer"
          />
        </div>
      </footer>
    </div>
  );
};
