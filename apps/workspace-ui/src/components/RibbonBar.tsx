import React from 'react';
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
  FileDown
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
  // Contextual Tabs per Document Type
  const getTabsForDocType = (): { id: ActiveRibbonTab; label: string }[] => {
    switch (docType) {
      case 'writer':
        return [
          { id: 'home', label: 'Home' },
          { id: 'insert', label: 'Insert' },
          { id: 'layout', label: 'Page Layout' },
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

        {/* ===================== WPS SMART TOOLBOX (SHEETS) ===================== */}
        {activeTab === 'toolbox' && (
          <div className="flex items-center space-x-2 text-xs">
            <button
              onClick={() => onOpenSmartToolbox('merge_no_loss')}
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-sheets text-white hover:bg-mercury-sheets/90 font-medium shadow-sm"
            >
              <Combine className="w-3.5 h-3.5" />
              <span>Merge Without Data Loss</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('split_sheet')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Split className="w-3.5 h-3.5 text-mercury-sheets" />
              <span>Split Sheet Wizard</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('combine_sheets')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Layers className="w-3.5 h-3.5 text-mercury-sheets" />
              <span>Combine Workbooks</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('compare_sheets')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Search className="w-3.5 h-3.5 text-mercury-sheets" />
              <span>Sheet Comparator</span>
            </button>
            <button
              onClick={() => onOpenSmartToolbox('data_hygiene')}
              className="flex items-center space-x-1 px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              <Wand2 className="w-3.5 h-3.5 text-mercury-primary" />
              <span>Data Hygiene Tools</span>
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
              className="flex items-center space-x-1 px-3 py-1.5 rounded-mercury-sm bg-mercury-writer text-white font-medium"
            >
              <span>Track Changes</span>
            </button>
            <button
              onClick={() => onExecuteCommand('accept_all_revisions')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Accept All
            </button>
            <button
              onClick={() => onExecuteCommand('reject_all_revisions')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              Reject All
            </button>
            <button
              onClick={() => onExecuteCommand('new_comment')}
              className="px-2.5 py-1.5 rounded-mercury-sm border border-mercury-fog/30 hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            >
              + New Comment
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
