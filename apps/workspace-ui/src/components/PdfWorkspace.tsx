import React, { useState, useRef } from 'react';
import {
  FileText,
  Table,
  Highlighter,
  Underline,
  Strikethrough,
  MessageSquare,
  PenTool,
  FileCheck,
  ShieldAlert,
  RotateCw,
  Search,
  Hand,
  MousePointer,
  ZoomIn,
  ZoomOut,
  Maximize2,
  FileUp,
  Stamp,
  Check,
  X,
  Plus,
  Trash2
} from 'lucide-react';
import { PdfParagraphBlock, PdfStickyNote, PdfSignature, PdfDocumentState } from '../types';

interface PdfWorkspaceProps {
  initialState?: Partial<PdfDocumentState>;
  onStateChange?: (state: PdfDocumentState) => void;
  onConvertToWord?: () => void;
  onConvertToExcel?: () => void;
  onOpenAiAssist?: (prompt: string) => void;
}

export const PdfWorkspace: React.FC<PdfWorkspaceProps> = ({
  initialState,
  onStateChange,
  onConvertToWord,
  onConvertToExcel,
  onOpenAiAssist,
}) => {
  // ─── Document Blocks ───
  const [blocks, setBlocks] = useState<PdfParagraphBlock[]>(
    initialState?.blocks || [
      {
        id: 'block-1',
        page: 1,
        role: 'title',
        text: 'MASTER ENTERPRISE SERVICE AGREEMENT',
      },
      {
        id: 'block-2',
        page: 1,
        role: 'heading',
        text: '1. PARTIES & RECITALS',
      },
      {
        id: 'block-3',
        page: 1,
        role: 'paragraph',
        text: 'This Master Enterprise Agreement ("Agreement") is executed between OpenWPS Technologies, Inc. and Enterprise Client ("Licensee"). Authorized Contact: John Doe, SSN: 999-00-1234, Email: john.doe@enterprise.com.',
      },
      {
        id: 'block-4',
        page: 1,
        role: 'heading',
        text: '2. LICENSING TIERS & SERVICE PRICING',
      },
      {
        id: 'block-5',
        page: 1,
        role: 'table',
        text: 'Pricing & Licensing Schedule',
        tableData: [
          ['Tier Name', 'Monthly Base', 'Support Level', 'Seat Cap'],
          ['Tier 1 Core', '$4,500', 'Standard 8x5', '100 Seats'],
          ['Tier 2 Enterprise', '$12,500', 'Dedicated 24x7', 'Unlimited'],
        ],
      },
      {
        id: 'block-6',
        page: 1,
        role: 'paragraph',
        text: '3. TERMINATION & JURISDICTION: Either party may terminate this agreement upon 30 days written notice. Early buyout fee is $45,000 as agreed upon by all parties.',
      },
    ]
  );

  const [notes, setNotes] = useState<PdfStickyNote[]>(
    initialState?.notes || [
      {
        id: 'note-1',
        page: 1,
        x: 82,
        y: 22,
        text: 'Legal council reviewed Section 1. Verified SSN needs redaction before sharing externally.',
        author: 'Fiza Syed',
        date: 'Today, 2:15 PM',
      },
    ]
  );

  const [signatures, setSignatures] = useState<PdfSignature[]>(initialState?.signatures || []);
  const [activeRibbonTab, setActiveRibbonTab] = useState<'home' | 'annotate' | 'edit' | 'convert'>('home');
  const [activeTool, setActiveTool] = useState<'select' | 'pan' | 'highlight' | 'note' | 'redact' | 'sign'>('select');
  const [highlightColor, setHighlightColor] = useState<'yellow' | 'lavender' | 'periwinkle'>('yellow');
  const [isEditingText, setIsEditingText] = useState<boolean>(false);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [zoom, setZoom] = useState<number>(100);

  // Signature Modal State
  const [showSignatureModal, setShowSignatureModal] = useState<boolean>(false);
  const [signatureMode, setSignatureMode] = useState<'draw' | 'type'>('type');
  const [typedSignature, setTypedSignature] = useState<string>('John Doe');
  const sigCanvasRef = useRef<HTMLCanvasElement | null>(null);
  const [isDrawing, setIsDrawing] = useState(false);

  // Toggle Highlight on Block
  const handleToggleHighlight = (blockId: string) => {
    setBlocks((prev) =>
      prev.map((b) => {
        if (b.id !== blockId) return b;
        return {
          ...b,
          highlight: b.highlight === highlightColor ? null : highlightColor,
        };
      })
    );
  };

  // Redact Sensitive PII (SSN / Email)
  const handleRedactPii = () => {
    setBlocks((prev) =>
      prev.map((b) => {
        let updatedText = b.text;
        // SSN regex: \d{3}-\d{2}-\d{4}
        if (updatedText.includes('999-00-1234')) {
          updatedText = updatedText.replace('999-00-1234', '[REDACTED-SSN]');
          return { ...b, text: updatedText, redacted: true };
        }
        return b;
      })
    );
  };

  // Edit Paragraph Text
  const handleBlockTextChange = (id: string, newText: string) => {
    setBlocks((prev) =>
      prev.map((b) => (b.id === id ? { ...b, text: newText } : b))
    );
  };

  // Place Signature
  const handleApplySignature = () => {
    const newSig: PdfSignature = {
      id: `sig-${Date.now()}`,
      page: 1,
      x: 65,
      y: 82,
      dataUrl: typedSignature,
      author: 'John Doe',
      date: new Date().toLocaleDateString(),
    };
    setSignatures([...signatures, newSig]);
    setShowSignatureModal(false);
  };

  // Canvas drawing handlers
  const startDrawing = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = sigCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    setIsDrawing(true);
    const rect = canvas.getBoundingClientRect();
    ctx.beginPath();
    ctx.moveTo(e.clientX - rect.left, e.clientY - rect.top);
  };

  const draw = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isDrawing) return;
    const canvas = sigCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const rect = canvas.getBoundingClientRect();
    ctx.lineTo(e.clientX - rect.left, e.clientY - rect.top);
    ctx.strokeStyle = '#2b579a';
    ctx.lineWidth = 2.5;
    ctx.stroke();
  };

  const stopDrawing = () => {
    setIsDrawing(false);
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
              onClick={() => setActiveRibbonTab('annotate')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'annotate' ? 'is-active' : ''}`}
            >
              Annotate & Mark
            </button>
            <button
              onClick={() => setActiveRibbonTab('edit')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'edit' ? 'is-active' : ''}`}
            >
              Edit PDF Content
            </button>
            <button
              onClick={() => setActiveRibbonTab('convert')}
              className={`wps-ribbon-tab-btn ${activeRibbonTab === 'convert' ? 'is-active' : ''}`}
            >
              Convert & Sign
            </button>
          </div>

          <div className="flex items-center gap-2 text-xs text-mercury-slate">
            <span>WPS PDF Toolkit</span>
          </div>
        </div>

        {/* Ribbon Body Actions */}
        <div className="wps-ribbon-body">
          {/* HOME TAB */}
          {activeRibbonTab === 'home' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setActiveTool('select')}
                    className={`wps-ribbon-btn ${activeTool === 'select' ? 'is-active' : ''}`}
                  >
                    <MousePointer className="w-5 h-5 text-blue-600" />
                    <span>Select Text</span>
                  </button>
                  <button
                    onClick={() => setActiveTool('pan')}
                    className={`wps-ribbon-btn ${activeTool === 'pan' ? 'is-active' : ''}`}
                  >
                    <Hand className="w-5 h-5 text-amber-600" />
                    <span>Hand Pan</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Navigation</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <div className="relative">
                    <Search className="w-3.5 h-3.5 text-gray-400 absolute left-2.5 top-1/2 -translate-y-1/2" />
                    <input
                      type="text"
                      placeholder="Find in PDF..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="h-8 pl-8 pr-3 text-xs border border-mercury-fog/50 rounded-full outline-none focus:border-mercury-primary"
                    />
                  </div>
                </div>
                <span className="wps-ribbon-group-title">Search</span>
              </div>
            </>
          )}

          {/* ANNOTATE TAB */}
          {activeRibbonTab === 'annotate' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => {
                      setActiveTool('highlight');
                      setHighlightColor('yellow');
                    }}
                    className={`wps-ribbon-btn ${
                      activeTool === 'highlight' && highlightColor === 'yellow' ? 'is-active' : ''
                    }`}
                  >
                    <Highlighter className="w-5 h-5 text-yellow-500" />
                    <span>Yellow Highlight</span>
                  </button>
                  <button
                    onClick={() => {
                      setActiveTool('highlight');
                      setHighlightColor('lavender');
                    }}
                    className={`wps-ribbon-btn ${
                      activeTool === 'highlight' && highlightColor === 'lavender' ? 'is-active' : ''
                    }`}
                  >
                    <Highlighter className="w-5 h-5 text-purple-500" />
                    <span>Lavender</span>
                  </button>
                  <button
                    onClick={() => {
                      setActiveTool('highlight');
                      setHighlightColor('periwinkle');
                    }}
                    className={`wps-ribbon-btn ${
                      activeTool === 'highlight' && highlightColor === 'periwinkle' ? 'is-active' : ''
                    }`}
                  >
                    <Highlighter className="w-5 h-5 text-blue-500" />
                    <span>Periwinkle</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Highlights</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => {
                      const newNote: PdfStickyNote = {
                        id: `note-${Date.now()}`,
                        page: 1,
                        x: 82,
                        y: 40,
                        text: 'Review pricing schedule with finance team.',
                        author: 'Current User',
                        date: 'Just now',
                      };
                      setNotes([...notes, newNote]);
                    }}
                    className="wps-ribbon-btn"
                  >
                    <MessageSquare className="w-5 h-5 text-amber-600" />
                    <span>Add Sticky Note</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Notes</span>
              </div>
            </>
          )}

          {/* EDIT PDF CONTENT TAB */}
          {activeRibbonTab === 'edit' && (
            <>
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setIsEditingText(!isEditingText)}
                    className={`wps-ribbon-btn ${isEditingText ? 'is-active' : ''}`}
                  >
                    <FileText className="w-5 h-5 text-blue-600" />
                    <span>{isEditingText ? 'Editing Active' : 'Edit PDF Text Inline'}</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Text Reflow Mode</span>
              </div>

              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => alert('Page rotated 90 degrees.')}
                    className="wps-ribbon-btn"
                  >
                    <RotateCw className="w-5 h-5 text-mercury-slate" />
                    <span>Rotate 90°</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Page Manipulation</span>
              </div>
            </>
          )}

          {/* CONVERT & SIGN TAB */}
          {activeRibbonTab === 'convert' && (
            <>
              {/* Convert to Word & Excel */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={onConvertToWord}
                    className="wps-ribbon-btn text-blue-700 hover:bg-blue-50"
                  >
                    <FileText className="w-5 h-5 text-blue-600" />
                    <span className="font-semibold">PDF to Word (.docx)</span>
                  </button>
                  <button
                    onClick={onConvertToExcel}
                    className="wps-ribbon-btn text-emerald-700 hover:bg-emerald-50"
                  >
                    <Table className="w-5 h-5 text-emerald-600" />
                    <span className="font-semibold">PDF to Excel (.xlsx)</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Instant Conversion</span>
              </div>

              {/* Sign PDF */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={() => setShowSignatureModal(true)}
                    className="wps-ribbon-btn text-indigo-700 hover:bg-indigo-50"
                  >
                    <FileCheck className="w-5 h-5 text-indigo-600" />
                    <span className="font-semibold">Sign PDF</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Signatures</span>
              </div>

              {/* Redaction */}
              <div className="wps-ribbon-group">
                <div className="wps-ribbon-controls">
                  <button
                    onClick={handleRedactPii}
                    className="wps-ribbon-btn text-rose-700 hover:bg-rose-50"
                  >
                    <ShieldAlert className="w-5 h-5 text-rose-600" />
                    <span className="font-semibold">Redact Sensitive Info</span>
                  </button>
                </div>
                <span className="wps-ribbon-group-title">Security</span>
              </div>
            </>
          )}
        </div>
      </div>

      {/* ─── Zone 3: Framed PDF Page Canvas ─── */}
      <div className="wps-canvas-container">
        <div className="wps-doc-viewport">
          <div className="wps-paper-sheet relative">
            {/* ISO 32000 PDF Watermark / Status Header */}
            <div className="flex items-center justify-between pb-4 mb-6 border-b border-gray-200 text-xs text-mercury-slate">
              <span className="font-semibold tracking-wider text-rose-600">ISO 32000-2 CERTIFIED DOCUMENT</span>
              <span>Page 1 of 1</span>
            </div>

            {/* Render PDF Blocks */}
            {blocks.map((block) => {
              const isTitle = block.role === 'title';
              const isHeading = block.role === 'heading';
              const isTable = block.role === 'table';

              // Highlight styling
              let highlightBg = undefined;
              if (block.highlight === 'yellow') highlightBg = 'rgba(254, 240, 138, 0.6)';
              if (block.highlight === 'lavender') highlightBg = 'rgba(237, 236, 251, 0.8)';
              if (block.highlight === 'periwinkle') highlightBg = 'rgba(156, 180, 232, 0.6)';

              return (
                <div
                  key={block.id}
                  onClick={() => {
                    if (activeTool === 'highlight') {
                      handleToggleHighlight(block.id);
                    }
                  }}
                  className={`mb-4 p-2 rounded transition-all relative ${
                    isEditingText ? 'border border-dashed border-mercury-primary/50 hover:bg-blue-50/20' : ''
                  }`}
                  style={{ backgroundColor: highlightBg }}
                >
                  {isTable ? (
                    <div>
                      <h4 className="font-semibold text-xs text-gray-700 mb-2">{block.text}</h4>
                      <table className="w-full border-collapse border border-gray-300 text-xs">
                        <tbody>
                          {block.tableData?.map((row, rIdx) => (
                            <tr key={rIdx}>
                              {row.map((cell, cIdx) => (
                                <td
                                  key={cIdx}
                                  className={`border border-gray-300 p-2 ${
                                    rIdx === 0 ? 'bg-gray-100 font-semibold text-gray-800' : 'text-gray-700'
                                  }`}
                                >
                                  {cell}
                                </td>
                              ))}
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  ) : (
                    <div
                      contentEditable={isEditingText}
                      suppressContentEditableWarning
                      onBlur={(e) => handleBlockTextChange(block.id, e.currentTarget.innerText)}
                      className={`outline-none leading-relaxed ${
                        isTitle ? 'text-2xl font-bold tracking-tight text-center text-gray-900 mb-6' : ''
                      } ${isHeading ? 'text-base font-semibold text-gray-800 mt-4 mb-2' : ''} ${
                        !isTitle && !isHeading ? 'text-xs text-gray-700 text-justify' : ''
                      }`}
                    >
                      {block.redacted ? (
                        <span>
                          {block.text.split('[REDACTED-SSN]').map((part, i, arr) => (
                            <React.Fragment key={i}>
                              {part}
                              {i < arr.length - 1 && (
                                <span className="mercury-redaction-mask">
                                  [REDACTED-SSN - PURGED]
                                </span>
                              )}
                            </React.Fragment>
                          ))}
                        </span>
                      ) : (
                        block.text
                      )}
                    </div>
                  )}
                </div>
              );
            })}

            {/* Render Placed Signatures */}
            {signatures.map((sig) => (
              <div
                key={sig.id}
                className="absolute p-3 bg-white/90 border border-indigo-400 rounded shadow-md flex flex-col gap-1 z-20"
                style={{ left: `${sig.x}%`, top: `${sig.y}%` }}
              >
                <div className="font-serif italic text-lg text-blue-900 border-b border-gray-300 pb-1">
                  {sig.dataUrl}
                </div>
                <div className="text-[10px] text-gray-500 flex items-center justify-between gap-4">
                  <span>Digitally Signed by: {sig.author}</span>
                  <span>{sig.date}</span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Right Sticky Notes Drawer */}
        {notes.length > 0 && (
          <aside className="w-72 bg-white border-l border-mercury-fog p-3 flex flex-col gap-3 overflow-y-auto">
            <div className="flex items-center justify-between text-xs font-semibold text-mercury-slate">
              <span>PDF Sticky Notes ({notes.length})</span>
            </div>

            {notes.map((n) => (
              <div
                key={n.id}
                className="p-3 rounded-mercury-md bg-amber-50 border border-amber-200 text-xs flex flex-col gap-1.5 shadow-sm"
              >
                <div className="flex items-center justify-between">
                  <span className="font-semibold text-amber-900">{n.author}</span>
                  <span className="text-[10px] text-amber-700">{n.date}</span>
                </div>
                <p className="text-amber-950 text-xs leading-relaxed">{n.text}</p>
                <button
                  onClick={() => setNotes((prev) => prev.filter((item) => item.id !== n.id))}
                  className="self-end text-[10px] text-amber-800 hover:underline mt-1 font-medium"
                >
                  Delete Note
                </button>
              </div>
            ))}
          </aside>
        )}
      </div>

      {/* ─── Signature Modal ─── */}
      {showSignatureModal && (
        <div className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4">
          <div className="bg-white rounded-mercury-md shadow-2xl border border-mercury-fog max-w-md w-full p-6 flex flex-col gap-4">
            <div className="flex items-center justify-between">
              <h3 className="font-semibold text-base text-mercury-ink">Create Digital Signature</h3>
              <button onClick={() => setShowSignatureModal(false)} className="p-1 rounded hover:bg-gray-100">
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex items-center bg-gray-100 rounded-full p-1 text-xs">
              <button
                onClick={() => setSignatureMode('type')}
                className={`flex-1 py-1 rounded-full text-center transition-colors ${
                  signatureMode === 'type' ? 'bg-white font-semibold text-mercury-primary shadow-sm' : 'text-gray-600'
                }`}
              >
                Type Signature
              </button>
              <button
                onClick={() => setSignatureMode('draw')}
                className={`flex-1 py-1 rounded-full text-center transition-colors ${
                  signatureMode === 'draw' ? 'bg-white font-semibold text-mercury-primary shadow-sm' : 'text-gray-600'
                }`}
              >
                Draw Signature
              </button>
            </div>

            {signatureMode === 'type' ? (
              <div className="flex flex-col gap-2">
                <label className="text-xs text-gray-500">Full Name</label>
                <input
                  type="text"
                  value={typedSignature}
                  onChange={(e) => setTypedSignature(e.target.value)}
                  className="h-10 px-3 text-sm border border-gray-300 rounded outline-none focus:border-mercury-primary font-serif italic text-lg"
                />
              </div>
            ) : (
              <div className="flex flex-col gap-2">
                <label className="text-xs text-gray-500">Draw with your mouse or trackpad:</label>
                <canvas
                  ref={sigCanvasRef}
                  width={380}
                  height={120}
                  onMouseDown={startDrawing}
                  onMouseMove={draw}
                  onMouseUp={stopDrawing}
                  onMouseLeave={stopDrawing}
                  className="border border-gray-300 rounded bg-gray-50 cursor-crosshair"
                />
              </div>
            )}

            <div className="flex items-center justify-end gap-2 mt-2">
              <button
                onClick={() => setShowSignatureModal(false)}
                className="px-4 py-2 rounded-full border border-gray-300 text-xs font-medium hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleApplySignature}
                className="button-primary px-5 py-2 text-xs"
              >
                Place Signature onto PDF
              </button>
            </div>
          </div>
        </div>
      )}

      {/* ─── Zone 5: Bottom Status Bar (28px) ─── */}
      <footer className="wps-status-bar">
        <div className="flex items-center gap-4">
          <span>Page 1 of 1</span>
          <span>Security: Signed & Verified</span>
          {isEditingText && <span className="text-blue-400 font-medium">Text Editing: Active</span>}
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
