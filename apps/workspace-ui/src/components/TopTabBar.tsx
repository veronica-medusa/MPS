import React from 'react';
import { 
  FileText, 
  Table, 
  Presentation, 
  FileCheck, 
  Plus, 
  X, 
  Columns, 
  Sun, 
  Moon, 
  Share2, 
  Sparkles, 
  Play,
  RotateCcw,
  RotateCw
} from 'lucide-react';
import { DocumentTab, DocumentType } from '../types';

interface TopTabBarProps {
  tabs: DocumentTab[];
  activeTabId: string;
  isSplitScreen: boolean;
  isDarkMode: boolean;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onNewDocument: (type: DocumentType) => void;
  onToggleSplitScreen: () => void;
  onToggleDarkMode: () => void;
  onOpenAi: () => void;
}

export const TopTabBar: React.FC<TopTabBarProps> = ({
  tabs,
  activeTabId,
  isSplitScreen,
  isDarkMode,
  onSelectTab,
  onCloseTab,
  onNewDocument,
  onToggleSplitScreen,
  onToggleDarkMode,
  onOpenAi,
}) => {
  const [showNewMenu, setShowNewMenu] = React.useState(false);

  const getTypeIcon = (type: DocumentType) => {
    switch (type) {
      case 'writer':
        return <FileText className="w-3.5 h-3.5 text-mercury-writer" />;
      case 'sheets':
        return <Table className="w-3.5 h-3.5 text-mercury-sheets" />;
      case 'slides':
        return <Presentation className="w-3.5 h-3.5 text-mercury-slides" />;
      case 'pdf':
        return <FileCheck className="w-3.5 h-3.5 text-mercury-pdf" />;
    }
  };

  const getBorderColor = (type: DocumentType) => {
    switch (type) {
      case 'writer':
        return 'border-t-mercury-writer';
      case 'sheets':
        return 'border-t-mercury-sheets';
      case 'slides':
        return 'border-t-mercury-slides';
      case 'pdf':
        return 'border-t-mercury-pdf';
    }
  };

  return (
    <header className="h-10 bg-white dark:bg-mercury-night border-b border-mercury-fog/30 flex items-center justify-between px-2 select-none z-30 transition-colors">
      {/* Left: Window Branding & Tabs */}
      <div className="flex items-center space-x-1 overflow-x-auto no-scrollbar flex-1 mr-4">
        <div className="flex items-center space-x-1.5 px-2.5 py-1 mr-2 rounded-mercury-md bg-mercury-primary/10 text-mercury-primary font-medium text-xs tracking-tight">
          <div className="w-2 h-2 rounded-full bg-mercury-primary animate-pulse" />
          <span className="font-semibold">OpenWPS</span>
        </div>

        {/* Tab Strip */}
        <div className="flex items-center space-x-1">
          {tabs.map((tab) => {
            const isActive = tab.id === activeTabId;
            return (
              <div
                key={tab.id}
                onClick={() => onSelectTab(tab.id)}
                className={`group relative flex items-center h-8 px-3 rounded-t-mercury-sm text-xs cursor-pointer border-t-2 transition-all ${
                  isActive
                    ? `bg-mercury-surface-tint dark:bg-mercury-night-raised text-mercury-ink dark:text-mercury-on-dark font-medium shadow-sm ${getBorderColor(
                        tab.type
                      )}`
                    : 'border-t-transparent hover:bg-mercury-lavender/50 dark:hover:bg-mercury-night-raised/50 text-mercury-ink-secondary dark:text-mercury-fog'
                }`}
              >
                <span className="mr-1.5">{getTypeIcon(tab.type)}</span>
                <span className="truncate max-w-[130px]">{tab.title}</span>

                {/* Dirty Dot */}
                {tab.isDirty && (
                  <span className="w-1.5 h-1.5 rounded-full bg-mercury-primary ml-1.5" title="Unsaved changes" />
                )}

                {/* Close Button */}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onCloseTab(tab.id);
                  }}
                  className="ml-2 p-0.5 rounded-full opacity-0 group-hover:opacity-100 hover:bg-mercury-fog/30 transition-opacity"
                  title="Close Tab"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
            );
          })}
        </div>

        {/* Plus Button with Menu */}
        <div className="relative">
          <button
            onClick={() => setShowNewMenu(!showNewMenu)}
            className="w-7 h-7 flex items-center justify-center rounded-mercury-sm text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night-raised transition-colors"
            title="New Document (Ctrl/Cmd + N)"
          >
            <Plus className="w-3.5 h-3.5" />
          </button>

          {showNewMenu && (
            <div
              className="absolute left-0 mt-1 w-48 bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-md shadow-lg py-1 z-50 text-xs"
              onMouseLeave={() => setShowNewMenu(false)}
            >
              <button
                onClick={() => {
                  onNewDocument('writer');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center space-x-2 hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <FileText className="w-4 h-4 text-mercury-writer" />
                <span>WPS Writer Document (.docx)</span>
              </button>
              <button
                onClick={() => {
                  onNewDocument('sheets');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center space-x-2 hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <Table className="w-4 h-4 text-mercury-sheets" />
                <span>WPS Spreadsheet (.xlsx)</span>
              </button>
              <button
                onClick={() => {
                  onNewDocument('slides');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center space-x-2 hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <Presentation className="w-4 h-4 text-mercury-slides" />
                <span>WPS Presentation (.pptx)</span>
              </button>
              <button
                onClick={() => {
                  onNewDocument('pdf');
                  setShowNewMenu(false);
                }}
                className="w-full px-3 py-1.5 text-left flex items-center space-x-2 hover:bg-mercury-lavender dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
              >
                <FileCheck className="w-4 h-4 text-mercury-pdf" />
                <span>WPS PDF Document (.pdf)</span>
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Right Controls: Split Screen, AI, Theme, Collaborators */}
      <div className="flex items-center space-x-2">
        {/* Active Tab Collaborators */}
        {tabs.find((t) => t.id === activeTabId)?.collaborators.map((c) => (
          <div
            key={c.id}
            className="w-6 h-6 rounded-full flex items-center justify-center text-[10px] text-white font-medium shadow-sm"
            style={{ backgroundColor: c.color }}
            title={`Collaborator: ${c.name}`}
          >
            {c.name.slice(0, 2).toUpperCase()}
          </div>
        ))}

        {/* Split Screen Toggle */}
        <button
          onClick={onToggleSplitScreen}
          className={`p-1.5 rounded-mercury-sm transition-colors ${
            isSplitScreen
              ? 'bg-mercury-primary text-white'
              : 'text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night-raised'
          }`}
          title="Toggle Split-Screen View (◫)"
        >
          <Columns className="w-3.5 h-3.5" />
        </button>

        {/* AI Co-Pilot Button */}
        <button
          onClick={onOpenAi}
          className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-pill bg-mercury-ai/10 text-mercury-ai hover:bg-mercury-ai/20 transition-all font-medium text-xs"
          title="Open WPS AI Co-Pilot"
        >
          <Sparkles className="w-3 h-3 text-mercury-ai animate-spin-slow" />
          <span>WPS AI</span>
        </button>

        {/* Dark/Light Mode Toggle */}
        <button
          onClick={onToggleDarkMode}
          className="p-1.5 rounded-mercury-sm text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night-raised transition-colors"
          title={isDarkMode ? 'Switch to Light Mode' : 'Switch to Precision Night Mode'}
        >
          {isDarkMode ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
        </button>
      </div>
    </header>
  );
};
