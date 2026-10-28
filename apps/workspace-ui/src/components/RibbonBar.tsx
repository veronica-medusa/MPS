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
        {activeTab === 'home' && (
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
          </div>
        )}

        {/* ===================== PDF TOOLS ===================== */}
        {(activeTab === 'pdf_annotate' || activeTab === 'pdf_edit' || activeTab === 'pdf_organize' || activeTab === 'pdf_convert') && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onExecuteCommand('pdf_highlight')}
              className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm bg-yellow-400/20 text-yellow-700 dark:text-yellow-300 font-medium"
            >
              <Highlighter className="w-3.5 h-3.5" />
              <span>Highlight</span>
            </button>
            <button
              onClick={() => onExecuteCommand('pdf_freehand_ink')}
              className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <PenTool className="w-3.5 h-3.5 text-mercury-pdf" />
              <span>Freehand Ink</span>
            </button>
            <button
              onClick={() => onExecuteCommand('pdf_direct_edit')}
              className="flex items-center space-x-1 px-3 py-1 rounded-mercury-sm bg-mercury-pdf text-white font-medium shadow-sm"
            >
              <CheckCircle2 className="w-3.5 h-3.5" />
              <span>Edit PDF Like Word</span>
            </button>
            <button
              onClick={() => onExecuteCommand('pdf_ocr')}
              className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <FileCheck2 className="w-3.5 h-3.5 text-mercury-pdf" />
              <span>Local OCR (ocrs)</span>
            </button>
            <button
              onClick={() => onExecuteCommand('pdf_to_word')}
              className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <FileDown className="w-3.5 h-3.5 text-mercury-writer" />
              <span>PDF &rarr; Word (.docx)</span>
            </button>
            <button
              onClick={() => onExecuteCommand('pdf_redact')}
              className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm border border-red-500/30 text-red-600 hover:bg-red-500/10"
            >
              <Scissors className="w-3.5 h-3.5" />
              <span>Redact</span>
            </button>
          </div>
        )}

        {/* ===================== REVIEW & TRACK CHANGES ===================== */}
        {activeTab === 'review' && (
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

        {/* ===================== SLIDES ANIMATIONS & MORPH ===================== */}
        {(activeTab === 'transitions' || activeTab === 'animations') && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onExecuteCommand('apply_morph_transition')}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-slides text-white font-medium shadow-sm"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Morph Transition</span>
            </button>
            <button
              onClick={() => onExecuteCommand('add_animation', 'fade_in')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Entrance: Fade In
            </button>
            <button
              onClick={() => onExecuteCommand('open_presenter_view')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <PlaySquare className="w-3.5 h-3.5 text-mercury-slides" />
              <span>Dual-Screen Presenter View</span>
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
