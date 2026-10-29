import React, { useState } from 'react';
import {
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
  Heading1,
  Heading2,
  Table as TableIcon,
  Image as ImageIcon,
  Sparkles,
  FunctionSquare,
  Filter,
  BarChart3,
  Split,
  Combine,
  Search,
  CheckCircle2,
  Wand2,
  PenTool,
  Highlighter,
  Scissors,
  FileCheck2,
  Lock,
  Eye,
  SlidersHorizontal,
  ChevronDown,
  Layers,
  PlaySquare,
  FileCode2,
  FileDown,
  Columns,
  BookOpen,
  Mail,
  FileText,
  Bookmark,
  Plus,
  RotateCw,
  RotateCcw,
  Crop,
  Trash2,
  Ruler,
  MessageSquare,
  Stamp,
  ShieldCheck,
  KeyRound,
  Type,
  FileSpreadsheet,
  Presentation,
  Cloud,
  Eraser,
  FileSignature,
  ArrowRight,
  ShieldAlert,
} from 'lucide-react';
import { ActiveRibbonTab, DocumentType, RibbonMode, SmartToolboxAction } from '../types';

interface RibbonBarProps {
  docType: DocumentType;
  mode: RibbonMode;
  activeTab: ActiveRibbonTab;
  onTabChange: (tab: ActiveRibbonTab) => void;
  onToggleMode: () => void;
  onExecuteCommand: (commandId: string, payload?: any) => void;
  onOpenSmartToolbox: (action: SmartToolboxAction) => void;
}

export const RibbonBar: React.FC<RibbonBarProps> = ({
  docType,
  mode,
  activeTab,
  onTabChange,
  onToggleMode,
  onExecuteCommand,
  onOpenSmartToolbox,
}) => {
  const [showTablePicker, setShowTablePicker] = useState(false);
  const [hoverGrid, setHoverGrid] = useState({ rows: 3, cols: 3 });
  const [showShapesPicker, setShowShapesPicker] = useState(false);
  const [showConnectorPicker, setShowConnectorPicker] = useState(false);

  // Contextual Tabs per Document Type
  const getTabsForDocType = (): { id: ActiveRibbonTab; label: string }[] => {
    switch (docType) {
      case 'writer':
        return [
          { id: 'home', label: 'Home' },
          { id: 'insert', label: 'Insert' },
          { id: 'layout', label: 'Page Layout' },
          { id: 'references', label: 'References & Mailings' },
          { id: 'review', label: 'Review & Track Changes' },
          { id: 'view', label: 'View' },
          { id: 'ai', label: 'WPS AI' },
        ];
      case 'sheets':
        return [
          { id: 'home', label: 'Home' },
          { id: 'insert', label: 'Insert' },
          { id: 'formulas', label: 'Formulas (150+)' },
          { id: 'data', label: 'Data & Pivot' },
          { id: 'toolbox', label: 'WPS Smart Toolbox' },
          { id: 'review', label: 'Review' },
          { id: 'view', label: 'View' },
          { id: 'ai', label: 'WPS AI' },
        ];
      case 'slides':
        return [
          { id: 'home', label: 'Home' },
          { id: 'insert', label: 'Insert' },
          { id: 'slides', label: 'Slide Master' },
          { id: 'transitions', label: 'Transitions & Morph' },
          { id: 'animations', label: 'Animation Stage' },
          { id: 'view', label: 'Presenter View' },
          { id: 'ai', label: 'WPS AI' },
        ];
      case 'pdf':
        return [
          { id: 'home', label: 'Home' },
          { id: 'pdf_annotate', label: 'Annotate & Ink' },
          { id: 'pdf_edit', label: 'Direct Text Edit' },
          { id: 'pdf_organize', label: 'Page Organizer' },
          { id: 'pdf_convert', label: 'PDF to Office & OCR' },
          { id: 'review', label: 'Security & Sign' },
          { id: 'ai', label: 'WPS AI' },
        ];
    }
  };

  const tabs = getTabsForDocType();

  return (
    <div className="bg-white dark:bg-mercury-night-raised border-b border-mercury-fog/30 flex flex-col select-none transition-colors">
      {/* Ribbon Tab Strip */}
      <div className="flex items-center justify-between px-3 h-8 border-b border-mercury-fog/20 text-xs">
        <div className="flex space-x-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => onTabChange(tab.id)}
              className={`px-3 py-1 rounded-t-mercury-sm font-medium transition-all ${
                activeTab === tab.id
                  ? 'text-mercury-primary border-b-2 border-mercury-primary bg-mercury-lavender/30 dark:bg-mercury-night'
                  : 'text-mercury-ink-secondary dark:text-mercury-fog hover:text-mercury-ink dark:hover:text-mercury-on-dark'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Compact / Classic Ribbon Mode Toggle */}
        <button
          onClick={onToggleMode}
          className="flex items-center space-x-1 px-2 py-0.5 rounded-mercury-sm text-[11px] text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night transition-colors"
          title="Toggle Ribbon Mode (Compact / Classic)"
        >
          <SlidersHorizontal className="w-3 h-3" />
          <span>{mode === 'classic' ? 'Compact Ribbon' : 'Classic Ribbon'}</span>
        </button>
      </div>

      {/* Ribbon Tool Body */}
      <div
        className={`px-3 flex items-center overflow-x-auto no-scrollbar transition-all ${
          mode === 'classic' ? 'h-[72px] py-1.5' : 'h-10 py-1'
        }`}
      >
        {/* ===================== HOME TAB ===================== */}
        {activeTab === 'home' && docType === 'pdf' ? (
          <div className="flex items-center space-x-3 text-xs">
            {/* View & Edit Modes */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_set_mode', 'read')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Continuous Page Read View"
              >
                <BookOpen className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Read View</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_set_mode', 'edit_content')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm bg-mercury-pdf text-white font-semibold shadow-sm hover:opacity-95 transition"
                title="Direct Inline Paragraph & Line Reflow Editing"
              >
                <CheckCircle2 className="w-3.5 h-3.5" />
                <span>Edit PDF Like Word</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_set_mode', 'reflow')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Responsive Single-Column Reading Mode"
              >
                <Columns className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Reflow Mode</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_set_mode', 'organize')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Visual Page Organizer Grid"
              >
                <Layers className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Page Grid</span>
              </button>
            </div>

            {/* Quick Annotation Tools */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'highlight', color: '#ffeb3b' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm bg-yellow-400/20 text-yellow-700 dark:text-yellow-300 font-medium"
                title="Highlight Selection"
              >
                <Highlighter className="w-3.5 h-3.5" />
                <span>Highlight</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'sticky_note', text: 'Important observation', author: 'Auditor' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Add Sticky Note"
              >
                <MessageSquare className="w-3.5 h-3.5 text-amber-500" />
                <span>Sticky Note</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'stamp', stamp_type: 'Approved' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-emerald-500/30 text-emerald-600 hover:bg-emerald-500/10 font-medium"
                title="Stamp Approved"
              >
                <Stamp className="w-3.5 h-3.5" />
                <span>Approved Stamp</span>
              </button>
            </div>

            {/* Instant Cross-Engine Export */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_to_word')}
                className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-writer/40 text-mercury-writer hover:bg-mercury-writer/10 font-semibold transition"
                title="Convert PDF to Word (.docx) AST"
              >
                <FileDown className="w-3.5 h-3.5" />
                <span>PDF &rarr; Word</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_to_excel')}
                className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-sheets/40 text-mercury-sheets hover:bg-mercury-sheets/10 font-semibold transition"
                title="Convert PDF to Excel (.xlsx) Workbook"
              >
                <FileSpreadsheet className="w-3.5 h-3.5" />
                <span>PDF &rarr; Excel</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_to_ppt')}
                className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-slides/40 text-mercury-slides hover:bg-mercury-slides/10 font-semibold transition"
                title="Convert PDF to PowerPoint (.pptx) Deck"
              >
                <Presentation className="w-3.5 h-3.5" />
                <span>PDF &rarr; PPT</span>
              </button>
            </div>

            {/* Quick Security & Redact */}
            <div className="flex items-center space-x-1">
              <button
                onClick={() => onExecuteCommand('pdf_redact_region')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-red-500/30 text-red-600 hover:bg-red-500/10 font-medium"
                title="Permanently scrub bytes and redact region"
              >
                <Scissors className="w-3.5 h-3.5" />
                <span>Redact Area</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_open_sidebar', 'search_redact')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Search and Redact All Occurrences"
              >
                <Search className="w-3.5 h-3.5 text-mercury-ink-secondary" />
                <span>Search & Redact</span>
              </button>
            </div>
          </div>
        ) : activeTab === 'home' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* Clipboard Group */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('paste')}
                className="flex flex-col items-center justify-center p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Paste (Ctrl/Cmd + V)"
              >
                <FileDown className="w-4 h-4 text-mercury-ink dark:text-mercury-on-dark" />
                {mode === 'classic' && <span className="text-[10px] mt-0.5">Paste</span>}
              </button>
            </div>

            {/* Typography Group */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <select
                onChange={(e) => onExecuteCommand('set_font_family', e.target.value)}
                className="h-7 px-2 border border-mercury-fog/40 rounded-mercury-sm text-xs bg-white dark:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark outline-none focus:border-mercury-primary"
                defaultValue="Inter"
              >
                <option value="Inter">Inter (Mercury Standard)</option>
                <option value="JetBrains Mono">JetBrains Mono</option>
                <option value="Arial">Arial</option>
                <option value="Times New Roman">Times New Roman</option>
              </select>

              <select
                onChange={(e) => onExecuteCommand('set_font_size', Number(e.target.value))}
                className="h-7 px-1.5 border border-mercury-fog/40 rounded-mercury-sm text-xs bg-white dark:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark outline-none focus:border-mercury-primary tabular-nums"
                defaultValue="11"
              >
                <option value="9">9 pt</option>
                <option value="10">10 pt</option>
                <option value="11">11 pt</option>
                <option value="12">12 pt</option>
                <option value="14">14 pt</option>
                <option value="16">16 pt</option>
                <option value="18">18 pt</option>
                <option value="24">24 pt</option>
              </select>

              <div className="flex items-center space-x-0.5 ml-1">
                <button
                  onClick={() => onExecuteCommand('toggle_bold')}
                  className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                  title="Bold (Ctrl/Cmd + B)"
                >
                  <Bold className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={() => onExecuteCommand('toggle_italic')}
                  className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                  title="Italic (Ctrl/Cmd + I)"
                >
                  <Italic className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={() => onExecuteCommand('toggle_underline')}
                  className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                  title="Underline (Ctrl/Cmd + U)"
                >
                  <Underline className="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={() => onExecuteCommand('toggle_strike')}
                  className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                  title="Strikethrough"
                >
                  <Strikethrough className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>

            {/* Paragraph / Alignment Group */}
            <div className="flex items-center space-x-0.5 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('align_left')}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Align Left"
              >
                <AlignLeft className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => onExecuteCommand('align_center')}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Align Center"
              >
                <AlignCenter className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => onExecuteCommand('align_right')}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Align Right"
              >
                <AlignRight className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => onExecuteCommand('align_justify')}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Justify"
              >
                <AlignJustify className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => onExecuteCommand('toggle_bullet_list')}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Bullets"
              >
                <List className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => onExecuteCommand('toggle_numbered_list')}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Numbering"
              >
                <ListOrdered className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Styles Gallery (Writer & Presentation) */}
            {(docType === 'writer' || docType === 'slides') && (
              <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
                <button
                  onClick={() => onExecuteCommand('apply_style', 'Normal')}
                  className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary text-xs"
                >
                  Normal
                </button>
                <button
                  onClick={() => onExecuteCommand('apply_style', 'Heading 1')}
                  className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary text-xs font-semibold text-mercury-primary"
                >
                  Heading 1
                </button>
                <button
                  onClick={() => onExecuteCommand('apply_style', 'Heading 2')}
                  className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary text-xs font-medium text-mercury-ink-secondary"
                >
                  Heading 2
                </button>
              </div>
            )}

            {/* Quick Insert Table / Image */}
            <div className="flex items-center space-x-1">
              <button
                onClick={() => onExecuteCommand('insert_table')}
                className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
              >
                <TableIcon className="w-3.5 h-3.5 text-mercury-sheets" />
                <span>Table</span>
              </button>
              <button
                onClick={() => onExecuteCommand('insert_image')}
                className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
              >
                <ImageIcon className="w-3.5 h-3.5 text-mercury-slides" />
                <span>Image</span>
              </button>
            </div>
          </div>
        )}

        {/* ===================== INSERT TAB ===================== */}
        {activeTab === 'insert' && (
          <div className="flex items-center space-x-2 text-xs relative">
            {/* 10x8 Table Grid Picker */}
            <div className="relative">
              <button
                onClick={() => setShowTablePicker(!showTablePicker)}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
              >
                <TableIcon className="w-3.5 h-3.5 text-mercury-sheets" />
                <span>Table Grid</span>
                <ChevronDown className="w-3 h-3 ml-0.5 text-mercury-fog" />
              </button>

              {showTablePicker && (
                <div className="absolute top-9 left-0 z-50 bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-md shadow-2xl p-3 animate-in fade-in zoom-in-95">
                  <div className="text-[11px] font-medium text-mercury-ink dark:text-mercury-on-dark mb-2">
                    {hoverGrid.rows > 0 ? `${hoverGrid.rows} × ${hoverGrid.cols} Table` : 'Select grid dimensions'}
                  </div>
                  <div className="grid grid-cols-10 gap-1 p-1 bg-mercury-canvas dark:bg-mercury-night rounded border border-mercury-fog/20">
                    {Array.from({ length: 8 }).map((_, r) =>
                      Array.from({ length: 10 }).map((_, c) => {
                        const isHighlighted = r < hoverGrid.rows && c < hoverGrid.cols;
                        return (
                          <div
                            key={`${r}-${c}`}
                            onMouseEnter={() => setHoverGrid({ rows: r + 1, cols: c + 1 })}
                            onClick={() => {
                              onExecuteCommand('insert_table_grid', { rows: r + 1, cols: c + 1 });
                              setShowTablePicker(false);
                            }}
                            className={`w-3.5 h-3.5 rounded-xs border cursor-pointer transition-colors ${
                              isHighlighted
                                ? 'bg-mercury-primary/40 border-mercury-primary'
                                : 'bg-white dark:bg-mercury-night-raised border-mercury-fog/30 hover:border-mercury-primary'
                            }`}
                          />
                        );
                      })
                    )}
                  </div>
                </div>
              )}
            </div>

            {/* 50+ Parametric Shapes Popover */}
            <div className="relative">
              <button
                onClick={() => setShowShapesPicker(!showShapesPicker)}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
              >
                <SlidersHorizontal className="w-3.5 h-3.5 text-mercury-slides" />
                <span>Shapes (50+)</span>
                <ChevronDown className="w-3 h-3 ml-0.5 text-mercury-fog" />
              </button>

              {showShapesPicker && (
                <div className="absolute top-9 left-0 z-50 bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-md shadow-2xl p-4 w-72 animate-in fade-in zoom-in-95 max-h-80 overflow-y-auto">
                  <div className="text-[11px] font-bold text-mercury-ink-secondary mb-1 uppercase tracking-wider">Basic Shapes</div>
                  <div className="grid grid-cols-4 gap-1.5 mb-3">
                    {['Rectangle', 'RoundedRect', 'Ellipse', 'Triangle', 'Diamond', 'Parallelogram', 'Trapezoid', 'Hexagon', 'Octagon', 'Donut', 'Heart', 'Lightning'].map((shape) => (
                      <button
                        key={shape}
                        onClick={() => {
                          onExecuteCommand('insert_shape', shape);
                          setShowShapesPicker(false);
                        }}
                        className="p-1.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night text-[11px] truncate text-center border border-mercury-fog/20"
                        title={shape}
                      >
                        {shape}
                      </button>
                    ))}
                  </div>

                  <div className="text-[11px] font-bold text-mercury-ink-secondary mb-1 uppercase tracking-wider">Block Arrows</div>
                  <div className="grid grid-cols-4 gap-1.5 mb-3">
                    {['RightArrow', 'LeftArrow', 'UpArrow', 'DownArrow', 'LeftRight', 'Chevron', 'QuadArrow'].map((shape) => (
                      <button
                        key={shape}
                        onClick={() => {
                          onExecuteCommand('insert_shape', shape);
                          setShowShapesPicker(false);
                        }}
                        className="p-1.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night text-[11px] truncate text-center border border-mercury-fog/20"
                        title={shape}
                      >
                        {shape}
                      </button>
                    ))}
                  </div>

                  <div className="text-[11px] font-bold text-mercury-ink-secondary mb-1 uppercase tracking-wider">Stars & Banners</div>
                  <div className="grid grid-cols-4 gap-1.5 mb-3">
                    {['Star4', 'Star5', 'Star6', 'Star8', 'Star12', 'Star24'].map((shape) => (
                      <button
                        key={shape}
                        onClick={() => {
                          onExecuteCommand('insert_shape', shape);
                          setShowShapesPicker(false);
                        }}
                        className="p-1.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night text-[11px] truncate text-center border border-mercury-fog/20"
                        title={shape}
                      >
                        {shape}
                      </button>
                    ))}
                  </div>

                  <div className="text-[11px] font-bold text-mercury-ink-secondary mb-1 uppercase tracking-wider">Flowchart</div>
                  <div className="grid grid-cols-3 gap-1.5">
                    {['Process', 'Decision', 'Document', 'Data', 'Terminator'].map((shape) => (
                      <button
                        key={shape}
                        onClick={() => {
                          onExecuteCommand('insert_shape', shape);
                          setShowShapesPicker(false);
                        }}
                        className="p-1.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night text-[11px] truncate text-center border border-mercury-fog/20"
                        title={shape}
                      >
                        {shape}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>

            {/* Smart Connectors Popover */}
            <div className="relative">
              <button
                onClick={() => setShowConnectorPicker(!showConnectorPicker)}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
              >
                <PenTool className="w-3.5 h-3.5 text-mercury-slides" />
                <span>Smart Connectors</span>
                <ChevronDown className="w-3 h-3 ml-0.5 text-mercury-fog" />
              </button>

              {showConnectorPicker && (
                <div className="absolute top-9 left-0 z-50 bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-md shadow-2xl p-2 w-48 animate-in fade-in zoom-in-95">
                  <button
                    onClick={() => {
                      onExecuteCommand('insert_connector', 'orthogonal');
                      setShowConnectorPicker(false);
                    }}
                    className="w-full text-left px-2.5 py-1.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night text-xs flex items-center justify-between"
                  >
                    <span>A* Orthogonal Connector</span>
                    <span className="text-[10px] text-mercury-slides font-bold">90°</span>
                  </button>
                  <button
                    onClick={() => {
                      onExecuteCommand('insert_connector', 'curved');
                      setShowConnectorPicker(false);
                    }}
                    className="w-full text-left px-2.5 py-1.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night text-xs flex items-center justify-between"
                  >
                    <span>Cubic Bézier Curved</span>
                    <span className="text-[10px] text-mercury-primary font-bold">~</span>
                  </button>
                  <button
                    onClick={() => {
                      onExecuteCommand('insert_connector', 'straight');
                      setShowConnectorPicker(false);
                    }}
                    className="w-full text-left px-2.5 py-1.5 rounded hover:bg-mercury-lavender dark:hover:bg-mercury-night text-xs flex items-center justify-between"
                  >
                    <span>Straight Line</span>
                    <span className="text-[10px] text-mercury-ink-secondary font-bold">/</span>
                  </button>
                </div>
              )}
            </div>

            <button
              onClick={() => onExecuteCommand('insert_toc')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <BookOpen className="w-3.5 h-3.5 text-mercury-primary" />
              <span>Table of Contents</span>
            </button>

            <button
              onClick={() => onExecuteCommand('insert_page_break')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <FileText className="w-3.5 h-3.5 text-mercury-ink-secondary" />
              <span>Page Break</span>
            </button>

            <button
              onClick={() => onExecuteCommand('insert_section_break')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Split className="w-3.5 h-3.5 text-mercury-ink-secondary" />
              <span>Section Break</span>
            </button>

            <button
              onClick={() => onExecuteCommand('insert_image')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <ImageIcon className="w-3.5 h-3.5 text-mercury-slides" />
              <span>Picture</span>
            </button>

            <button
              onClick={() => onExecuteCommand('insert_footnote')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Plus className="w-3.5 h-3.5 text-mercury-primary" />
              <span>Footnote</span>
            </button>

            <button
              onClick={() => onExecuteCommand('insert_bookmark')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Bookmark className="w-3.5 h-3.5 text-mercury-ink-secondary" />
              <span>Bookmark</span>
            </button>
          </div>
        )}

        {/* ===================== PAGE LAYOUT TAB ===================== */}
        {activeTab === 'layout' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* Margins */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] text-mercury-ink-secondary mr-1">Margins:</span>
              <button
                onClick={() => onExecuteCommand('set_margins', { top: 1440, bottom: 1440, left: 1440, right: 1440 })}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                Normal (1.0")
              </button>
              <button
                onClick={() => onExecuteCommand('set_margins', { top: 720, bottom: 720, left: 720, right: 720 })}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                Narrow (0.5")
              </button>
              <button
                onClick={() => onExecuteCommand('set_margins', { top: 1440, bottom: 1440, left: 2880, right: 2880 })}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                Wide (2.0")
              </button>
            </div>

            {/* Orientation */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] text-mercury-ink-secondary mr-1">Orientation:</span>
              <button
                onClick={() => onExecuteCommand('set_orientation', false)}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                Portrait
              </button>
              <button
                onClick={() => onExecuteCommand('set_orientation', true)}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                Landscape
              </button>
            </div>

            {/* Paper Size */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] text-mercury-ink-secondary mr-1">Size:</span>
              <button
                onClick={() => onExecuteCommand('set_page_size', 'A4')}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                A4
              </button>
              <button
                onClick={() => onExecuteCommand('set_page_size', 'Letter')}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                US Letter
              </button>
            </div>

            {/* Multi-Columns */}
            <div className="flex items-center space-x-1">
              <Columns className="w-3.5 h-3.5 text-mercury-ink-secondary mr-1" />
              <button
                onClick={() => onExecuteCommand('set_columns', 1)}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                1 Col
              </button>
              <button
                onClick={() => onExecuteCommand('set_columns', 2)}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                2 Cols
              </button>
              <button
                onClick={() => onExecuteCommand('set_columns', 3)}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-primary"
              >
                3 Cols
              </button>
            </div>
          </div>
        )}

        {/* ===================== REFERENCES & MAILINGS TAB ===================== */}
        {activeTab === 'references' && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onExecuteCommand('update_toc')}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-writer text-white font-medium shadow-sm hover:bg-mercury-writer/90"
            >
              <BookOpen className="w-3.5 h-3.5" />
              <span>Update Table of Contents</span>
            </button>

            <button
              onClick={() => onExecuteCommand('open_mail_merge')}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-primary/10 text-mercury-primary border border-mercury-primary/30 font-medium hover:bg-mercury-primary/20"
            >
              <Mail className="w-3.5 h-3.5" />
              <span>Mail Merge Batch Generator</span>
            </button>

            <button
              onClick={() => onExecuteCommand('insert_footnote')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Insert Footnote
            </button>

            <button
              onClick={() => onExecuteCommand('insert_endnote')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Insert Endnote
            </button>
          </div>
        )}

        {/* ===================== FORMULAS TAB (SHEETS) ===================== */}
        {activeTab === 'formulas' && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onExecuteCommand('insert_formula', '=SUM(')}
              className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm bg-mercury-sheets/10 text-mercury-sheets hover:bg-mercury-sheets/20 font-medium"
            >
              <FunctionSquare className="w-3.5 h-3.5" />
              <span>AutoSum</span>
            </button>
            <button
              onClick={() => onExecuteCommand('insert_formula', '=XLOOKUP(')}
              className="px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              XLOOKUP
            </button>
            <button
              onClick={() => onExecuteCommand('insert_formula', '=SUMIFS(')}
              className="px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              SUMIFS
            </button>
            <button
              onClick={() => onExecuteCommand('insert_formula', '=FILTER(')}
              className="px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              FILTER (Array)
            </button>
            <button
              onClick={() => onExecuteCommand('insert_formula', '=PY("')}
              className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-pill bg-mercury-ai/10 text-mercury-ai hover:bg-mercury-ai/20 font-medium"
            >
              <FileCode2 className="w-3.5 h-3.5" />
              <span>=PY() Python</span>
            </button>
          </div>
        )}

        {/* ===================== DATA & PIVOT TAB (SHEETS) ===================== */}
        {activeTab === 'data' && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onOpenSmartToolbox('pivot_wizard')}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-sheets text-white hover:bg-mercury-sheets/90 font-medium shadow-sm"
            >
              <TableIcon className="w-3.5 h-3.5" />
              <span>Pivot Table Cube</span>
            </button>

            <div className="flex items-center space-x-1 px-2 border-l border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('sort_range', { ascending: true })}
                className="flex items-center space-x-1 px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Sort Ascending A -> Z"
              >
                <span>Sort A &rarr; Z</span>
              </button>
              <button
                onClick={() => onExecuteCommand('sort_range', { ascending: false })}
                className="flex items-center space-x-1 px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Sort Descending Z -> A"
              >
                <span>Sort Z &rarr; A</span>
              </button>
            </div>

            <button
              onClick={() => onExecuteCommand('toggle_autofilter')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Filter className="w-3.5 h-3.5 text-mercury-sheets" />
              <span>AutoFilter</span>
            </button>

            <button
              onClick={() => onOpenSmartToolbox('data_validation')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <CheckCircle2 className="w-3.5 h-3.5 text-mercury-primary" />
              <span>Data Validation</span>
            </button>

            <button
              onClick={() => onExecuteCommand('remove_duplicates')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <span>Remove Duplicates</span>
            </button>
          </div>
        )}

        {/* ===================== WPS SMART TOOLBOX (SHEETS) ===================== */}
        {activeTab === 'toolbox' && (
          <div className="flex items-center space-x-2 text-xs overflow-x-auto py-0.5">
            <button
              onClick={() => onOpenSmartToolbox('merge_no_loss')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm bg-mercury-sheets text-white hover:bg-mercury-sheets/90 font-medium shadow-sm shrink-0"
              title="Merge cells concatenating data"
            >
              <Combine className="w-3.5 h-3.5" />
              <span>Merge No Loss</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('unmerge_fill')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
              title="Unmerge and autofill merged values into all cells"
            >
              <span>Unmerge & Fill</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('range_arithmetic')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
              title="Apply batch math (+, -, *, /) to all selected numbers"
            >
              <span>Range Arithmetic</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('add_prefix_suffix')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
              title="Batch prepend or append text across range"
            >
              <span>Prefix / Suffix</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('case_conversion')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
              title="Convert text to UPPERCASE, lowercase, or Proper Case"
            >
              <span>Case Conversion</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('regex_extract')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
              title="Extract numbers, letters, or regex patterns into target column"
            >
              <span>Regex Extract</span>
            </button>
            <button
              onClick={() => onExecuteCommand('convert_formulas_to_values')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
              title="Freeze calculated values permanently replacing formulas"
            >
              <span>Formulas &rarr; Values</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('split_sheet')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
            >
              <Split className="w-3.5 h-3.5 text-mercury-sheets" />
              <span>Split Sheet</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('compare_sheets')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
            >
              <Search className="w-3.5 h-3.5 text-mercury-sheets" />
              <span>Sheet Comparator</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('data_hygiene')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night shrink-0"
            >
              <Wand2 className="w-3.5 h-3.5 text-mercury-primary" />
              <span>Data Hygiene</span>
            </button>
          </div>
        )}

        {/* ===================== VIEW TAB ===================== */}
        {activeTab === 'view' && (
          <div className="flex items-center space-x-2 text-xs">
            {docType === 'slides' ? (
              <>
                <button
                  onClick={() => onExecuteCommand('open_presenter_view')}
                  className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm bg-mercury-slides text-white font-semibold shadow-sm hover:opacity-95 transition"
                >
                  <PlaySquare className="w-3.5 h-3.5" />
                  <span>Launch Presenter View (F5)</span>
                </button>
                <button
                  onClick={() => onExecuteCommand('start_slideshow_current')}
                  className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                >
                  <span>From Current Slide</span>
                </button>
                <button
                  onClick={() => onExecuteCommand('toggle_speaker_notes')}
                  className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                >
                  <span>Speaker Notes Pane</span>
                </button>
              </>
            ) : (
              <>
                <button
                  onClick={() => onExecuteCommand('toggle_reading_view')}
                  className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-sheets/10 text-mercury-sheets border border-mercury-sheets/30 hover:bg-mercury-sheets/20 font-medium"
                  title="Highlight active row and column headers with crosshair spotlight"
                >
                  <Eye className="w-3.5 h-3.5" />
                  <span>Reading View Spotlight</span>
                </button>

                <button
                  onClick={() => onExecuteCommand('freeze_panes')}
                  className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                >
                  <span>Freeze Panes</span>
                </button>

                <button
                  onClick={() => onExecuteCommand('freeze_top_row')}
                  className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                >
                  <span>Freeze Top Row</span>
                </button>

                <button
                  onClick={() => onExecuteCommand('freeze_first_col')}
                  className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                >
                  <span>Freeze First Column</span>
                </button>
              </>
            )}
          </div>
        )}

        {/* ===================== PDF ANNOTATE TAB ===================== */}
        {activeTab === 'pdf_annotate' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* Text Markup Group */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'highlight', color: '#ffeb3b' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm bg-yellow-400/20 text-yellow-700 dark:text-yellow-300 font-medium hover:opacity-90"
                title="Highlight (Yellow)"
              >
                <Highlighter className="w-3.5 h-3.5" />
                <span>Highlight</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'underline', color: '#5266eb' })}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-primary"
                title="Underline Text"
              >
                <Underline className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'strikethrough', color: '#e03131' })}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night text-red-600"
                title="Strikethrough"
              >
                <Strikethrough className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'squiggly', color: '#e8590c' })}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night text-amber-600"
                title="Squiggly Underline"
              >
                <span className="font-bold underline decoration-wavy text-xs">~</span>
              </button>
            </div>

            {/* Note, Free Text & Callout */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'sticky_note', text: 'Review comment', author: 'Reviewer' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Add Sticky Note"
              >
                <MessageSquare className="w-3.5 h-3.5 text-amber-500" />
                <span>Sticky Note</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'free_text', text: 'New Note', color: '#28253b' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Free Text Box"
              >
                <Type className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Free Text</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'callout', text: 'Action Item', color: '#e03131' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Callout Box with Leader Line"
              >
                <ArrowRight className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Callout Box</span>
              </button>
            </div>

            {/* Shapes & Cloud Polygon */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'shape', shape_type: 'Rectangle', color: '#e03131' })}
                className="p-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Rectangle Shape"
              >
                <span className="w-3 h-3 border border-mercury-pdf inline-block"></span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'shape', shape_type: 'Ellipse', color: '#e03131' })}
                className="p-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Ellipse Shape"
              >
                <span className="w-3 h-3 border border-mercury-pdf rounded-full inline-block"></span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'cloud_polygon', color: '#e03131' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                title="Cloud Polygon with Scalloped Arcs"
              >
                <Cloud className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Cloud</span>
              </button>
            </div>

            {/* Freehand Ink */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_freehand_ink')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Freehand Ink Drawing"
              >
                <PenTool className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Ink Pen</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_clear_ink')}
                className="p-1.5 rounded-mercury-sm hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-ink-secondary"
                title="Ink Eraser"
              >
                <Eraser className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Stamps Gallery */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              {[
                { label: 'Approved', color: '#0ca678' },
                { label: 'Confidential', color: '#e03131' },
                { label: 'Draft', color: '#f59f00' },
                { label: 'Rejected', color: '#fa5252' },
              ].map((st) => (
                <button
                  key={st.label}
                  onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'stamp', stamp_type: st.label })}
                  className="px-2 py-1 rounded-mercury-sm border text-[11px] font-bold uppercase transition"
                  style={{ borderColor: `${st.color}50`, color: st.color }}
                  title={`Apply ${st.label} Stamp`}
                >
                  {st.label}
                </button>
              ))}
            </div>

            {/* Precision Calibrated Measurement Tools */}
            <div className="flex items-center space-x-1">
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'measurement' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Calibrated Distance Ruler Tool"
              >
                <Ruler className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Distance Ruler</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'perimeter' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Perimeter Measurement Tool"
              >
                <span>Perimeter</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_annotation', { kind: 'area_polygon' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Shoelace Area Measurement Tool"
              >
                <span>Area Tool</span>
              </button>
            </div>
          </div>
        )}

        {/* ===================== PDF EDIT TAB ===================== */}
        {activeTab === 'pdf_edit' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* Inline Editing Toggle */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_toggle_edit_content')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm bg-mercury-pdf text-white font-semibold shadow-sm hover:opacity-95 transition"
                title="Direct Inline Paragraph & Line Reflow Editing"
              >
                <CheckCircle2 className="w-3.5 h-3.5" />
                <span>Edit PDF Like Word</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_add_text_block')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Add New Text Block"
              >
                <Plus className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Add Text</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_crop_page')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Crop Page Margins"
              >
                <Crop className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Crop Page</span>
              </button>
            </div>

            {/* Watermark Studio */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] font-medium text-mercury-ink-secondary mr-1">Watermark:</span>
              <button
                onClick={() => onExecuteCommand('pdf_apply_watermark', 'CONFIDENTIAL')}
                className="px-2.5 py-1.5 rounded-mercury-sm border border-red-500/30 text-red-600 hover:bg-red-500/10 font-medium"
              >
                CONFIDENTIAL
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_apply_watermark', 'DRAFT')}
                className="px-2.5 py-1.5 rounded-mercury-sm border border-amber-500/30 text-amber-600 hover:bg-amber-500/10 font-medium"
              >
                DRAFT
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_apply_watermark', 'COPY')}
                className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/40 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
              >
                COPY
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_remove_watermark')}
                className="px-2 py-1.5 text-mercury-ink-secondary hover:text-red-500"
                title="Remove Watermark"
              >
                Remove
              </button>
            </div>

            {/* Bates Numbering */}
            <div className="flex items-center space-x-1">
              <span className="text-[11px] font-medium text-mercury-ink-secondary mr-1">Bates Stamping:</span>
              <button
                onClick={() => onExecuteCommand('pdf_apply_bates', { prefix: 'CONF-LIT-', start: 1, digits: 6, position: 'BottomRight' })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-pdf text-mercury-pdf hover:bg-mercury-pdf/10 font-semibold"
                title="Apply Bates Numbering (CONF-LIT-000001)"
              >
                <FileCode2 className="w-3.5 h-3.5" />
                <span>Apply Bates Stamping</span>
              </button>
            </div>
          </div>
        )}

        {/* ===================== PDF ORGANIZE TAB ===================== */}
        {activeTab === 'pdf_organize' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* Page Manipulation Group */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_rotate_page', 90)}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Rotate Clockwise 90°"
              >
                <RotateCw className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Rotate 90° CW</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_rotate_page', -90)}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Rotate Counter-Clockwise 90°"
              >
                <RotateCcw className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Rotate 90° CCW</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_insert_blank_page')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Insert Blank Page"
              >
                <Plus className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Insert Blank</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_delete_page')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-red-500/30 text-red-600 hover:bg-red-500/10 font-medium"
                title="Delete Selected Page"
              >
                <Trash2 className="w-3.5 h-3.5" />
                <span>Delete Page</span>
              </button>
            </div>

            {/* Split & Merge */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_split_document', { mode: 'every_n', count: 1 })}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Split PDF into multiple files"
              >
                <Split className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Split PDF</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_merge_documents')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Merge external PDF into this document"
              >
                <Combine className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Merge PDFs</span>
              </button>
            </div>

            {/* Compress PDF */}
            <div className="flex items-center space-x-1">
              <span className="text-[11px] font-medium text-mercury-ink-secondary mr-1">Compress:</span>
              <button
                onClick={() => onExecuteCommand('pdf_compress', 'standard')}
                className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Standard Compression (Balanced Quality)"
              >
                Standard
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_compress', 'high')}
                className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="High Flate Stream Compression"
              >
                High
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_compress', 'extreme')}
                className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-pdf text-mercury-pdf hover:bg-mercury-pdf/10 font-semibold"
                title="Extreme Minification"
              >
                Extreme
              </button>
            </div>
          </div>
        )}

        {/* ===================== PDF CONVERT TAB ===================== */}
        {activeTab === 'pdf_convert' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* Pure-Rust Local OCR */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_ocr', 'searchable')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm bg-mercury-pdf text-white font-semibold shadow-sm hover:opacity-95 transition"
                title="Inject invisible 3 Tr searchable text layer over scanned PDF pages"
              >
                <FileCheck2 className="w-3.5 h-3.5" />
                <span>OCR Searchable Layer</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_ocr', 'editable')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Run OCR and reconstruct editable paragraphs"
              >
                <Type className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>OCR Editable Reconstruction</span>
              </button>
            </div>

            {/* Instant Cross-Engine AST Converters */}
            <div className="flex items-center space-x-1">
              <button
                onClick={() => onExecuteCommand('pdf_to_word')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm border border-mercury-writer/50 text-mercury-writer hover:bg-mercury-writer/10 font-bold transition shadow-xs"
                title="Convert PDF to live Writer document tab (.docx)"
              >
                <FileDown className="w-3.5 h-3.5" />
                <span>Convert to Word (.docx)</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_to_excel')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm border border-mercury-sheets/50 text-mercury-sheets hover:bg-mercury-sheets/10 font-bold transition shadow-xs"
                title="Extract tables and convert to live Spreadsheets tab (.xlsx)"
              >
                <FileSpreadsheet className="w-3.5 h-3.5" />
                <span>Convert to Excel (.xlsx)</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_to_ppt')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm border border-mercury-slides/50 text-mercury-slides hover:bg-mercury-slides/10 font-bold transition shadow-xs"
                title="Convert slides and vector scenes to live Presentation tab (.pptx)"
              >
                <Presentation className="w-3.5 h-3.5" />
                <span>Convert to PPT (.pptx)</span>
              </button>
            </div>
          </div>
        )}

        {/* ===================== REVIEW & TRACK CHANGES (PDF & GENERAL) ===================== */}
        {activeTab === 'review' && docType === 'pdf' ? (
          <div className="flex items-center space-x-3 text-xs">
            {/* Permanent Redaction Studio */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_redact_region')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm bg-red-600 text-white font-semibold shadow-sm hover:opacity-95 transition"
                title="Permanently remove sensitive bytes from stream with opaque overlay"
              >
                <Scissors className="w-3.5 h-3.5" />
                <span>Permanent Redact Area</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_open_sidebar', 'search_redact')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-red-500/30 text-red-600 hover:bg-red-500/10 font-medium"
                title="Search and Redact PII Across All Pages"
              >
                <Search className="w-3.5 h-3.5" />
                <span>Search & Redact PII</span>
              </button>
            </div>

            {/* Interactive Forms & Signatures */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('pdf_sign_document')}
                className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-pdf hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Digital Signature (FIPS 180-4 SHA-256)"
              >
                <FileSignature className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Sign Document</span>
              </button>
              <button
                onClick={() => onExecuteCommand('pdf_flatten_forms')}
                className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                title="Flatten Interactive AcroForms into static vectors"
              >
                <CheckCircle2 className="w-3.5 h-3.5 text-mercury-pdf" />
                <span>Flatten Forms</span>
              </button>
            </div>

            {/* AES-256 Security & Encryption */}
            <div className="flex items-center space-x-1">
              <button
                onClick={() => onExecuteCommand('pdf_encrypt_document')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm border border-mercury-pdf text-mercury-pdf hover:bg-mercury-pdf/10 font-semibold transition"
                title="Encrypt PDF with FIPS 197 AES-256 Standard"
              >
                <Lock className="w-3.5 h-3.5" />
                <span>AES-256 Encrypt</span>
              </button>
            </div>
          </div>
        ) : activeTab === 'review' && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onExecuteCommand('toggle_track_changes')}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-writer text-white font-medium shadow-sm hover:bg-mercury-writer/90"
            >
              <span>Track Changes</span>
            </button>

            <select
              onChange={(e) => onExecuteCommand('set_markup_mode', e.target.value)}
              className="h-7 px-2 border border-mercury-fog/40 rounded-mercury-sm text-xs bg-white dark:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark outline-none focus:border-mercury-primary"
              defaultValue="AllMarkup"
            >
              <option value="AllMarkup">All Markup (Inline & Margin)</option>
              <option value="SimpleMarkup">Simple Markup</option>
              <option value="NoMarkup">No Markup (Clean Preview)</option>
              <option value="Original">Original</option>
            </select>

            <button
              onClick={() => onExecuteCommand('accept_all_revisions')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-green-500/40 text-green-700 dark:text-green-300 hover:bg-green-500/10 font-medium"
            >
              Accept All Revisions
            </button>

            <button
              onClick={() => onExecuteCommand('reject_all_revisions')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-red-500/40 text-red-600 dark:text-red-400 hover:bg-red-500/10 font-medium"
            >
              Reject All Revisions
            </button>

            <button
              onClick={() => onExecuteCommand('new_comment')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Plus className="w-3.5 h-3.5 text-mercury-primary" />
              <span>+ New Comment</span>
            </button>
          </div>
        )}

        {/* ===================== SLIDES DESIGN & MASTER TAB ===================== */}
        {activeTab === 'slides' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* 8 Mercury Themes Gallery */}
            <div className="flex items-center space-x-1.5 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] font-medium text-mercury-ink-secondary mr-1">Mercury Themes:</span>
              {[
                { name: 'Modern', color: '#5266eb' },
                { name: 'Slate', color: '#495057' },
                { name: 'Emerald', color: '#0ca678' },
                { name: 'Coral', color: '#e8590c' },
                { name: 'Crimson', color: '#e03131' },
                { name: 'Iris', color: '#7048e8' },
                { name: 'Cyber', color: '#00f2fe' },
                { name: 'Sand', color: '#f59f00' },
              ].map((theme) => (
                <button
                  key={theme.name}
                  onClick={() => onExecuteCommand('apply_theme', theme.name)}
                  className="flex items-center gap-1.5 px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-slides hover:bg-mercury-lavender dark:hover:bg-mercury-night transition"
                  title={`Apply ${theme.name} Theme`}
                >
                  <span className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: theme.color }}></span>
                  <span className="font-medium text-[11px]">{theme.name}</span>
                </button>
              ))}
            </div>

            {/* Slide Size Switcher */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] font-medium text-mercury-ink-secondary mr-1">Size:</span>
              <button
                onClick={() => onExecuteCommand('set_slide_size', '16:9')}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-slides font-medium"
              >
                16:9 Wide
              </button>
              <button
                onClick={() => onExecuteCommand('set_slide_size', '4:3')}
                className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-slides font-medium"
              >
                4:3 Standard
              </button>
            </div>

            {/* Master Mode */}
            <button
              onClick={() => onExecuteCommand('toggle_master_mode')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Layers className="w-3.5 h-3.5 text-mercury-slides" />
              <span>Slide Master View</span>
            </button>
          </div>
        )}

        {/* ===================== SLIDES TRANSITIONS TAB ===================== */}
        {activeTab === 'transitions' && (
          <div className="flex items-center space-x-3 text-xs">
            {/* Transition Gallery */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <button
                onClick={() => onExecuteCommand('apply_morph_transition')}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm bg-mercury-slides text-white font-semibold shadow-sm hover:opacity-95 transition"
              >
                <Sparkles className="w-3.5 h-3.5" />
                <span>Morph (Flagship)</span>
              </button>
              {['Fade', 'Push', 'Wipe', 'Split', 'Zoom', 'None'].map((trans) => (
                <button
                  key={trans}
                  onClick={() => onExecuteCommand('set_slide_transition', trans)}
                  className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-slides hover:bg-mercury-lavender dark:hover:bg-mercury-night font-medium"
                >
                  {trans}
                </button>
              ))}
            </div>

            {/* Duration Selector */}
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] text-mercury-ink-secondary mr-1">Duration:</span>
              {['0.5s', '1.0s', '1.5s', '2.0s'].map((dur) => (
                <button
                  key={dur}
                  onClick={() => onExecuteCommand('set_transition_duration', parseFloat(dur) * 1000)}
                  className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-slides"
                >
                  {dur}
                </button>
              ))}
            </div>

            {/* Live Preview Button */}
            <button
              onClick={() => onExecuteCommand('preview_morph_transition')}
              className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-sm border border-mercury-slides text-mercury-slides hover:bg-mercury-slides/10 font-semibold transition"
            >
              <PlaySquare className="w-3.5 h-3.5" />
              <span>Preview Transition</span>
            </button>
          </div>
        )}

        {/* ===================== SLIDES ANIMATIONS TAB ===================== */}
        {activeTab === 'animations' && (
          <div className="flex items-center space-x-3 text-xs">
            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] font-medium text-mercury-ink-secondary mr-1">Entrance:</span>
              {['Fade In', 'Fly In', 'Zoom In', 'Bounce', 'Spin'].map((anim) => (
                <button
                  key={anim}
                  onClick={() => onExecuteCommand('add_animation', { category: 'entrance', effect: anim })}
                  className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-slides hover:bg-mercury-lavender dark:hover:bg-mercury-night"
                >
                  {anim}
                </button>
              ))}
            </div>

            <div className="flex items-center space-x-1 pr-3 border-r border-mercury-fog/30">
              <span className="text-[11px] font-medium text-mercury-ink-secondary mr-1">Emphasis:</span>
              {['Pulse', 'Spin 360', 'Teeter'].map((anim) => (
                <button
                  key={anim}
                  onClick={() => onExecuteCommand('add_animation', { category: 'emphasis', effect: anim })}
                  className="px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:border-mercury-slides"
                >
                  {anim}
                </button>
              ))}
            </div>

            <button
              onClick={() => onExecuteCommand('toggle_animation_pane')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <SlidersHorizontal className="w-3.5 h-3.5 text-mercury-slides" />
              <span>Animation Pane</span>
            </button>
          </div>
        )}

        {/* ===================== WPS AI TAB ===================== */}
        {activeTab === 'ai' && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onExecuteCommand('ai_rewrite')}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-ai text-white font-medium shadow-sm"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Rewrite & Polish</span>
            </button>
            <button
              onClick={() => onExecuteCommand('ai_summarize')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Summarize Section
            </button>
            <button
              onClick={() => onExecuteCommand('ai_generate_formula')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Natural Language &rarr; Formula
            </button>
            <button
              onClick={() => onExecuteCommand('ai_deck_generator')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Document &rarr; Slide Deck
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
