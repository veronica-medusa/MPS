import React, { useState } from 'react';
import {
  Home,
  FilePlus,
  FolderOpen,
  LayoutTemplate,
  History,
  Search,
  FileText,
  Table,
  Presentation,
  FileCheck,
  MoreVertical,
  Trash2,
  Copy,
  ExternalLink,
  Sparkles,
  ArrowRight,
  Moon,
  Sun,
  Check,
  Layers,
  Shield,
  Zap,
  Globe
} from 'lucide-react';
import { OfficeTab, DocumentType } from '../types';

interface HomeHubViewProps {
  onNewDocument: (type: DocumentType) => void;
  onOpenTemplate: (templateId: string) => void;
  onOpenFile: () => void;
  recentTabs: OfficeTab[];
  onOpenRecentTab: (id: string) => void;
  onDuplicateTab: (id: string) => void;
  onDeleteTab: (id: string) => void;
  onOpenHistory: () => void;
}

export const HomeHubView: React.FC<HomeHubViewProps> = ({
  onNewDocument,
  onOpenTemplate,
  onOpenFile,
  recentTabs,
  onOpenRecentTab,
  onDuplicateTab,
  onDeleteTab,
  onOpenHistory,
}) => {
  const [activeNav, setActiveNav] = useState<'home' | 'new' | 'open' | 'templates' | 'history' | 'showcase'>('home');
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<string>('all');
  const [isAnnualPricing, setIsAnnualPricing] = useState<boolean>(true);
  const [homeWorld, setHomeWorld] = useState<'light-workspace' | 'night-showcase'>('light-workspace');

  // Filtered recent tabs
  const filteredTabs = recentTabs.filter((tab) => {
    const matchesQuery = tab.title.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesType = filterType === 'all' || tab.type === filterType;
    return matchesQuery && matchesType;
  });

  const getDocBadge = (type: DocumentType) => {
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
    <div className="wps-home-workspace">
      {/* Left Sidebar Navigation (220px) */}
      <aside className="wps-home-nav-sidebar">
        <div className="px-3 py-2 mb-4">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-[10px] bg-gradient-to-tr from-[#5266eb] to-[#9cb4e8] flex items-center justify-center text-white font-medium text-sm shadow-sm">
              W
            </div>
            <div>
              <div className="font-semibold text-sm text-mercury-ink dark:text-mercury-on-dark tracking-tight">OpenWPS</div>
              <div className="text-[11px] text-mercury-slate">Mercury Design System</div>
            </div>
          </div>
        </div>

        <nav className="flex flex-col gap-1">
          <button
            onClick={() => {
              setActiveNav('home');
              setHomeWorld('light-workspace');
            }}
            className={`wps-home-nav-item ${activeNav === 'home' && homeWorld === 'light-workspace' ? 'is-active' : ''}`}
          >
            <Home className="w-4 h-4" />
            <span>Productivity Hub</span>
          </button>

          <button
            onClick={() => {
              setActiveNav('showcase');
              setHomeWorld('night-showcase');
            }}
            className={`wps-home-nav-item ${homeWorld === 'night-showcase' ? 'is-active' : ''}`}
          >
            <Moon className="w-4 h-4 text-mercury-periwinkle" />
            <span>Night Launchpad</span>
          </button>

          <button
            onClick={() => {
              setActiveNav('new');
              onNewDocument('writer');
            }}
            className={`wps-home-nav-item ${activeNav === 'new' ? 'is-active' : ''}`}
          >
            <FilePlus className="w-4 h-4" />
            <span>New Document</span>
          </button>

          <button
            onClick={() => {
              setActiveNav('open');
              onOpenFile();
            }}
            className={`wps-home-nav-item ${activeNav === 'open' ? 'is-active' : ''}`}
          >
            <FolderOpen className="w-4 h-4" />
            <span>Open File</span>
          </button>

          <button
            onClick={() => setActiveNav('templates')}
            className={`wps-home-nav-item ${activeNav === 'templates' ? 'is-active' : ''}`}
          >
            <LayoutTemplate className="w-4 h-4" />
            <span>Templates</span>
          </button>

          <button
            onClick={() => {
              setActiveNav('history');
              onOpenHistory();
            }}
            className={`wps-home-nav-item ${activeNav === 'history' ? 'is-active' : ''}`}
          >
            <History className="w-4 h-4" />
            <span>File Roaming / History</span>
          </button>
        </nav>

        {/* World Switcher Capsule */}
        <div className="mt-4 p-2 rounded-mercury-md bg-mercury-canvas border border-mercury-fog/30 flex items-center justify-between text-xs">
          <span className="text-[11px] text-mercury-slate font-medium">Surface World:</span>
          <button
            onClick={() => setHomeWorld(homeWorld === 'light-workspace' ? 'night-showcase' : 'light-workspace')}
            className="px-2.5 py-1 rounded-full bg-white dark:bg-mercury-night-raised border border-mercury-fog/40 text-[11px] flex items-center gap-1.5 font-medium hover:border-mercury-primary transition-colors"
          >
            {homeWorld === 'light-workspace' ? (
              <>
                <Moon className="w-3 h-3 text-mercury-primary" />
                <span>Night</span>
              </>
            ) : (
              <>
                <Sun className="w-3 h-3 text-amber-500" />
                <span>Light</span>
              </>
            )}
          </button>
        </div>

        <div className="mt-auto p-3 rounded-mercury-md bg-mercury-lavender/60 border border-mercury-primary/20 text-xs">
          <div className="flex items-center gap-1.5 font-semibold text-mercury-primary mb-1">
            <Sparkles className="w-3.5 h-3.5" />
            <span>WPS AI Connected</span>
          </div>
          <p className="text-[11px] text-mercury-ink-secondary leading-relaxed">
            AI Assistant is active for instant document drafting, formula generation, and slide creation.
          </p>
        </div>
      </aside>

      {/* ─── WORLD A: LIGHT WORKSPACE HUB ─── */}
      {homeWorld === 'light-workspace' && (
        <main className="wps-home-main">
          {/* Top Quick-Create Row (4 Clear Cards) */}
          <section>
            <div className="flex items-center justify-between mb-4">
              <div>
                <h2 className="text-xl font-semibold text-mercury-ink dark:text-mercury-on-dark tracking-tight">
                  Create New Document
                </h2>
                <p className="text-xs text-mercury-slate mt-0.5">Select a productivity module to begin editing immediately</p>
              </div>
              <button
                onClick={() => setHomeWorld('night-showcase')}
                className="button-tint-light h-8 px-3 text-xs flex items-center gap-1.5"
              >
                <Moon className="w-3.5 h-3.5" />
                <span>View Night Launchpad</span>
              </button>
            </div>

            <div className="wps-quick-create-grid">
              {/* Card 1: Writer */}
              <div
                onClick={() => onNewDocument('writer')}
                className="wps-quick-card group"
              >
                <div className="w-10 h-10 rounded-lg bg-blue-50 text-blue-600 flex items-center justify-center font-bold text-lg group-hover:bg-blue-600 group-hover:text-white transition-colors">
                  W
                </div>
                <div>
                  <h3 className="font-semibold text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    WPS Writer
                  </h3>
                  <p className="text-xs text-mercury-slate mt-1 leading-relaxed">
                    Create rich documents, letters, reports, and formatted articles.
                  </p>
                </div>
                <div className="mt-auto text-[11px] font-medium text-blue-600 flex items-center gap-1">
                  <span>New Blank Document</span>
                  <ArrowRight className="w-3 h-3 group-hover:translate-x-0.5 transition-transform" />
                </div>
              </div>

              {/* Card 2: Spreadsheets */}
              <div
                onClick={() => onNewDocument('sheets')}
                className="wps-quick-card group"
              >
                <div className="w-10 h-10 rounded-lg bg-emerald-50 text-emerald-600 flex items-center justify-center font-bold text-lg group-hover:bg-emerald-600 group-hover:text-white transition-colors">
                  S
                </div>
                <div>
                  <h3 className="font-semibold text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    WPS Spreadsheets
                  </h3>
                  <p className="text-xs text-mercury-slate mt-1 leading-relaxed">
                    Calculate formulas, financial budgets, charts, and data tables.
                  </p>
                </div>
                <div className="mt-auto text-[11px] font-medium text-emerald-600 flex items-center gap-1">
                  <span>New Blank Workbook</span>
                  <ArrowRight className="w-3 h-3 group-hover:translate-x-0.5 transition-transform" />
                </div>
              </div>

              {/* Card 3: Presentation */}
              <div
                onClick={() => onNewDocument('slides')}
                className="wps-quick-card group"
              >
                <div className="w-10 h-10 rounded-lg bg-orange-50 text-orange-600 flex items-center justify-center font-bold text-lg group-hover:bg-orange-600 group-hover:text-white transition-colors">
                  P
                </div>
                <div>
                  <h3 className="font-semibold text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    WPS Presentation
                  </h3>
                  <p className="text-xs text-mercury-slate mt-1 leading-relaxed">
                    Design pitch decks, business proposals, and interactive slideshows.
                  </p>
                </div>
                <div className="mt-auto text-[11px] font-medium text-orange-600 flex items-center gap-1">
                  <span>New Blank Slides</span>
                  <ArrowRight className="w-3 h-3 group-hover:translate-x-0.5 transition-transform" />
                </div>
              </div>

              {/* Card 4: PDF */}
              <div
                onClick={() => onNewDocument('pdf')}
                className="wps-quick-card group"
              >
                <div className="w-10 h-10 rounded-lg bg-rose-50 text-rose-600 flex items-center justify-center font-bold text-sm group-hover:bg-rose-600 group-hover:text-white transition-colors">
                  PDF
                </div>
                <div>
                  <h3 className="font-semibold text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    WPS PDF Toolkit
                  </h3>
                  <p className="text-xs text-mercury-slate mt-1 leading-relaxed">
                    Read, annotate, sign, and convert PDFs to Word or Excel files.
                  </p>
                </div>
                <div className="mt-auto text-[11px] font-medium text-rose-600 flex items-center gap-1">
                  <span>Open / Convert PDF</span>
                  <ArrowRight className="w-3 h-3 group-hover:translate-x-0.5 transition-transform" />
                </div>
              </div>
            </div>
          </section>

          {/* Template Starters */}
          <section>
            <div className="flex items-center justify-between mb-4">
              <div>
                <h2 className="text-xl font-semibold text-mercury-ink dark:text-mercury-on-dark tracking-tight">
                  Recommended Templates
                </h2>
                <p className="text-xs text-mercury-slate mt-0.5">One-click professionally populated starter files</p>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div
                onClick={() => onOpenTemplate('template-proposal')}
                className="wps-template-card group"
              >
                <div className="w-12 h-12 rounded-lg bg-blue-100/70 text-blue-700 flex items-center justify-center flex-shrink-0">
                  <FileText className="w-6 h-6" />
                </div>
                <div>
                  <h4 className="font-medium text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    Project Proposal.docx
                  </h4>
                  <p className="text-xs text-mercury-slate mt-0.5">Executive plan, milestones, budget table</p>
                </div>
              </div>

              <div
                onClick={() => onOpenTemplate('template-budget')}
                className="wps-template-card group"
              >
                <div className="w-12 h-12 rounded-lg bg-emerald-100/70 text-emerald-700 flex items-center justify-center flex-shrink-0">
                  <Table className="w-6 h-6" />
                </div>
                <div>
                  <h4 className="font-medium text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    Monthly Financial Budget.xlsx
                  </h4>
                  <p className="text-xs text-mercury-slate mt-0.5">Revenue, expenses, formulas, live chart</p>
                </div>
              </div>

              <div
                onClick={() => onOpenTemplate('template-pitch')}
                className="wps-template-card group"
              >
                <div className="w-12 h-12 rounded-lg bg-orange-100/70 text-orange-700 flex items-center justify-center flex-shrink-0">
                  <Presentation className="w-6 h-6" />
                </div>
                <div>
                  <h4 className="font-medium text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    Executive Pitch Deck.pptx
                  </h4>
                  <p className="text-xs text-mercury-slate mt-0.5">Problem, solution, market size, roadmap</p>
                </div>
              </div>

              <div
                onClick={() => onOpenTemplate('template-contract')}
                className="wps-template-card group"
              >
                <div className="w-12 h-12 rounded-lg bg-rose-100/70 text-rose-700 flex items-center justify-center flex-shrink-0">
                  <FileCheck className="w-6 h-6" />
                </div>
                <div>
                  <h4 className="font-medium text-sm text-mercury-ink group-hover:text-mercury-primary transition-colors">
                    Service Agreement Contract.pdf
                  </h4>
                  <p className="text-xs text-mercury-slate mt-0.5">Master terms, payment clauses, signature line</p>
                </div>
              </div>
            </div>
          </section>

          {/* Recent Files Table */}
          <section>
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-4">
              <div>
                <h2 className="text-xl font-semibold text-mercury-ink dark:text-mercury-on-dark tracking-tight">
                  Recent Files
                </h2>
                <p className="text-xs text-mercury-slate mt-0.5">Continue editing your local and cloud documents</p>
              </div>

              <div className="flex items-center gap-3 w-full sm:w-auto">
                {/* Filter Pills */}
                <div className="flex items-center bg-white dark:bg-mercury-night-raised border border-mercury-fog/40 rounded-mercury-pill p-0.5 text-xs">
                  <button
                    onClick={() => setFilterType('all')}
                    className={`px-3 py-1 rounded-mercury-pill transition-colors ${
                      filterType === 'all' ? 'bg-mercury-primary text-white' : 'text-mercury-ink-secondary'
                    }`}
                  >
                    All
                  </button>
                  <button
                    onClick={() => setFilterType('writer')}
                    className={`px-3 py-1 rounded-mercury-pill transition-colors ${
                      filterType === 'writer' ? 'bg-mercury-primary text-white' : 'text-mercury-ink-secondary'
                    }`}
                  >
                    Documents
                  </button>
                  <button
                    onClick={() => setFilterType('sheets')}
                    className={`px-3 py-1 rounded-mercury-pill transition-colors ${
                      filterType === 'sheets' ? 'bg-mercury-primary text-white' : 'text-mercury-ink-secondary'
                    }`}
                  >
                    Spreadsheets
                  </button>
                  <button
                    onClick={() => setFilterType('slides')}
                    className={`px-3 py-1 rounded-mercury-pill transition-colors ${
                      filterType === 'slides' ? 'bg-mercury-primary text-white' : 'text-mercury-ink-secondary'
                    }`}
                  >
                    Presentations
                  </button>
                  <button
                    onClick={() => setFilterType('pdf')}
                    className={`px-3 py-1 rounded-mercury-pill transition-colors ${
                      filterType === 'pdf' ? 'bg-mercury-primary text-white' : 'text-mercury-ink-secondary'
                    }`}
                  >
                    PDFs
                  </button>
                </div>

                {/* Search Box */}
                <div className="relative flex-1 sm:w-64">
                  <Search className="w-3.5 h-3.5 text-mercury-slate absolute left-3 top-1/2 -translate-y-1/2" />
                  <input
                    type="text"
                    placeholder="Search recent files..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full h-8 pl-8 pr-3 bg-white dark:bg-mercury-night-raised border border-mercury-fog/40 rounded-mercury-pill text-xs text-mercury-ink dark:text-mercury-on-dark outline-none focus:border-mercury-primary transition-colors"
                  />
                </div>
              </div>
            </div>

            <div className="bg-white dark:bg-mercury-night-raised border border-mercury-fog/40 rounded-mercury-md shadow-sm overflow-hidden">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="bg-mercury-surface-tint border-b border-mercury-fog/30 text-mercury-slate font-medium">
                    <th className="py-2.5 px-4">File Name</th>
                    <th className="py-2.5 px-4">Type</th>
                    <th className="py-2.5 px-4">Last Modified</th>
                    <th className="py-2.5 px-4 text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-mercury-fog/20">
                  {filteredTabs.length === 0 ? (
                    <tr>
                      <td colSpan={4} className="py-8 text-center text-mercury-slate">
                        No files found matching your criteria.
                      </td>
                    </tr>
                  ) : (
                    filteredTabs.map((tab) => (
                      <tr
                        key={tab.id}
                        onClick={() => onOpenRecentTab(tab.id)}
                        className="hover:bg-mercury-lavender/30 dark:hover:bg-mercury-night cursor-pointer transition-colors"
                      >
                        <td className="py-3 px-4 flex items-center gap-3">
                          {getDocBadge(tab.type)}
                          <span className="font-medium text-mercury-ink dark:text-mercury-on-dark">{tab.title}</span>
                        </td>
                        <td className="py-3 px-4 text-mercury-slate uppercase text-[11px] tracking-wider">
                          {tab.type}
                        </td>
                        <td className="py-3 px-4 text-mercury-slate">
                          {tab.lastModified || 'Just now'}
                        </td>
                        <td className="py-3 px-4 text-right">
                          <div
                            className="flex items-center justify-end gap-1"
                            onClick={(e) => e.stopPropagation()}
                          >
                            <button
                              onClick={() => onOpenRecentTab(tab.id)}
                              className="p-1 rounded hover:bg-mercury-lavender text-mercury-slate hover:text-mercury-primary transition-colors"
                              title="Open File"
                            >
                              <ExternalLink className="w-3.5 h-3.5" />
                            </button>
                            <button
                              onClick={() => onDuplicateTab(tab.id)}
                              className="p-1 rounded hover:bg-mercury-lavender text-mercury-slate hover:text-mercury-primary transition-colors"
                              title="Duplicate File"
                            >
                              <Copy className="w-3.5 h-3.5" />
                            </button>
                            <button
                              onClick={() => onDeleteTab(tab.id)}
                              className="p-1 rounded hover:bg-red-50 text-mercury-slate hover:text-red-600 transition-colors"
                              title="Delete File"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </section>
        </main>
      )}

      {/* ─── WORLD B: CANONICAL MERCURY NIGHT LAUNCHPAD SHOWCASE (DESIGN-mercury.md) ─── */}
      {homeWorld === 'night-showcase' && (
        <main className="flex-1 bg-[#171721] text-[#ededf3] overflow-y-auto px-8 py-10 selection:bg-[#5266eb]/30 select-none">
          <div className="max-w-6xl mx-auto space-y-16">
            {/* Top Showcase Navigation Header */}
            <div className="flex items-center justify-between pb-6 border-b border-white/10">
              <div className="inline-flex items-center space-x-2 px-3.5 py-1.5 rounded-full bg-[rgba(156,180,232,0.14)] border border-[rgba(156,180,232,0.25)] text-[#9cb4e8] text-xs font-[420]">
                <Sparkles className="w-3.5 h-3.5" />
                <span>Mercury Design System • Dark Night World Specification</span>
              </div>
              <button
                onClick={() => setHomeWorld('light-workspace')}
                className="button-tint-dark h-8 px-4 text-xs flex items-center gap-1.5"
              >
                <Sun className="w-3.5 h-3.5 text-amber-400" />
                <span>Return to Office Workspace</span>
              </button>
            </div>

            {/* Hero Section */}
            <div className="space-y-4 max-w-3xl">
              <h1 className="mercury-display text-white font-[480] tracking-tight">
                Quiet, precise and slightly nocturnal.
              </h1>
              <p className="mercury-lead text-[#c3c3cc] font-[480] leading-relaxed">
                OpenWPS moves between a deep ink-violet night world on the launchpad and a pale near-white light world on document canvases, unified by one confident indigo action pill.
              </p>
              <div className="flex items-center gap-3 pt-2">
                <button
                  onClick={() => onNewDocument('writer')}
                  className="button-primary h-10 px-5 text-sm flex items-center gap-2"
                >
                  <span>Open Account & Create File</span>
                  <ArrowRight className="w-4 h-4" />
                </button>
                <button
                  onClick={() => setHomeWorld('light-workspace')}
                  className="button-glass-dark h-10 px-5 text-sm flex items-center gap-2"
                >
                  <span>Launch Office Canvas</span>
                </button>
              </div>
            </div>

            {/* 4-Up Stat Block Row (60px Numerals, Arcadia Display 480) */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
                <span className="mercury-stat text-white font-[480]">120<span className="text-2xl">fps</span></span>
                <div className="mt-2">
                  <span className="mercury-ui text-[#9cb4e8] font-[420] block">Vector Rendering</span>
                  <span className="mercury-legal text-[#70707d] font-[400]">Zero-jitter cosmic-text glyphs</span>
                </div>
              </div>

              <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
                <span className="mercury-stat text-[#38d9a9] font-[480]">150+</span>
                <div className="mt-2">
                  <span className="mercury-ui text-[#9cb4e8] font-[420] block">Reactive Formulas</span>
                  <span className="mercury-legal text-[#70707d] font-[400]">=PY(), XLOOKUP, Dynamic Spill</span>
                </div>
              </div>

              <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
                <span className="mercury-stat text-[#748ffc] font-[480]">1.04M</span>
                <div className="mt-2">
                  <span className="mercury-ui text-[#9cb4e8] font-[420] block">Spreadsheet Rows</span>
                  <span className="mercury-legal text-[#70707d] font-[400]">Sparse BTreeMap memory layout</span>
                </div>
              </div>

              <div className="p-6 rounded-[24px] bg-[#1e1e2a] border border-[rgba(175,178,206,0.16)] flex flex-col justify-between">
                <span className="mercury-stat text-[#da77f2] font-[480]">AES-256</span>
                <div className="mt-2">
                  <span className="mercury-ui text-[#9cb4e8] font-[420] block">ISO 32000 Security</span>
                  <span className="mercury-legal text-[#70707d] font-[400]">Permanent byte-level redaction</span>
                </div>
              </div>
            </div>

            {/* 4 Feature Cards (feature-card-dark in #1e1e2a) */}
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <div>
                  <span className="eyebrow-label-dark">PRODUCT PLATFORM</span>
                  <h2 className="mercury-h2 text-white font-[480] mt-1">Four Native Document Engines</h2>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {/* Feature Card 1 */}
                <div className="mercury-feature-card-dark flex flex-col justify-between">
                  <div className="space-y-3">
                    <span className="text-blue-400 font-bold text-lg">W</span>
                    <h3 className="mercury-h2 text-xl font-[480] text-white">WPS Writer</h3>
                    <p className="text-sm text-[#c3c3cc]/80 font-[360] leading-relaxed">
                      Rope buffer text pagination, cascading styles, and Track Changes redlines.
                    </p>
                  </div>
                  <button
                    onClick={() => onNewDocument('writer')}
                    className="button-glass-dark mt-6 w-full text-xs"
                  >
                    Launch Writer
                  </button>
                </div>

                {/* Feature Card 2 */}
                <div className="mercury-feature-card-dark flex flex-col justify-between">
                  <div className="space-y-3">
                    <span className="text-emerald-400 font-bold text-lg">S</span>
                    <h3 className="mercury-h2 text-xl font-[480] text-white">Spreadsheets</h3>
                    <p className="text-sm text-[#c3c3cc]/80 font-[360] leading-relaxed">
                      Reactive formula dependency DAG, 150+ financial functions, and live charts.
                    </p>
                  </div>
                  <button
                    onClick={() => onNewDocument('sheets')}
                    className="button-glass-dark mt-6 w-full text-xs"
                  >
                    Launch Spreadsheets
                  </button>
                </div>

                {/* Feature Card 3 */}
                <div className="mercury-feature-card-dark flex flex-col justify-between">
                  <div className="space-y-3">
                    <span className="text-orange-400 font-bold text-lg">P</span>
                    <h3 className="mercury-h2 text-xl font-[480] text-white">Presentation</h3>
                    <p className="text-sm text-[#c3c3cc]/80 font-[360] leading-relaxed">
                      16:9 vector stage, parametric shapes, Morph transitions, and presenter view.
                    </p>
                  </div>
                  <button
                    onClick={() => onNewDocument('slides')}
                    className="button-glass-dark mt-6 w-full text-xs"
                  >
                    Launch Presentation
                  </button>
                </div>

                {/* Feature Card 4 */}
                <div className="mercury-feature-card-dark flex flex-col justify-between">
                  <div className="space-y-3">
                    <span className="text-rose-400 font-bold text-lg">PDF</span>
                    <h3 className="mercury-h2 text-xl font-[480] text-white">PDF Toolkit</h3>
                    <p className="text-sm text-[#c3c3cc]/80 font-[360] leading-relaxed">
                      ISO 32000-2 layout reconstructor, digital signatures, and PDF to Word/Excel.
                    </p>
                  </div>
                  <button
                    onClick={() => onNewDocument('pdf')}
                    className="button-glass-dark mt-6 w-full text-xs"
                  >
                    Launch PDF Toolkit
                  </button>
                </div>
              </div>
            </div>

            {/* Pricing Card & Comparison Table (pricing-card) */}
            <div className="space-y-6">
              <div className="text-center space-y-2 max-w-xl mx-auto">
                <span className="eyebrow-label-dark">TRANSPARENT PLANS</span>
                <h2 className="mercury-h1 text-white font-[480]">Choose Your Workplace Plan</h2>
                <p className="text-sm text-[#c3c3cc] font-[360]">
                  No hidden limits. Scale from individuals to enterprise engineering teams.
                </p>
              </div>

              <div className="mercury-pricing-card">
                {/* Header with Eyebrow and Toggle */}
                <div className="pricing-card-header">
                  <span className="eyebrow-label">COMPARE PLANS</span>
                  <div className="flex items-center gap-3">
                    <span className="text-xs text-mercury-ink-secondary">
                      Annual Pricing (15% off)
                    </span>
                    <button
                      onClick={() => setIsAnnualPricing(!isAnnualPricing)}
                      className={`mercury-toggle-switch ${isAnnualPricing ? 'is-on' : ''}`}
                    >
                      <span className="mercury-toggle-knob" />
                    </button>
                  </div>
                </div>

                {/* 3 Pricing Tier Columns */}
                <div className="pricing-tier-grid">
                  {/* Tier 1: Core */}
                  <div className="mercury-pricing-tier-column">
                    <div>
                      <div className="pricing-tier-name">Starter</div>
                      <div className="pricing-tier-price mt-2">$0</div>
                      <p className="pricing-tier-description mt-2">
                        Complete client-side office suite with local file editing and zero cloud lock-in.
                      </p>
                    </div>
                    <button
                      onClick={() => onNewDocument('writer')}
                      className="button-tint-light mt-auto w-full text-xs"
                    >
                      Get Started Free
                    </button>
                    <ul className="text-xs text-mercury-ink space-y-2 pt-4 border-t border-gray-200">
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>All 4 productivity modules</span>
                      </li>
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>Offline WebAssembly engine</span>
                      </li>
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>Standard export (.docx, .xlsx, .pptx, .pdf)</span>
                      </li>
                    </ul>
                  </div>

                  {/* Tier 2: Plus (Featured with Primary Pill) */}
                  <div className="mercury-pricing-tier-column bg-mercury-surface-tint border-x border-gray-200">
                    <div>
                      <div className="pricing-tier-name">
                        Pro <span className="brand-suffix">Plus</span>
                      </div>
                      <div className="pricing-tier-price mt-2">
                        {isAnnualPricing ? '$12' : '$15'}
                        <span className="text-sm font-normal text-mercury-slate">/user/mo</span>
                      </div>
                      <p className="pricing-tier-description mt-2">
                        For growing teams needing real-time collaboration, cloud roaming, and AI assistants.
                      </p>
                    </div>
                    <button
                      onClick={() => onNewDocument('writer')}
                      className="button-primary mt-auto w-full text-xs"
                    >
                      Start Free Trial
                    </button>
                    <ul className="text-xs text-mercury-ink space-y-2 pt-4 border-t border-gray-200">
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>Unlimited File Roaming Snapshots</span>
                      </li>
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>Embedded Python (=PY) execution</span>
                      </li>
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>AST-native WPS AI Co-pilot</span>
                      </li>
                    </ul>
                  </div>

                  {/* Tier 3: Enterprise */}
                  <div className="mercury-pricing-tier-column">
                    <div>
                      <div className="pricing-tier-name">Enterprise</div>
                      <div className="pricing-tier-price mt-2">Custom</div>
                      <p className="pricing-tier-description mt-2">
                        Dedicated on-premises deployments, custom audit compliance, and SSO / SAML.
                      </p>
                    </div>
                    <button
                      onClick={() => alert('Contacting enterprise sales team...')}
                      className="button-tint-light mt-auto w-full text-xs"
                    >
                      Contact Sales
                    </button>
                    <ul className="text-xs text-mercury-ink space-y-2 pt-4 border-t border-gray-200">
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>On-premises air-gapped deployment</span>
                      </li>
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>Custom FIPS 140-2 encryption keys</span>
                      </li>
                      <li className="flex items-center gap-2">
                        <Check className="w-3.5 h-3.5 text-mercury-primary" />
                        <span>24/7 dedicated engineering support</span>
                      </li>
                    </ul>
                  </div>
                </div>
              </div>
            </div>

            {/* Pull Quote Block (quote-block) */}
            <div className="mercury-quote-block">
              <p className="quote-block-text text-white">
                “OpenWPS combines the quiet craftsmanship of a private financial bank with the raw computational performance of a native Rust kernel.”
              </p>
              <p className="quote-block-attribution text-[#9cb4e8]">
                Chief Architect, Fortune 500 Infrastructure Group
              </p>
            </div>

            {/* Logo Strip (logo-strip) */}
            <div className="mercury-logo-strip text-white/50 text-sm tracking-widest font-mono">
              <span>ACME CORP</span>
              <span>•</span>
              <span>GLOBAL VENTURES</span>
              <span>•</span>
              <span>STATION NINE</span>
              <span>•</span>
              <span>HYPERION LABS</span>
            </div>

            {/* Site Footer (site-footer) */}
            <footer className="pt-8 border-t border-white/10 text-xs text-[#70707d] flex flex-col sm:flex-row items-center justify-between gap-4">
              <div className="flex items-center gap-4">
                <span className="text-[#ededf3]">OpenWPS Mercury</span>
                <span>Version 2.0.0</span>
                <span>Pure-Rust & WebAssembly Kernel</span>
              </div>
              <div className="flex items-center gap-6">
                <span className="hover:text-white cursor-pointer" onClick={() => setHomeWorld('light-workspace')}>Productivity Hub</span>
                <span className="hover:text-white cursor-pointer" onClick={() => onNewDocument('writer')}>Writer</span>
                <span className="hover:text-white cursor-pointer" onClick={() => onNewDocument('sheets')}>Spreadsheets</span>
                <span className="hover:text-white cursor-pointer" onClick={() => onNewDocument('slides')}>Presentation</span>
                <span className="hover:text-white cursor-pointer" onClick={() => onNewDocument('pdf')}>PDF</span>
              </div>
            </footer>
          </div>
        </main>
      )}
    </div>
  );
};
