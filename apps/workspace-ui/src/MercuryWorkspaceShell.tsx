import React, { useState, useRef } from 'react';
import './styles/mercury-tokens.css';
import { OfficeTab, DocumentType } from './types';
import { TopTabBar } from './components/TopTabBar';
import { HomeHubView } from './components/HomeHubView';
import { WriterWorkspace } from './components/WriterWorkspace';
import { SheetsWorkspace } from './components/SheetsWorkspace';
import { SlidesWorkspace } from './components/SlidesWorkspace';
import { PdfWorkspace } from './components/PdfWorkspace';
import { RightAssistantDrawer } from './components/RightAssistantDrawer';
import { PresenterViewModal } from './components/PresenterViewModal';
import {
  OfficeFileFormat,
  OfficeDocumentPayload,
  parseOfficeFileBytes,
  exportOfficeFileBytes,
  downloadOfficeBytes,
  MercuryFileModal,
} from './services/fileIo';

// ============================================================================
// INITIAL WPS OFFICE WORKSPACE TABS (ZERO JARGON)
// ============================================================================

const INITIAL_OFFICE_TABS: OfficeTab[] = [
  {
    id: 'tab-writer-1',
    title: 'Project_Proposal.docx',
    type: 'writer',
    isDirty: false,
    lastModified: 'Today, 2:15 PM',
  },
  {
    id: 'tab-sheets-1',
    title: 'Monthly_Financial_Budget.xlsx',
    type: 'sheets',
    isDirty: false,
    lastModified: 'Today, 1:40 PM',
  },
  {
    id: 'tab-slides-1',
    title: 'Executive_Pitch_Deck.pptx',
    type: 'slides',
    isDirty: false,
    lastModified: 'Today, 11:25 AM',
  },
  {
    id: 'tab-pdf-1',
    title: 'Service_Agreement_Contract.pdf',
    type: 'pdf',
    isDirty: false,
    lastModified: 'Yesterday, 5:10 PM',
  },
];

export const MercuryWorkspaceShell: React.FC = () => {
  // Tabs & Navigation State
  const [tabs, setTabs] = useState<OfficeTab[]>(INITIAL_OFFICE_TABS);
  const [activeTabId, setActiveTabId] = useState<string>('tab-writer-1');
  const [isHomeActive, setIsHomeActive] = useState<boolean>(false);

  // Assistant & History Drawer State
  const [isAiOpen, setIsAiOpen] = useState<boolean>(false);
  const [isHistoryOpen, setIsHistoryOpen] = useState<boolean>(false);
  const [drawerTab, setDrawerTab] = useState<'ai' | 'history'>('ai');

  // Presenter View Cockpit Modal
  const [isPresenterModalOpen, setIsPresenterModalOpen] = useState<boolean>(false);
  const [presenterSlideIdx, setPresenterSlideIdx] = useState<number>(0);

  // In-Memory File I/O Modal State
  const [fileModalState, setFileModalState] = useState<{
    isOpen: boolean;
    mode: 'open' | 'export';
  }>({
    isOpen: false,
    mode: 'open',
  });

  // Hidden File Input Ref for "Open File..."
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  // Current Active Tab
  const activeTab = tabs.find((t) => t.id === activeTabId) || tabs[0];

  // ─── Tab Action Handlers ───

  const handleSelectTab = (id: string) => {
    setActiveTabId(id);
    setIsHomeActive(false);
  };

  const handleSelectHome = () => {
    setIsHomeActive(true);
  };

  const handleCloseTab = (id: string) => {
    const remaining = tabs.filter((t) => t.id !== id);
    setTabs(remaining);
    if (activeTabId === id) {
      if (remaining.length > 0) {
        setActiveTabId(remaining[0].id);
      } else {
        setIsHomeActive(true);
      }
    }
  };

  const handleRenameTab = (id: string, newTitle: string) => {
    setTabs((prev) =>
      prev.map((t) => (t.id === id ? { ...t, title: newTitle, isDirty: true } : t))
    );
  };

  const handleNewDocument = (type: DocumentType) => {
    const ext = type === 'writer' ? '.docx' : type === 'sheets' ? '.xlsx' : type === 'slides' ? '.pptx' : '.pdf';
    const count = tabs.filter((t) => t.type === type).length + 1;
    const title =
      type === 'writer'
        ? `Untitled_Document_${count}${ext}`
        : type === 'sheets'
        ? `Untitled_Spreadsheet_${count}${ext}`
        : type === 'slides'
        ? `Untitled_Presentation_${count}${ext}`
        : `Untitled_PDF_${count}${ext}`;

    const newTab: OfficeTab = {
      id: `tab-${type}-${Date.now()}`,
      title,
      type,
      isDirty: false,
      lastModified: 'Just now',
    };

    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newTab.id);
    setIsHomeActive(false);
  };

  const handleOpenTemplate = (templateId: string) => {
    let title = 'Document_Template.docx';
    let type: DocumentType = 'writer';

    if (templateId === 'template-res' || templateId === 'template-proposal') {
      title = templateId === 'template-res' ? 'Executive_Resume.docx' : 'Enterprise_Service_Proposal.docx';
      type = 'writer';
    } else if (templateId === 'template-budget') {
      title = 'Company_Budget_Model.xlsx';
      type = 'sheets';
    } else if (templateId === 'template-pitch') {
      title = 'Startup_Pitch_Deck.pptx';
      type = 'slides';
    } else if (templateId === 'template-contract') {
      title = 'Standard_Service_Agreement.pdf';
      type = 'pdf';
    }

    const newTab: OfficeTab = {
      id: `tab-template-${Date.now()}`,
      title,
      type,
      isDirty: false,
      lastModified: 'Just now',
    };

    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newTab.id);
    setIsHomeActive(false);
  };

  const handleOpenFileClick = () => {
    setFileModalState({ isOpen: true, mode: 'open' });
  };

  const handleFileLoaded = (payload: OfficeDocumentPayload) => {
    let type: DocumentType = 'writer';
    if (payload.format === 'xlsx' || payload.format === 'csv') {
      type = 'sheets';
    } else if (payload.format === 'pptx') {
      type = 'slides';
    } else if (payload.format === 'pdf') {
      type = 'pdf';
    }

    const newTab: OfficeTab = {
      id: `tab-${type}-${Date.now()}`,
      title: payload.fileName,
      type,
      isDirty: false,
      lastModified: 'Just now',
    };

    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newTab.id);
    setIsHomeActive(false);
  };

  const handleFileInputChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files || files.length === 0) return;

    const file = files[0];
    try {
      const buffer = await file.arrayBuffer();
      const payload = await parseOfficeFileBytes(file.name, new Uint8Array(buffer));
      handleFileLoaded(payload);
    } catch (err) {
      console.error('Failed to parse file:', err);
    }
  };

  const handleDuplicateTab = (id: string) => {
    const tabToDupe = tabs.find((t) => t.id === id);
    if (!tabToDupe) return;

    const duplicated: OfficeTab = {
      ...tabToDupe,
      id: `tab-${tabToDupe.type}-${Date.now()}`,
      title: `Copy_of_${tabToDupe.title}`,
      lastModified: 'Just now',
      isDirty: false,
    };

    setTabs((prev) => [...prev, duplicated]);
    setActiveTabId(duplicated.id);
    setIsHomeActive(false);
  };

  // ─── PDF Conversion Handlers ───
  const handlePdfConvertToWord = () => {
    const newTab: OfficeTab = {
      id: `tab-writer-${Date.now()}`,
      title: 'Converted_From_PDF.docx',
      type: 'writer',
      isDirty: true,
      lastModified: 'Just now',
    };
    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newTab.id);
  };

  const handlePdfConvertToExcel = () => {
    const newTab: OfficeTab = {
      id: `tab-sheets-${Date.now()}`,
      title: 'Extracted_Tables_From_PDF.xlsx',
      type: 'sheets',
      isDirty: true,
      lastModified: 'Just now',
    };
    setTabs((prev) => [...prev, newTab]);
    setActiveTabId(newTab.id);
  };

  // ─── Export / Save Handler ───
  const handleExportFile = async (format: string) => {
    const fmt = (format ? format.toLowerCase() : 'docx') as OfficeFileFormat;
    const activeTitle = activeTab ? activeTab.title.replace(/\.[^/.]+$/, '') : 'document';
    const filename = `${activeTitle}.${fmt}`;

    let payload: OfficeDocumentPayload;
    if (activeTab?.type === 'writer' || fmt === 'docx') {
      payload = {
        format: 'docx',
        fileName: filename,
        paragraphs: [
          { style: 'Title', text: activeTitle.replace(/_/g, ' ') },
          { style: 'Heading 1', text: 'Executive Document Summary' },
          { style: 'Normal', text: 'Generated directly via OpenWPS in-memory binary pipeline.' },
        ],
      };
    } else if (activeTab?.type === 'sheets' || fmt === 'xlsx' || fmt === 'csv') {
      if (fmt === 'csv') {
        payload = {
          format: 'csv',
          fileName: filename,
          rows: [
            ['Region', 'Q1 Revenue', 'Q2 Revenue', 'Full Year Target'],
            ['North America', '$1,250,000', '$1,420,000', '$2,800,000'],
            ['Europe (EMEA)', '$890,000', '$940,000', '$1,900,000'],
            ['Asia Pacific', '$1,452,000', '$1,620,000', '$3,100,000'],
          ],
        };
      } else {
        payload = {
          format: 'xlsx',
          fileName: filename,
          sheetName: 'Sheet1',
          rows: [
            ['Region', 'Q1 Revenue', 'Q2 Revenue', 'Full Year Target'],
            ['North America', '$1,250,000', '$1,420,000', '$2,800,000'],
            ['Europe (EMEA)', '$890,000', '$940,000', '$1,900,000'],
            ['Asia Pacific', '$1,452,000', '$1,620,000', '$3,100,000'],
          ],
        };
      }
    } else if (activeTab?.type === 'slides' || fmt === 'pptx') {
      payload = {
        format: 'pptx',
        fileName: filename,
        slides: [
          {
            slide_number: 1,
            title: activeTitle.replace(/_/g, ' '),
            subtitle: 'Exported from OpenWPS Presentation Engine',
            bullets: [
              'Zero-intermediate AST preservation across formats.',
              'Native PowerPoint OOXML shape and text rendering.',
            ],
            speaker_notes: 'Presenter notes exported from presentation workspace.',
          },
        ],
      };
    } else {
      payload = {
        format: 'pdf',
        fileName: filename,
        pageCount: 1,
        blocks: [
          {
            block_id: 1,
            page_number: 1,
            role: 'Heading',
            bbox: [50, 72, 540, 108],
            text: activeTitle.replace(/_/g, ' '),
          },
          {
            block_id: 2,
            page_number: 1,
            role: 'Paragraph',
            bbox: [50, 120, 540, 180],
            text: 'This PDF document was produced cleanly in-memory with ISO 32000 %PDF-1.7 stream generation.',
          },
        ],
      };
    }

    try {
      const bytes = await exportOfficeFileBytes(payload);
      downloadOfficeBytes(filename, bytes);
    } catch (err) {
      console.error('Export error:', err);
    }
  };

  return (
    <div className="wps-shell">
      {/* Hidden File Input for Native File Picking */}
      <input
        type="file"
        ref={fileInputRef}
        onChange={handleFileInputChange}
        className="hidden"
        accept=".docx,.xlsx,.pptx,.pdf,.csv,.txt"
      />

      {/* ─── Zone 1: WPS All-in-One Top Tab Bar (48px) ─── */}
      <TopTabBar
        tabs={tabs}
        activeTabId={activeTabId}
        isHomeActive={isHomeActive}
        onSelectTab={handleSelectTab}
        onSelectHome={handleSelectHome}
        onCloseTab={handleCloseTab}
        onRenameTab={handleRenameTab}
        onNewDocument={handleNewDocument}
        onOpenFile={handleOpenFileClick}
        isAiOpen={isAiOpen}
        onToggleAi={() => {
          setIsAiOpen(!isAiOpen);
          setDrawerTab('ai');
        }}
        isHistoryOpen={isHistoryOpen}
        onToggleHistory={() => {
          setIsHistoryOpen(!isHistoryOpen);
          setDrawerTab('history');
        }}
        onExportFile={handleExportFile}
      />

      {/* ─── Main Workspace & Active Module Viewport ─── */}
      <div className="flex-1 flex overflow-hidden relative">
        {/* VIEW A: WPS Home Hub */}
        {isHomeActive ? (
          <HomeHubView
            onNewDocument={handleNewDocument}
            onOpenTemplate={handleOpenTemplate}
            onOpenFile={handleOpenFileClick}
            onFileLoaded={handleFileLoaded}
            recentTabs={tabs}
            onOpenRecentTab={handleSelectTab}
            onDuplicateTab={handleDuplicateTab}
            onDeleteTab={handleCloseTab}
            onOpenHistory={() => {
              setIsHistoryOpen(true);
              setDrawerTab('history');
            }}
          />
        ) : (
          <>
            {/* VIEW B: Active Productivity Module */}
            {activeTab?.type === 'writer' && (
              <WriterWorkspace
                onOpenAiAssist={(p) => {
                  setIsAiOpen(true);
                  setDrawerTab('ai');
                }}
              />
            )}

            {activeTab?.type === 'sheets' && (
              <SheetsWorkspace
                onOpenAiAssist={(p) => {
                  setIsAiOpen(true);
                  setDrawerTab('ai');
                }}
              />
            )}

            {activeTab?.type === 'slides' && (
              <SlidesWorkspace
                onOpenAiAssist={(p) => {
                  setIsAiOpen(true);
                  setDrawerTab('ai');
                }}
                onOpenPresenterView={() => setIsPresenterModalOpen(true)}
              />
            )}

            {activeTab?.type === 'pdf' && (
              <PdfWorkspace
                onConvertToWord={handlePdfConvertToWord}
                onConvertToExcel={handlePdfConvertToExcel}
                onOpenAiAssist={(p) => {
                  setIsAiOpen(true);
                  setDrawerTab('ai');
                }}
              />
            )}
          </>
        )}

        {/* ─── Zone 4: Right Assistant & History Drawer (340px) ─── */}
        <RightAssistantDrawer
          isOpen={isAiOpen || isHistoryOpen}
          onClose={() => {
            setIsAiOpen(false);
            setIsHistoryOpen(false);
          }}
          activeDrawerTab={drawerTab}
          onTabChange={(tab) => {
            setDrawerTab(tab);
            if (tab === 'ai') setIsAiOpen(true);
            if (tab === 'history') setIsHistoryOpen(true);
          }}
          docType={activeTab?.type || 'writer'}
          onApplyAiAction={(actionTag) => {
            // Contextual action dispatch
          }}
          onRestoreSnapshot={(id) => {
            // Restore snapshot
          }}
        />
      </div>

      {/* ─── In-Memory File I/O Modal Dialog ─── */}
      <MercuryFileModal
        isOpen={fileModalState.isOpen}
        mode={fileModalState.mode}
        currentTitle={activeTab?.title || 'document'}
        onClose={() => setFileModalState((prev) => ({ ...prev, isOpen: false }))}
        onFileLoaded={handleFileLoaded}
        onExportConfirm={handleExportFile}
      />

      {/* ─── Presenter Cockpit Modal (When invoked from Slides) ─── */}
      {isPresenterModalOpen && (
        <PresenterViewModal
          isOpen={isPresenterModalOpen}
          onClose={() => setIsPresenterModalOpen(false)}
          slides={[
            {
              id: 'slide-1',
              title: 'OpenWPS Executive Strategy',
              layoutId: 'title',
              shapes: [],
              connectors: [],
              speakerNotes: 'Welcome the board. OpenWPS delivers native office performance.',
            },
            {
              id: 'slide-2',
              title: 'Core Architectural Pillars',
              layoutId: 'title_content',
              shapes: [],
              connectors: [],
              speakerNotes: 'Walk through the 3 core pillars: unified kernel, zero jargon, and built-in AI.',
            },
            {
              id: 'slide-3',
              title: 'Rollout Roadmap',
              layoutId: 'title_content',
              shapes: [],
              connectors: [],
              speakerNotes: 'Milestones for Q1 and Q2.',
            },
          ]}
          currentSlideIndex={presenterSlideIdx}
          onSlideChange={(idx) => setPresenterSlideIdx(idx)}
          themeName="mercury-night"
        />
      )}
    </div>
  );
};

export default MercuryWorkspaceShell;
