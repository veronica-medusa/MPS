import React, { useState, useRef, useEffect } from 'react';
import {
  Home,
  Plus,
  X,
  Sparkles,
  History,
  Download,
  FileText,
  Table,
  Presentation,
  FileCheck,
  FolderOpen,
  ChevronDown
} from 'lucide-react';
import { OfficeTab, DocumentType } from '../types';

interface TopTabBarProps {
  tabs: OfficeTab[];
  activeTabId: string;
  isHomeActive: boolean;
  onSelectTab: (id: string) => void;
  onSelectHome: () => void;
  onCloseTab: (id: string) => void;
  onRenameTab: (id: string, newTitle: string) => void;
  onNewDocument: (type: DocumentType) => void;
  onOpenFile: () => void;
  isAiOpen: boolean;
  onToggleAi: () => void;
  isHistoryOpen: boolean;
  onToggleHistory: () => void;
  onExportFile: (format: string) => void;
}

export const TopTabBar: React.FC<TopTabBarProps> = ({
  tabs,
  activeTabId,
  isHomeActive,
  onSelectTab,
  onSelectHome,
  onCloseTab,
  onRenameTab,
  onNewDocument,
  onOpenFile,
  isAiOpen,
  onToggleAi,
  isHistoryOpen,
  onToggleHistory,
  onExportFile,
}) => {
  const [showNewMenu, setShowNewMenu] = useState(false);
  const [showExportMenu, setShowExportMenu] = useState(false);
  const [editingTabId, setEditingTabId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState('');
  const newMenuRef = useRef<HTMLDivElement>(null);
  const exportMenuRef = useRef<HTMLDivElement>(null);

  // Close popovers on click outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (newMenuRef.current && !newMenuRef.current.contains(e.target as Node)) {
        setShowNewMenu(false);
      }
      if (exportMenuRef.current && !exportMenuRef.current.contains(e.target as Node)) {
        setShowExportMenu(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleStartRename = (id: string, currentTitle: string) => {
    setEditingTabId(id);
    setEditingTitle(currentTitle);
  };

  const handleFinishRename = (id: string) => {
    if (editingTitle.trim()) {
      onRenameTab(id, editingTitle.trim());
    }
    setEditingTabId(null);
  };

  const getBadge = (type: DocumentType) => {
    switch (type) {
      case 'writer':
        return <span className="wps-doc-tab-badge wps-badge-writer">W</span>;
      case 'sheets':
        return <span className="wps-doc-tab-badge wps-badge-sheets">S</span>;
      case 'slides':
        return <span className="wps-doc-tab-badge wps-badge-slides">P</span>;
      case 'pdf':
        return <span className="wps-doc-tab-badge wps-badge-pdf">PDF</span>;
    }
  };

  return (
    <header className="wps-top-bar">
      {/* Left: Home Button */}
      <button
        onClick={onSelectHome}
        className={`wps-home-btn ${isHomeActive ? 'is-active' : ''}`}
        title="Open WPS Home Hub"
      >
        <Home className="w-4 h-4" />
        <span>Home</span>
      </button>

      {/* Center-Left: Scrollable Document Tab Strip */}
      <div className="wps-doc-tabs-strip">
        {tabs.map((tab) => {
          const isActive = !isHomeActive && tab.id === activeTabId;
          const isEditing = editingTabId === tab.id;

          return (
            <div
              key={tab.id}
              onClick={() => onSelectTab(tab.id)}
              onDoubleClick={() => handleStartRename(tab.id, tab.title)}
              className={`wps-doc-tab ${isActive ? 'is-active' : ''}`}
              title={`${tab.title} (Double-click to rename)`}
            >
              {getBadge(tab.type)}

              {isEditing ? (
                <input
                  type="text"
                  value={editingTitle}
                  onChange={(e) => setEditingTitle(e.target.value)}
                  onBlur={() => handleFinishRename(tab.id)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleFinishRename(tab.id);
                    if (e.key === 'Escape') setEditingTabId(null);
                  }}
                  autoFocus
                  className="bg-transparent border-b border-mercury-primary text-xs outline-none px-1 text-inherit"
                  onClick={(e) => e.stopPropagation()}
                />
              ) : (
                <span className="truncate max-w-[150px]">{tab.title}</span>
              )}

              {tab.isDirty && <span className="wps-dirty-dot" title="Unsaved changes" />}

              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onCloseTab(tab.id);
                }}
                className="wps-tab-close"
                title="Close Tab"
              >
                <X className="w-3 h-3" />
              </button>
            </div>
          );
        })}

        {/* Plus Button with Dropdown Menu */}
        <div className="relative" ref={newMenuRef}>
          <button
            onClick={() => setShowNewMenu(!showNewMenu)}
            className="wps-add-tab-btn"
            title="New Document or Open File"
          >
            <Plus className="w-4 h-4" />
          </button>

          {showNewMenu && (
            <div className="absolute left-0 top-full mt-1.5 w-60 bg-white dark:bg-mercury-night-raised border border-mercury-fog/40 rounded-mercury-md shadow-lg py-1.5 z-50 text-xs">
              <button
                onClick={() => {
                  onNewDocument('writer');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-2 text-left flex items-center gap-2.5 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <span className="wps-doc-tab-badge wps-badge-writer">W</span>
                <div>
                  <div className="font-medium">New Document (.docx)</div>
                  <div className="text-[10px] text-mercury-slate">WPS Writer</div>
                </div>
              </button>

              <button
                onClick={() => {
                  onNewDocument('sheets');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-2 text-left flex items-center gap-2.5 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <span className="wps-doc-tab-badge wps-badge-sheets">S</span>
                <div>
                  <div className="font-medium">New Spreadsheet (.xlsx)</div>
                  <div className="text-[10px] text-mercury-slate">WPS Spreadsheets</div>
                </div>
              </button>

              <button
                onClick={() => {
                  onNewDocument('slides');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-2 text-left flex items-center gap-2.5 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <span className="wps-doc-tab-badge wps-badge-slides">P</span>
                <div>
                  <div className="font-medium">New Presentation (.pptx)</div>
                  <div className="text-[10px] text-mercury-slate">WPS Presentation</div>
                </div>
              </button>

              <button
                onClick={() => {
                  onNewDocument('pdf');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-2 text-left flex items-center gap-2.5 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <span className="wps-doc-tab-badge wps-badge-pdf">PDF</span>
                <div>
                  <div className="font-medium">Open / Convert PDF (.pdf)</div>
                  <div className="text-[10px] text-mercury-slate">WPS PDF Toolkit</div>
                </div>
              </button>

              <div className="border-t border-mercury-fog/30 my-1" />

              <button
                onClick={() => {
                  onOpenFile();
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-2 text-left flex items-center gap-2.5 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <FolderOpen className="w-4 h-4 text-mercury-slate ml-0.5" />
                <span className="font-medium">Open Local File...</span>
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Right Action Group */}
      <div className="flex items-center gap-2 flex-shrink-0">
        {/* Version History Button */}
        <button
          onClick={onToggleHistory}
          className={`button-glass-dark h-8 px-3 text-xs flex items-center gap-1.5 ${
            isHistoryOpen ? 'bg-mercury-glass-dark-border text-white' : ''
          }`}
          title="Toggle Version History"
        >
          <History className="w-3.5 h-3.5" />
          <span className="hidden sm:inline">Version History</span>
        </button>

        {/* WPS AI Button */}
        <button
          onClick={onToggleAi}
          className={`button-tint-dark h-8 px-3 text-xs flex items-center gap-1.5 ${
            isAiOpen ? 'bg-mercury-primary text-white border-transparent' : ''
          }`}
          title="Toggle WPS AI Assistant"
        >
          <Sparkles className="w-3.5 h-3.5 text-amber-300" />
          <span>WPS AI</span>
        </button>

        {/* Export / Save Primary CTA */}
        <div className="relative" ref={exportMenuRef}>
          <button
            onClick={() => setShowExportMenu(!showExportMenu)}
            className="button-primary h-8 px-3.5 text-xs flex items-center gap-1.5"
            title="Save and Export Current File"
          >
            <Download className="w-3.5 h-3.5" />
            <span>Export / Save</span>
            <ChevronDown className="w-3 h-3 opacity-70" />
          </button>

          {showExportMenu && (
            <div className="absolute right-0 top-full mt-1.5 w-48 bg-white dark:bg-mercury-night-raised border border-mercury-fog/40 rounded-mercury-md shadow-lg py-1.5 z-50 text-xs">
              <div className="px-3 py-1 text-[10px] font-semibold text-mercury-slate uppercase tracking-wider">
                Download Active File
              </div>
              <button
                onClick={() => {
                  onExportFile('docx');
                  setShowExportMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <FileText className="w-3.5 h-3.5 text-mercury-primary" />
                <span>Word Document (.docx)</span>
              </button>
              <button
                onClick={() => {
                  onExportFile('xlsx');
                  setShowExportMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <Table className="w-3.5 h-3.5 text-emerald-600" />
                <span>Excel Workbook (.xlsx)</span>
              </button>
              <button
                onClick={() => {
                  onExportFile('pptx');
                  setShowExportMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <Presentation className="w-3.5 h-3.5 text-orange-600" />
                <span>PowerPoint (.pptx)</span>
              </button>
              <button
                onClick={() => {
                  onExportFile('pdf');
                  setShowExportMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <FileCheck className="w-3.5 h-3.5 text-rose-600" />
                <span>PDF Document (.pdf)</span>
              </button>
              <div className="border-t border-mercury-fog/30 my-1" />
              <button
                onClick={() => {
                  onExportFile('csv');
                  setShowExportMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <span>Comma Separated (.csv)</span>
              </button>
              <button
                onClick={() => {
                  onExportFile('txt');
                  setShowExportMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center gap-2 hover:bg-mercury-lavender/40 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <span>Plain Text (.txt)</span>
              </button>
            </div>
          )}
        </div>
      </div>
    </header>
  );
};
