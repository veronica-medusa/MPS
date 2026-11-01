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
  ArrowRight
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
  const [activeNav, setActiveNav] = useState<'home' | 'new' | 'open' | 'templates' | 'history'>('home');
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<string>('all');

  // Filtered recent tabs
  const filteredTabs = recentTabs.filter((tab) => {
    const matchesQuery = tab.title.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesType = filterType === 'all' || tab.type === filterType;
    return matchesQuery && matchesType;
  });

  const getDocTypeIcon = (type: DocumentType) => {
    switch (type) {
      case 'writer':
        return <FileText className="w-5 h-5 text-blue-600" />;
      case 'sheets':
        return <Table className="w-5 h-5 text-emerald-600" />;
      case 'slides':
        return <Presentation className="w-5 h-5 text-orange-600" />;
      case 'pdf':
        return <FileCheck className="w-5 h-5 text-rose-600" />;
    }
  };

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
            <div className="w-7 h-7 rounded-lg bg-mercury-primary flex items-center justify-center text-white font-bold text-sm shadow-sm">
              W
            </div>
            <div>
              <div className="font-semibold text-sm text-mercury-ink dark:text-mercury-on-dark tracking-tight">WPS Office</div>
              <div className="text-[11px] text-mercury-slate">Home Hub Workspace</div>
            </div>
          </div>
        </div>

        <nav className="flex flex-col gap-1">
          <button
            onClick={() => setActiveNav('home')}
            className={`wps-home-nav-item ${activeNav === 'home' ? 'is-active' : ''}`}
          >
            <Home className="w-4 h-4" />
            <span>Home</span>
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

      {/* Main Home Workspace Content */}
      <main className="wps-home-main">
        {/* Top Quick-Create Row (4 Clear Cards) */}
        <section>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-mercury-ink dark:text-mercury-on-dark tracking-tight">
              Create New
            </h2>
            <span className="text-xs text-mercury-slate">Select a blank module to start editing immediately</span>
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
            <h2 className="text-lg font-semibold text-mercury-ink dark:text-mercury-on-dark tracking-tight">
              Recommended Templates
            </h2>
            <span className="text-xs text-mercury-slate">One-click professionally populated files</span>
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
            <h2 className="text-lg font-semibold text-mercury-ink dark:text-mercury-on-dark tracking-tight">
              Recent Files
            </h2>

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
    </div>
  );
};
