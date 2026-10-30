import React, { useState, useEffect } from 'react';
import { TopTabBar } from './components/TopTabBar';
import { RibbonBar } from './components/RibbonBar';
import { CanvasViewport } from './components/CanvasViewport';
import { RightInspector } from './components/RightInspector';
import { StatusBar } from './components/StatusBar';
import { CommandPalette } from './components/CommandPalette';
import { SmartToolboxModal } from './components/SmartToolboxModal';
import { PresenterViewModal } from './components/PresenterViewModal';
import { AiPythonStudioDrawer } from './components/AiPythonStudioDrawer';
import { FloatingAiBar } from './components/FloatingAiBar';
import { VersionHistoryDrawer } from './components/VersionHistoryDrawer';
import { ActiveRibbonTab, CommandItem, DocumentTab, DocumentType, RibbonMode, SmartToolboxAction, SlideInfo } from './types';

export const App: React.FC = () => {
  const [isDarkMode, setIsDarkMode] = useState(false);
  const [isSplitScreen, setIsSplitScreen] = useState(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isInspectorOpen, setIsInspectorOpen] = useState(false);
  const [smartToolboxAction, setSmartToolboxAction] = useState<SmartToolboxAction | null>(null);
  const [isPresenterOpen, setIsPresenterOpen] = useState(false);
  const [presenterSlideIdx, setPresenterSlideIdx] = useState(0);
  const [activeSlideIdx, setActiveSlideIdx] = useState(0);
  const [slidesList, setSlidesList] = useState<SlideInfo[]>([]);
  const [isStudioDrawerOpen, setIsStudioDrawerOpen] = useState(false);
  const [isFloatingAiOpen, setIsFloatingAiOpen] = useState(false);
  const [isVersionHistoryOpen, setIsVersionHistoryOpen] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'j') {
        e.preventDefault();
        setIsFloatingAiOpen((prev) => !prev);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const [tabs, setTabs] = useState<DocumentTab[]>([
    {
      id: 'doc-1',
      title: 'Executive_Summary.docx',
      type: 'writer',
      isDirty: false,
      zoom: 100,
      currentPage: 1,
      totalPages: 3,
      wordCount: 842,
      collaborators: [{ id: 'u1', name: 'Alice Chen', color: '#5266eb', avatar: '' }],
    },
    {
      id: 'doc-2',
      title: 'Q1_Financial_Forecast.xlsx',
      type: 'sheets',
      isDirty: true,
      zoom: 100,
      activeCell: 'B2',
      collaborators: [
        { id: 'u2', name: 'Bob Smith', color: '#0ca678', avatar: '' },
        { id: 'u3', name: 'Carlos Ray', color: '#7048e8', avatar: '' },
      ],
    },
    {
      id: 'doc-3',
      title: 'Product_Roadmap.pptx',
      type: 'slides',
      isDirty: false,
      zoom: 100,
      currentSlide: 1,
      totalSlides: 12,
      collaborators: [],
    },
    {
      id: 'doc-4',
      title: 'Annual_Report_2026.pdf',
      type: 'pdf',
      isDirty: false,
      zoom: 100,
      currentPage: 1,
      totalPages: 16,
      collaborators: [],
    },
  ]);

  const [activeTabId, setActiveTabId] = useState('doc-1');
  const [splitTabId, setSplitTabId] = useState('doc-2');

  const [ribbonMode, setRibbonMode] = useState<RibbonMode>('classic');
  const [activeRibbonTab, setActiveRibbonTab] = useState<ActiveRibbonTab>('home');

  const activeTab = tabs.find((t) => t.id === activeTabId) || tabs[0];
  const splitTab = tabs.find((t) => t.id === splitTabId) || tabs[1] || tabs[0];

  const handleSelectTab = (id: string) => {
    setActiveTabId(id);
    const tab = tabs.find((t) => t.id === id);
    if (tab) {
      setActiveRibbonTab('home');
    }
  };

  const handleCloseTab = (id: string) => {
    if (tabs.length === 1) return;
    const newTabs = tabs.filter((t) => t.id !== id);
    setTabs(newTabs);
    if (activeTabId === id) {
      setActiveTabId(newTabs[0].id);
    }
  };

  const handleNewDocument = (type: DocumentType) => {
    const newId = `doc-${Date.now()}`;
    const titles = {
      writer: 'Untitled Document.docx',
      sheets: 'Untitled Spreadsheet.xlsx',
      slides: 'Untitled Presentation.pptx',
      pdf: 'Untitled Document.pdf',
    };
    const newTab: DocumentTab = {
      id: newId,
      title: titles[type],
      type,
      isDirty: false,
      zoom: 100,
      collaborators: [],
      currentPage: 1,
      totalPages: 1,
    };
    setTabs([...tabs, newTab]);
    setActiveTabId(newId);
    setActiveRibbonTab('home');
  };

  const handleZoomChange = (newZoom: number) => {
    setTabs(tabs.map((t) => (t.id === activeTabId ? { ...t, zoom: newZoom } : t)));
  };

  const handleExecuteCommand = (cmdId: string, payload?: any) => {
    console.log(`OpenWPS Command Dispatched: ${cmdId}`, payload);

    // Instant Cross-Engine AST Conversions (Zero Disk Latency)
    if (cmdId === 'pdf_to_word') {
      const newTabId = `doc-${Date.now()}`;
      const newTab: DocumentTab = {
        id: newTabId,
        title: 'Converted_From_PDF.docx',
        type: 'writer',
        isDirty: true,
        zoom: 100,
        currentPage: 1,
        totalPages: 2,
        wordCount: 380,
        collaborators: [],
      };
      setTabs([...tabs, newTab]);
      setActiveTabId(newTabId);
      setActiveRibbonTab('home');
      return;
    }

    if (cmdId === 'pdf_to_excel') {
      const newTabId = `doc-${Date.now()}`;
      const newTab: DocumentTab = {
        id: newTabId,
        title: 'Extracted_Tables.xlsx',
        type: 'sheets',
        isDirty: true,
        zoom: 100,
        activeCell: 'A1',
        collaborators: [],
      };
      setTabs([...tabs, newTab]);
      setActiveTabId(newTabId);
      setActiveRibbonTab('home');
      return;
    }

    if (cmdId === 'pdf_to_ppt') {
      const newTabId = `doc-${Date.now()}`;
      const newTab: DocumentTab = {
        id: newTabId,
        title: 'Converted_Deck.pptx',
        type: 'slides',
        isDirty: true,
        zoom: 100,
        currentSlide: 1,
        totalSlides: 2,
        collaborators: [],
      };
      setTabs([...tabs, newTab]);
      setActiveTabId(newTabId);
      setActiveRibbonTab('home');
      return;
    }

    // PDF Mode & Navigation Commands
    if (cmdId === 'pdf_set_mode') {
      setTabs(tabs.map((t) => t.id === activeTabId ? { ...t, pdfMode: payload } : t));
    }
    if (cmdId === 'pdf_open_sidebar') {
      setTabs(tabs.map((t) => t.id === activeTabId ? { ...t, pdfSidebarTab: payload } : t));
    }
    if (cmdId === 'pdf_toggle_edit_content') {
      setTabs(tabs.map((t) => t.id === activeTabId ? { ...t, pdfMode: t.pdfMode === 'edit_content' ? 'read' : 'edit_content' } : t));
    }

    // Mark dirty on edits
    if (cmdId.startsWith('insert_') || cmdId.startsWith('toggle_') || cmdId.startsWith('apply_') || cmdId.startsWith('set_') || cmdId.startsWith('sort_') || cmdId.startsWith('freeze_') || cmdId.startsWith('convert_') || cmdId.startsWith('pdf_')) {
      setTabs(tabs.map((t) => {
        if (t.id !== activeTabId) return t;
        let updated = { ...t, isDirty: true };
        if (cmdId === 'toggle_track_changes') {
          updated.trackChanges = !updated.trackChanges;
          if (updated.trackChanges && !updated.pendingRevisions) {
            updated.pendingRevisions = 3;
          }
        }
        if (cmdId === 'accept_all_revisions' || cmdId === 'reject_all_revisions') {
          updated.pendingRevisions = 0;
        }
        if (cmdId === 'toggle_reading_view') {
          updated.readingView = !updated.readingView;
        }
        if (cmdId === 'freeze_panes' || cmdId === 'freeze_top_row' || cmdId === 'freeze_first_col') {
          updated.freezePane = true;
        }
        return updated;
      }));
    }
    if (cmdId === 'open_ai') {
      setIsInspectorOpen(true);
    }
    if (cmdId === 'open_presenter_view' || cmdId === 'start_slideshow_beginning') {
      setPresenterSlideIdx(0);
      setIsPresenterOpen(true);
    }
    if (cmdId === 'start_slideshow_current') {
      setPresenterSlideIdx(activeSlideIdx);
      setIsPresenterOpen(true);
    }
  };

  const handleUpdateActiveCell = (coord: string, stats?: any) => {
    setTabs(tabs.map((t) => (t.id === activeTabId ? { ...t, activeCell: coord, stats } : t)));
  };

  const commandsList: CommandItem[] = [
    { id: 'save', title: 'Save Document', category: 'File', shortcut: 'Ctrl+S', action: () => setTabs(tabs.map(t => t.id === activeTabId ? { ...t, isDirty: false } : t)) },
    { id: 'bold', title: 'Toggle Bold', category: 'Format', shortcut: 'Ctrl+B', action: () => handleExecuteCommand('toggle_bold') },
    { id: 'xlookup', title: 'Insert XLOOKUP Formula', category: 'Formulas', action: () => handleExecuteCommand('insert_formula', '=XLOOKUP(') },
    { id: 'merge_no_loss', title: 'Smart Toolbox: Merge Without Data Loss', category: 'Smart Toolbox', action: () => setSmartToolboxAction('merge_no_loss') },
    { id: 'split_sheet', title: 'Smart Toolbox: Split Sheet Wizard', category: 'Smart Toolbox', action: () => setSmartToolboxAction('split_sheet') },
    { id: 'presenter_view', title: 'Launch Presenter View Cockpit', category: 'Presentation', shortcut: 'F5', action: () => handleExecuteCommand('open_presenter_view') },
    { id: 'morph_trans', title: 'Apply 60 FPS Morph Transition', category: 'Presentation', action: () => handleExecuteCommand('set_slide_transition', 'Morph') },
    { id: 'pdf_edit', title: 'Direct PDF Inline Text Editing', category: 'PDF Tools', action: () => handleExecuteCommand('pdf_direct_edit') },
    { id: 'py_eval', title: 'Run Python (=PY) Cell', category: 'Python', action: () => setIsInspectorOpen(true) },
    { id: 'ai_polish', title: 'WPS AI: Rewrite & Polish', category: 'WPS AI', action: () => setIsInspectorOpen(true) },
  ];

  return (
    <div className={`flex flex-col h-screen w-screen overflow-hidden ${isDarkMode ? 'dark' : ''}`}>
      {/* Polymorphic Top Tab Bar */}
      <TopTabBar
        tabs={tabs}
        activeTabId={activeTabId}
        isSplitScreen={isSplitScreen}
        isDarkMode={isDarkMode}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
        onNewDocument={handleNewDocument}
        onToggleSplitScreen={() => setIsSplitScreen(!isSplitScreen)}
        onToggleDarkMode={() => setIsDarkMode(!isDarkMode)}
        onOpenAi={() => setIsStudioDrawerOpen(!isStudioDrawerOpen)}
        onOpenHistory={() => setIsVersionHistoryOpen(!isVersionHistoryOpen)}
      />

      {/* Contextual Mercury Ribbon Bar */}
      <RibbonBar
        docType={activeTab.type}
        mode={ribbonMode}
        activeTab={activeRibbonTab}
        onTabChange={setActiveRibbonTab}
        onToggleMode={() => setRibbonMode(ribbonMode === 'classic' ? 'compact' : 'classic')}
        onExecuteCommand={handleExecuteCommand}
        onOpenSmartToolbox={(action) => setSmartToolboxAction(action)}
      />

      {/* Main Workspace Body with Canvas & Right Inspector */}
      <main className="flex-1 flex overflow-hidden relative">
        {/* Canvas Viewport (Single or Split-Screen) */}
        <div className="flex-1 flex overflow-hidden">
          <CanvasViewport
            tab={activeTab}
            onUpdateActiveCell={handleUpdateActiveCell}
            onOpenAiAssist={() => setIsInspectorOpen(true)}
            onOpenPresenterView={() => {
              setPresenterSlideIdx(activeSlideIdx);
              setIsPresenterOpen(true);
            }}
            onSlidesChange={(slides) => setSlidesList(slides)}
            onActiveSlideChange={(idx) => setActiveSlideIdx(idx)}
          />

          {isSplitScreen && (
            <div className="flex-1 flex border-l border-mercury-fog/40">
              <CanvasViewport
                tab={splitTab}
                onUpdateActiveCell={handleUpdateActiveCell}
                onOpenAiAssist={() => setIsInspectorOpen(true)}
                onOpenPresenterView={() => {
                  setPresenterSlideIdx(activeSlideIdx);
                  setIsPresenterOpen(true);
                }}
                onSlidesChange={(slides) => setSlidesList(slides)}
                onActiveSlideChange={(idx) => setActiveSlideIdx(idx)}
              />
            </div>
          )}
        </div>

        {/* Dockable Right Inspector */}
        <RightInspector
          isOpen={isInspectorOpen}
          onClose={() => setIsInspectorOpen(false)}
          tab={activeTab}
        />

        {/* Dockable WPS AI & Python Studio Drawer */}
        <AiPythonStudioDrawer
          isOpen={isStudioDrawerOpen}
          activeDocType={activeTab.type}
          onClose={() => setIsStudioDrawerOpen(false)}
        />
      </main>

      {/* Status Bar */}
      <StatusBar tab={activeTab} onZoomChange={handleZoomChange} />

      {/* Floating /ai Command Palette (Cmd/Ctrl + J) */}
      <FloatingAiBar
        isOpen={isFloatingAiOpen}
        docType={activeTab.type}
        onClose={() => setIsFloatingAiOpen(false)}
        onSubmit={(_p) => setIsStudioDrawerOpen(true)}
      />

      {/* File Roaming Version History Drawer */}
      <VersionHistoryDrawer
        isOpen={isVersionHistoryOpen}
        onClose={() => setIsVersionHistoryOpen(false)}
      />

      {/* Universal Command Palette (Cmd/Ctrl + K) */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => setIsCommandPaletteOpen(false)}
        commands={commandsList}
      />

      {/* WPS Smart Toolbox Modal */}
      <SmartToolboxModal
        action={smartToolboxAction}
        onClose={() => setSmartToolboxAction(null)}
        onApply={(act, config) => {
          console.log(`Executed Smart Toolbox Action: ${act}`, config);
          setTabs(tabs.map(t => t.id === activeTabId ? { ...t, isDirty: true } : t));
        }}
      />

      {/* WPS Dual-Screen Presenter View Cockpit Modal */}
      <PresenterViewModal
        isOpen={isPresenterOpen}
        onClose={() => setIsPresenterOpen(false)}
        slides={slidesList}
        currentSlideIndex={presenterSlideIdx}
        onSlideChange={(newIdx) => setPresenterSlideIdx(newIdx)}
        themeName="Modern Indigo"
      />
    </div>
  );
};
