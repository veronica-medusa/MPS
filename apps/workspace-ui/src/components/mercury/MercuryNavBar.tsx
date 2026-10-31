import React from 'react';
import {
  FileText,
  Table,
  Presentation,
  FileCheck,
  History,
  Sparkles,
  Plus,
  Home,
  Save,
  Share2,
} from 'lucide-react';
import { MercuryButton } from './MercuryButton';
import { DocumentTab, DocumentType } from '../../types';

export interface MercuryNavBarProps {
  viewMode: 'launchpad' | 'workspace';
  onToggleViewMode: () => void;
  activeTab?: DocumentTab;
  onNewDocument: (type: DocumentType) => void;
  onOpenHistory: () => void;
  onOpenAi: () => void;
  onSave?: () => void;
}

export const MercuryNavBar: React.FC<MercuryNavBarProps> = ({
  viewMode,
  onToggleViewMode,
  activeTab,
  onNewDocument,
  onOpenHistory,
  onOpenAi,
  onSave,
}) => {
  return (
    <header className="h-[72px] bg-[#171721] text-[#ededf3] px-6 flex items-center justify-between border-b border-[rgba(175,178,206,0.18)] select-none shrink-0 z-30">
      {/* Left: Brand Wordmark & Mode Switcher */}
      <div className="flex items-center space-x-6">
        <div
          onClick={onToggleViewMode}
          className="flex items-center space-x-3 cursor-pointer group"
          title="Toggle Home Launchpad / Workspace"
        >
          {/* Logo Mark */}
          <div className="w-10 h-10 rounded-[14px] bg-gradient-to-tr from-[#5266eb] to-[#9cb4e8] flex items-center justify-center shadow-[0_2px_12px_rgba(82,102,235,0.35)] group-hover:scale-105 transition-transform">
            <span className="text-white font-[500] text-lg tracking-tight font-sans">W</span>
          </div>

          <div>
            <div className="flex items-center space-x-2">
              <span className="mercury-lead text-white font-[480] tracking-wide">OpenWPS</span>
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-[rgba(156,180,232,0.18)] text-[#9cb4e8] font-[420] tracking-wider uppercase">
                Mercury
              </span>
            </div>
            <p className="text-[11px] text-[#c3c3cc]/70 font-[360] tracking-normal -mt-0.5">
              High-Density Office Productivity Suite
            </p>
          </div>
        </div>

        {/* Global Nav-Link Capsules (40px height, pill radius) */}
        <nav className="hidden md:flex items-center space-x-2 pl-4 border-l border-[rgba(175,178,206,0.15)]">
          <button
            onClick={onToggleViewMode}
            className={`h-[40px] px-4 rounded-[40px] mercury-ui flex items-center space-x-2 transition-all ${
              viewMode === 'launchpad'
                ? 'bg-[rgba(156,180,232,0.20)] text-white shadow-sm'
                : 'text-[#ededf3]/80 hover:text-white hover:bg-[rgba(175,178,206,0.12)]'
            }`}
          >
            <Home className="w-4 h-4 text-[#9cb4e8]" />
            <span className="text-sm">Home Launchpad</span>
          </button>

          <button
            onClick={() => {
              if (viewMode !== 'workspace') onToggleViewMode();
            }}
            className={`h-[40px] px-4 rounded-[40px] mercury-ui flex items-center space-x-2 transition-all ${
              viewMode === 'workspace'
                ? 'bg-[rgba(156,180,232,0.20)] text-white shadow-sm'
                : 'text-[#ededf3]/80 hover:text-white hover:bg-[rgba(175,178,206,0.12)]'
            }`}
          >
            <FileText className="w-4 h-4 text-[#9cb4e8]" />
            <span className="text-sm">Document Workspace</span>
            {activeTab && (
              <span className="text-[11px] opacity-70 truncate max-w-[120px]">
                ({activeTab.title})
              </span>
            )}
          </button>
        </nav>
      </div>

      {/* Center: Quick Module Launchers */}
      <div className="hidden lg:flex items-center space-x-1.5 bg-[#1e1e2a]/80 p-1.5 rounded-[40px] border border-[rgba(175,178,206,0.12)]">
        <button
          onClick={() => onNewDocument('writer')}
          className="h-[32px] px-3 rounded-[32px] hover:bg-[rgba(156,180,232,0.15)] text-[#ededf3] text-xs flex items-center space-x-1.5 transition"
          title="New Writer (.docx)"
        >
          <FileText className="w-3.5 h-3.5 text-[#748ffc]" />
          <span>Writer</span>
        </button>

        <button
          onClick={() => onNewDocument('sheets')}
          className="h-[32px] px-3 rounded-[32px] hover:bg-[rgba(156,180,232,0.15)] text-[#ededf3] text-xs flex items-center space-x-1.5 transition"
          title="New Spreadsheets (.xlsx)"
        >
          <Table className="w-3.5 h-3.5 text-[#38d9a9]" />
          <span>Spreadsheets</span>
        </button>

        <button
          onClick={() => onNewDocument('slides')}
          className="h-[32px] px-3 rounded-[32px] hover:bg-[rgba(156,180,232,0.15)] text-[#ededf3] text-xs flex items-center space-x-1.5 transition"
          title="New Presentation (.pptx)"
        >
          <Presentation className="w-3.5 h-3.5 text-[#ffa94d]" />
          <span>Presentation</span>
        </button>

        <button
          onClick={() => onNewDocument('pdf')}
          className="h-[32px] px-3 rounded-[32px] hover:bg-[rgba(156,180,232,0.15)] text-[#ededf3] text-xs flex items-center space-x-1.5 transition"
          title="Open PDF Toolkit (.pdf)"
        >
          <FileCheck className="w-3.5 h-3.5 text-[#ff8787]" />
          <span>PDF Studio</span>
        </button>
      </div>

      {/* Right: Actions & Tools (Single Solid #5266eb Primary Action Rule) */}
      <div className="flex items-center space-x-3">
        {/* File Roaming Snapshot Trigger */}
        <MercuryButton
          variant="glass-dark"
          onClick={onOpenHistory}
          className="!h-[38px] !px-3.5"
          title="File Roaming SHA-256 Version History"
          icon={<History className="w-4 h-4 text-[#9cb4e8]" />}
        >
          <span className="hidden sm:inline text-xs">Roaming</span>
        </MercuryButton>

        {/* AST-Native WPS AI Co-Pilot */}
        <MercuryButton
          variant="tint-dark"
          onClick={onOpenAi}
          className="!h-[38px] !px-3.5"
          title="WPS AI & Python Studio"
          icon={<Sparkles className="w-4 h-4 text-[#9cb4e8]" />}
        >
          <span className="hidden sm:inline text-xs">WPS AI</span>
        </MercuryButton>

        {/* Collaborators Stack */}
        {activeTab?.collaborators && activeTab.collaborators.length > 0 && (
          <div className="flex -space-x-1.5 items-center mr-1">
            {activeTab.collaborators.slice(0, 3).map((c, i) => (
              <div
                key={c.id || i}
                title={c.name}
                className="w-7 h-7 rounded-full border border-[#171721] flex items-center justify-center text-[10px] font-[500] text-white shadow-xs"
                style={{ backgroundColor: c.color || '#5266eb' }}
              >
                {c.name.slice(0, 2).toUpperCase()}
              </div>
            ))}
          </div>
        )}

        {/* Single Primary Action: Save / Create Document */}
        <MercuryButton
          variant="primary"
          onClick={onSave}
          className="!h-[38px] !px-4"
          icon={<Save className="w-4 h-4" />}
        >
          <span className="text-sm font-[420]">Save</span>
        </MercuryButton>
      </div>
    </header>
  );
};
