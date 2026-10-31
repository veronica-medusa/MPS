import React from 'react';
import {
  FileText,
  Table,
  Presentation,
  FileCheck,
  Sparkles,
  Cpu,
  History,
  Layers,
  Clock,
  ArrowRight,
  Plus,
} from 'lucide-react';
import { MercuryButton } from './MercuryButton';
import { DocumentTab, DocumentType } from '../../types';

export interface MercuryHomeLaunchpadProps {
  tabs: DocumentTab[];
  activeTabId: string;
  onSelectTab: (id: string) => void;
  onNewDocument: (type: DocumentType) => void;
  onOpenHistory: () => void;
  onOpenAi: () => void;
}

export const MercuryHomeLaunchpad: React.FC<MercuryHomeLaunchpadProps> = ({
  tabs,
  activeTabId,
  onSelectTab,
  onNewDocument,
  onOpenHistory,
  onOpenAi,
}) => {
  return (
    <div className="flex-1 bg-[#171721] text-[#ededf3] overflow-y-auto px-8 py-10 selection:bg-[#5266eb]/30 select-none">
      <div className="max-w-6xl mx-auto space-y-12">
        {/* Hero Section */}
        <div className="space-y-4">
          <div className="inline-flex items-center space-x-2 px-3.5 py-1.5 rounded-full bg-[rgba(156,180,232,0.14)] border border-[rgba(156,180,232,0.25)] text-[#9cb4e8] text-xs font-[420]">
            <Sparkles className="w-3.5 h-3.5" />
            <span>OpenWPS 2.0 • Pure-Rust Office Kernel & Mercury Design System</span>
          </div>

          <h1 className="mercury-display text-white font-[480] tracking-tight">
            Welcome to OpenWPS Mercury
          </h1>
          <p className="mercury-lead text-[#c3c3cc] font-[480] max-w-2xl">
            Mathematical parity with Microsoft Office, real-time reactive formula DAG, 60 FPS vector presentations, and ISO 32000 PDF layout reconstruction.
          </p>
        </div>

        {/* 4-Up Stat Block Row (60px Numerals, Arcadia Display 480) */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {/* Stat 1: Open Documents */}
          <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
            <span className="mercury-stat text-white font-[480]">{tabs.length}</span>
            <div className="mt-2">
              <span className="mercury-ui text-[#9cb4e8] font-[420] block">Active Documents</span>
              <span className="mercury-legal text-[#70707d] font-[400]">Multi-tab session memory</span>
            </div>
          </div>

          {/* Stat 2: Reactive DAG Nodes */}
          <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
            <span className="mercury-stat text-[#38d9a9] font-[480]">150+</span>
            <div className="mt-2">
              <span className="mercury-ui text-[#9cb4e8] font-[420] block">Formula DAG Nodes</span>
              <span className="mercury-legal text-[#70707d] font-[400]">=PY(), XLOOKUP, Dynamic Spill</span>
            </div>
          </div>

          {/* Stat 3: Roaming Snapshots */}
          <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
            <span className="mercury-stat text-[#748ffc] font-[480]">256</span>
            <div className="mt-2">
              <span className="mercury-ui text-[#9cb4e8] font-[420] block">SHA-256 Snapshots</span>
              <span className="mercury-legal text-[#70707d] font-[400]">Content-addressed time machine</span>
            </div>
          </div>

          {/* Stat 4: AI & Macro Copilot */}
          <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
            <span className="mercury-stat text-[#da77f2] font-[480]">100%</span>
            <div className="mt-2">
              <span className="mercury-ui text-[#9cb4e8] font-[420] block">AST-Native AI RAG</span>
              <span className="mercury-legal text-[#70707d] font-[400]">Deterministic local & cloud copilot</span>
            </div>
          </div>
        </div>

        {/* 4-Up Feature Launch Cards (Writer, Spreadsheets, Presentation, PDF) */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="mercury-h2 text-white font-[480]">Document Engines</h2>
            <span className="mercury-legal text-[#c3c3cc]/70">Zero-latency instant initialization</span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* Card 1: Writer */}
            <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] hover:border-[#5266eb]/50 transition-all flex flex-col justify-between space-y-6 group">
              <div className="space-y-3">
                <div className="w-12 h-12 rounded-[16px] bg-[#3b5bdb]/15 border border-[#3b5bdb]/30 flex items-center justify-center text-[#748ffc] group-hover:scale-105 transition-transform">
                  <FileText className="w-6 h-6" />
                </div>
                <div>
                  <h3 className="mercury-h2 text-[20px] text-white font-[480]">OpenWPS Writer</h3>
                  <span className="mercury-legal text-[#9cb4e8] font-[420]">.DOCX • Word Processor</span>
                </div>
                <p className="mercury-body-lg text-[14px] text-[#c3c3cc]/80 font-[360] leading-relaxed">
                  Rope-backed editing, continuous pagination, tracked revisions, multi-column flow, and mathematical OOXML fidelity.
                </p>
              </div>

              <MercuryButton
                variant="primary"
                onClick={() => onNewDocument('writer')}
                className="w-full !rounded-[40px] text-xs"
                trailingIcon={<ArrowRight className="w-3.5 h-3.5" />}
              >
                Create Document
              </MercuryButton>
            </div>

            {/* Card 2: Spreadsheets */}
            <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] hover:border-[#0ca678]/50 transition-all flex flex-col justify-between space-y-6 group">
              <div className="space-y-3">
                <div className="w-12 h-12 rounded-[16px] bg-[#0ca678]/15 border border-[#0ca678]/30 flex items-center justify-center text-[#38d9a9] group-hover:scale-105 transition-transform">
                  <Table className="w-6 h-6" />
                </div>
                <div>
                  <h3 className="mercury-h2 text-[20px] text-white font-[480]">Spreadsheets</h3>
                  <span className="mercury-legal text-[#38d9a9] font-[420]">.XLSX • Formula Engine</span>
                </div>
                <p className="mercury-body-lg text-[14px] text-[#c3c3cc]/80 font-[360] leading-relaxed">
                  150+ formula DAG, dynamic 2D array spill, embedded Python (=PY()), Pivot Table Cube, and Smart Toolbox.
                </p>
              </div>

              <MercuryButton
                variant="glass-dark"
                onClick={() => onNewDocument('sheets')}
                className="w-full !rounded-[40px] text-xs hover:!bg-[#0ca678]/20 hover:!border-[#0ca678]/40"
                trailingIcon={<ArrowRight className="w-3.5 h-3.5" />}
              >
                Create Workbook
              </MercuryButton>
            </div>

            {/* Card 3: Presentation */}
            <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] hover:border-[#e8590c]/50 transition-all flex flex-col justify-between space-y-6 group">
              <div className="space-y-3">
                <div className="w-12 h-12 rounded-[16px] bg-[#e8590c]/15 border border-[#e8590c]/30 flex items-center justify-center text-[#ffa94d] group-hover:scale-105 transition-transform">
                  <Presentation className="w-6 h-6" />
                </div>
                <div>
                  <h3 className="mercury-h2 text-[20px] text-white font-[480]">Presentation</h3>
                  <span className="mercury-legal text-[#ffa94d] font-[420]">.PPTX • Vector Stage</span>
                </div>
                <p className="mercury-body-lg text-[14px] text-[#c3c3cc]/80 font-[360] leading-relaxed">
                  3-Tier master slide scene graph, 60 FPS Morph transitions, A* smart connectors, and dual-screen presenter cockpit.
                </p>
              </div>

              <MercuryButton
                variant="glass-dark"
                onClick={() => onNewDocument('slides')}
                className="w-full !rounded-[40px] text-xs hover:!bg-[#e8590c]/20 hover:!border-[#e8590c]/40"
                trailingIcon={<ArrowRight className="w-3.5 h-3.5" />}
              >
                Create Deck
              </MercuryButton>
            </div>

            {/* Card 4: PDF Studio */}
            <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] hover:border-[#e03131]/50 transition-all flex flex-col justify-between space-y-6 group">
              <div className="space-y-3">
                <div className="w-12 h-12 rounded-[16px] bg-[#e03131]/15 border border-[#e03131]/30 flex items-center justify-center text-[#ff8787] group-hover:scale-105 transition-transform">
                  <FileCheck className="w-6 h-6" />
                </div>
                <div>
                  <h3 className="mercury-h2 text-[20px] text-white font-[480]">PDF Studio</h3>
                  <span className="mercury-legal text-[#ff8787] font-[420]">.PDF • ISO 32000</span>
                </div>
                <p className="mercury-body-lg text-[14px] text-[#c3c3cc]/80 font-[360] leading-relaxed">
                  XY-Cut layout reconstruction, direct inline text editing with reflow, pure-Rust OCR, AcroForms, and AES-256 security.
                </p>
              </div>

              <MercuryButton
                variant="glass-dark"
                onClick={() => onNewDocument('pdf')}
                className="w-full !rounded-[40px] text-xs hover:!bg-[#e03131]/20 hover:!border-[#e03131]/40"
                trailingIcon={<ArrowRight className="w-3.5 h-3.5" />}
              >
                Open PDF Studio
              </MercuryButton>
            </div>
          </div>
        </div>

        {/* Open Documents Session Drawer */}
        <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <Clock className="w-4 h-4 text-[#9cb4e8]" />
              <h3 className="mercury-h2 text-[18px] text-white font-[480]">Active Session Tabs</h3>
            </div>
            <span className="mercury-legal text-[#70707d]">{tabs.length} tabs in memory</span>
          </div>

          <div className="divide-y divide-[rgba(175,178,206,0.10)]">
            {tabs.map((t) => (
              <div
                key={t.id}
                onClick={() => onSelectTab(t.id)}
                className={`py-3.5 px-4 rounded-[16px] flex items-center justify-between cursor-pointer transition-colors ${
                  t.id === activeTabId
                    ? 'bg-[rgba(156,180,232,0.15)] text-white'
                    : 'hover:bg-[rgba(175,178,206,0.08)] text-[#c3c3cc]'
                }`}
              >
                <div className="flex items-center space-x-3">
                  {t.type === 'writer' && <FileText className="w-4 h-4 text-[#748ffc]" />}
                  {t.type === 'sheets' && <Table className="w-4 h-4 text-[#38d9a9]" />}
                  {t.type === 'slides' && <Presentation className="w-4 h-4 text-[#ffa94d]" />}
                  {t.type === 'pdf' && <FileCheck className="w-4 h-4 text-[#ff8787]" />}

                  <div>
                    <span className="mercury-ui text-sm text-white font-[420]">{t.title}</span>
                    <span className="text-xs text-[#70707d] ml-3 uppercase tracking-wider">
                      {t.type}
                    </span>
                  </div>
                </div>

                <div className="flex items-center space-x-4">
                  {t.isDirty && (
                    <span className="text-[11px] px-2 py-0.5 rounded-full bg-[rgba(255,169,77,0.20)] text-[#ffa94d]">
                      Unsaved Changes
                    </span>
                  )}
                  <span className="mercury-legal text-xs text-[#9cb4e8] font-[420] flex items-center">
                    Switch to Tab <ArrowRight className="w-3 h-3 ml-1" />
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
