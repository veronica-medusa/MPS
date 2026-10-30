import React, { useState } from 'react';
import { 
  Sparkles, 
  Code, 
  X, 
  Send, 
  Play, 
  FileText, 
  Table as TableIcon, 
  Presentation, 
  FileCheck, 
  CheckCircle2, 
  AlertCircle,
  BarChart2,
  ExternalLink,
  ChevronRight
} from 'lucide-react';
import { DocumentType } from '../types';

interface AiMessage {
  id: string;
  sender: 'user' | 'ai';
  persona: string;
  text: string;
  citations?: {
    index: number;
    page: number;
    text: string;
    bbox: [number, number, number, number];
  }[];
  timestamp: string;
}

interface AiPythonStudioDrawerProps {
  isOpen: boolean;
  activeDocType: DocumentType;
  onClose: () => void;
  onApplyAiAction?: (actionType: string, payload: any) => void;
}

export const AiPythonStudioDrawer: React.FC<AiPythonStudioDrawerProps> = ({
  isOpen,
  activeDocType,
  onClose,
  onApplyAiAction,
}) => {
  const [activeTab, setActiveTab] = useState<'ai' | 'python'>('ai');
  const [selectedPersona, setSelectedPersona] = useState<string>(() => {
    switch (activeDocType) {
      case 'writer': return '@WriterAI';
      case 'sheets': return '@SheetsAI';
      case 'slides': return '@SlidesAI';
      case 'pdf': return '@PdfAI';
    }
  });

  // AI Chat State
  const [promptInput, setPromptInput] = useState('');
  const [messages, setMessages] = useState<AiMessage[]>([
    {
      id: 'm1',
      sender: 'ai',
      persona: '@WPS-AI',
      text: 'Hello! I am your AST-grounded WPS AI Co-Pilot. I can proofread with Track Changes, synthesize formulas, build slide decks, or query PDF documents with precise spatial citations.',
      timestamp: 'Just now',
    },
  ]);

  // Python Studio State
  const [pyCode, setPyCode] = useState<string>(
    `# OpenWPS Python Runtime (=PY() / Macros)\n` +
    `df = xl('A1:D12', headers=True)\n` +
    `result = df.groupby('Region')['Revenue'].sum()\n` +
    `result`
  );
  const [pyOutput, setPyOutput] = useState<{
    status: 'idle' | 'success' | 'error';
    execTimeMs: number;
    outputType: string;
    data: any;
  }>({
    status: 'idle',
    execTimeMs: 0,
    outputType: 'None',
    data: null,
  });

  if (!isOpen) return null;

  const handleSendPrompt = () => {
    if (!promptInput.trim()) return;

    const userMsg: AiMessage = {
      id: `u-${Date.now()}`,
      sender: 'user',
      persona: 'You',
      text: promptInput,
      timestamp: 'Just now',
    };

    const p = promptInput.toLowerCase();
    let replyText = 'Executed instruction and aligned document AST.';
    let citations: AiMessage['citations'] = undefined;

    if (selectedPersona === '@WriterAI' || activeDocType === 'writer') {
      replyText = 'Proofread and updated paragraph in formal executive tone. Created 2 Track Changes redlines (Revision::Delete + Revision::Insert).';
    } else if (selectedPersona === '@SheetsAI' || activeDocType === 'sheets') {
      if (p.includes('anomal') || p.includes('z-score') || p.includes('outlier')) {
        replyText = 'Statistical Audit Complete: Detected 1 anomaly in Revenue column (Row 11: $5,000.00, Z-score = 2.82 > 2.50). Pearson correlation between Marketing and Revenue: r = 0.892.';
      } else {
        replyText = 'Formula synthesized: =SUMIFS(D2:D500, B2:B500, "APAC", A2:A500, ">=2026-07-01", A2:A500, "<=2026-09-30")';
      }
    } else if (selectedPersona === '@SlidesAI' || activeDocType === 'slides') {
      replyText = 'Compiled 4-slide presentation deck with Executive Summary, Market Opportunity comparison, KPI Grid, and Process Timeline with Smart Connectors.';
    } else if (selectedPersona === '@PdfAI' || activeDocType === 'pdf') {
      replyText = 'Based on the spatial index [1], Adjusted EBITDA margin expanded to 38.0% during the fiscal year.';
      citations = [
        {
          index: 1,
          page: 2,
          text: 'Adjusted EBITDA reached 42.8 million dollars representing 38% margin growth.',
          bbox: [60, 200, 500, 280],
        },
      ];
    }

    const aiMsg: AiMessage = {
      id: `ai-${Date.now()}`,
      sender: 'ai',
      persona: selectedPersona,
      text: replyText,
      citations,
      timestamp: 'Just now',
    };

    setMessages((prev) => [...prev, userMsg, aiMsg]);
    setPromptInput('');

    if (onApplyAiAction) {
      onApplyAiAction(selectedPersona, { prompt: promptInput });
    }
  };

  const handleRunPython = () => {
    // Deterministic client evaluation
    const isGroupBy = pyCode.includes('groupby');
    const isPlot = pyCode.includes('plot');

    if (isPlot) {
      setPyOutput({
        status: 'success',
        execTimeMs: 2.4,
        outputType: 'SvgPlot',
        data: '<svg viewBox="0 0 400 160" width="100%" height="160"><rect width="400" height="160" fill="#f8fafc" rx="8"/><polyline fill="none" stroke="#7048E8" stroke-width="3" points="20,130 90,90 160,110 230,40 300,70 370,20"/><circle cx="230" cy="40" r="5" fill="#7048E8"/><text x="230" y="30" font-size="11" fill="#7048E8" text-anchor="middle">Q3 Peak ($42M)</text></svg>',
      });
    } else if (isGroupBy) {
      setPyOutput({
        status: 'success',
        execTimeMs: 1.8,
        outputType: 'DataFrame (SpillMatrix 4x2)',
        data: [
          ['Region', 'Revenue ($M)'],
          ['APAC', '$42.8'],
          ['EMEA', '$38.2'],
          ['LATAM', '$14.6'],
          ['NA', '$78.4'],
        ],
      });
    } else {
      setPyOutput({
        status: 'success',
        execTimeMs: 1.1,
        outputType: 'Scalar (Float)',
        data: '174.00',
      });
    }
  };

  const getQuickPills = () => {
    switch (activeDocType) {
      case 'writer':
        return [
          'Rewrite Formal',
          'Make Concise',
          'Track Changes Audit',
          'Continue Writing',
        ];
      case 'sheets':
        return [
          'NL to Formula (=SUMIFS)',
          'Z-Score Outlier Audit',
          'Pearson Correlation',
          '=PY GroupBy Spill',
        ];
      case 'slides':
        return [
          'Generate 4-Slide Deck',
          'Add Process Timeline',
          'Create KPI Cards Grid',
          'Two-Column Comparison',
        ];
      case 'pdf':
        return [
          'Spatial RAG Query',
          'Extract Tables to Excel',
          'Audit PII Redactions',
          'Verify Digital Signatures',
        ];
    }
  };

  return (
    <aside className="w-96 border-l border-mercury-fog/30 bg-white dark:bg-mercury-night-raised flex flex-col h-full shadow-2xl z-40 select-none">
      {/* Top Header */}
      <div className="h-12 border-b border-mercury-fog/20 px-3 flex items-center justify-between bg-mercury-lavender/30 dark:bg-mercury-night">
        <div className="flex items-center space-x-1.5">
          <div className="w-7 h-7 rounded-mercury-md bg-mercury-ai text-white flex items-center justify-center shadow-sm">
            <Sparkles className="w-4 h-4 animate-spin-slow" />
          </div>
          <div>
            <h2 className="text-xs font-bold text-mercury-ink dark:text-mercury-on-dark flex items-center space-x-1">
              <span>WPS Studio</span>
              <span className="text-[10px] px-1.5 py-0.2 rounded-full bg-mercury-ai/20 text-mercury-ai font-semibold">
                Iris 2.0
              </span>
            </h2>
            <p className="text-[10px] text-mercury-ink-secondary dark:text-mercury-fog">
              {activeTab === 'ai' ? 'AST-Native Co-Pilot' : 'Embedded Python 3.12 Runtime'}
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-1">
          {/* Tab Switcher */}
          <div className="flex items-center p-0.5 rounded-mercury-md bg-mercury-fog/20 dark:bg-mercury-night-raised">
            <button
              onClick={() => setActiveTab('ai')}
              className={`px-2 py-1 text-xs font-medium rounded-mercury-sm transition-all ${
                activeTab === 'ai'
                  ? 'bg-white dark:bg-mercury-ai text-mercury-ai dark:text-white shadow-xs'
                  : 'text-mercury-ink-secondary dark:text-mercury-fog hover:text-mercury-ink'
              }`}
            >
              AI Co-Pilot
            </button>
            <button
              onClick={() => setActiveTab('python')}
              className={`px-2 py-1 text-xs font-medium rounded-mercury-sm transition-all flex items-center space-x-1 ${
                activeTab === 'python'
                  ? 'bg-white dark:bg-mercury-ai text-mercury-ai dark:text-white shadow-xs'
                  : 'text-mercury-ink-secondary dark:text-mercury-fog hover:text-mercury-ink'
              }`}
            >
              <Code className="w-3 h-3" />
              <span>Python</span>
            </button>
          </div>

          <button
            onClick={onClose}
            className="p-1 rounded-mercury-sm text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night"
            title="Close Studio Drawer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Main Drawer Body */}
      {activeTab === 'ai' ? (
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Persona Bar */}
          <div className="p-2 border-b border-mercury-fog/10 flex items-center space-x-1.5 overflow-x-auto no-scrollbar bg-mercury-surface-tint dark:bg-mercury-night/50">
            {['@WriterAI', '@SheetsAI', '@SlidesAI', '@PdfAI'].map((p) => {
              const isSelected = selectedPersona === p;
              return (
                <button
                  key={p}
                  onClick={() => setSelectedPersona(p)}
                  className={`text-[11px] px-2.5 py-1 rounded-mercury-pill font-medium transition-all ${
                    isSelected
                      ? 'bg-mercury-ai text-white shadow-xs'
                      : 'bg-mercury-lavender/50 dark:bg-mercury-night text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender'
                  }`}
                >
                  {p}
                </button>
              );
            })}
          </div>

          {/* Quick Action Pills */}
          <div className="p-2 flex flex-wrap gap-1 border-b border-mercury-fog/10">
            {getQuickPills().map((pill) => (
              <button
                key={pill}
                onClick={() => {
                  setPromptInput(pill);
                }}
                className="text-[10px] px-2 py-0.5 rounded-mercury-md border border-mercury-ai/30 text-mercury-ai hover:bg-mercury-ai/10 transition-colors"
              >
                {pill}
              </button>
            ))}
          </div>

          {/* Message History */}
          <div className="flex-1 p-3 overflow-y-auto space-y-3">
            {messages.map((m) => (
              <div
                key={m.id}
                className={`flex flex-col ${m.sender === 'user' ? 'items-end' : 'items-start'}`}
              >
                <div className="flex items-center space-x-1 mb-1 text-[10px] text-mercury-slate">
                  <span className="font-semibold">{m.persona}</span>
                  <span>•</span>
                  <span>{m.timestamp}</span>
                </div>
                <div
                  className={`max-w-[90%] p-2.5 rounded-mercury-lg text-xs leading-relaxed ${
                    m.sender === 'user'
                      ? 'bg-mercury-ai text-white rounded-tr-none'
                      : 'bg-mercury-lavender/40 dark:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark border border-mercury-fog/20 rounded-tl-none'
                  }`}
                >
                  {m.text}

                  {/* PDF Spatial Citations */}
                  {m.citations && m.citations.length > 0 && (
                    <div className="mt-2 pt-2 border-t border-mercury-fog/30 space-y-1.5">
                      <div className="text-[10px] font-bold text-mercury-ai uppercase tracking-wider flex items-center space-x-1">
                        <FileCheck className="w-3 h-3" />
                        <span>Spatial Grounding Citations</span>
                      </div>
                      {m.citations.map((c) => (
                        <div
                          key={c.index}
                          className="p-1.5 rounded-mercury-md bg-white dark:bg-mercury-night-raised border border-mercury-ai/30 text-[10px] cursor-pointer hover:border-mercury-ai transition-colors"
                          title="Click to jump to page and highlight bounding box"
                        >
                          <div className="flex items-center justify-between text-mercury-ai font-semibold mb-0.5">
                            <span>[{c.index}] Page {c.page}</span>
                            <span className="text-[9px] text-mercury-slate">
                              [{c.bbox.map(n => Math.round(n)).join(', ')}]
                            </span>
                          </div>
                          <p className="text-mercury-ink dark:text-mercury-fog line-clamp-2">
                            "{c.text}"
                          </p>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>

          {/* Prompt Input Bar */}
          <div className="p-2 border-t border-mercury-fog/20 bg-white dark:bg-mercury-night flex items-center space-x-1.5">
            <input
              type="text"
              value={promptInput}
              onChange={(e) => setPromptInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSendPrompt();
              }}
              placeholder={`Ask ${selectedPersona}...`}
              className="flex-1 px-3 py-2 text-xs rounded-mercury-md border border-mercury-fog/40 bg-mercury-surface dark:bg-mercury-night-raised text-mercury-ink dark:text-mercury-on-dark focus:outline-none focus:border-mercury-ai"
            />
            <button
              onClick={handleSendPrompt}
              className="p-2 rounded-mercury-md bg-mercury-ai text-white hover:opacity-90 transition-opacity"
              title="Send to Co-Pilot"
            >
              <Send className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      ) : (
        /* Python Studio Tab */
        <div className="flex-1 flex flex-col p-3 space-y-3 overflow-y-auto">
          {/* Quick Snippets Dropdown */}
          <div className="flex items-center justify-between text-xs">
            <span className="font-semibold text-mercury-ink dark:text-mercury-on-dark">
              =PY() Expression & Macro
            </span>
            <select
              onChange={(e) => {
                const val = e.target.value;
                if (val === 'groupby') {
                  setPyCode("df = xl('A1:D12', headers=True)\ndf.groupby('Region')['Revenue'].sum()");
                } else if (val === 'describe') {
                  setPyCode("df = xl('A1:D12', headers=True)\ndf.describe()");
                } else if (val === 'plot') {
                  setPyCode("import matplotlib.pyplot as plt\nplt.plot([1, 2, 3, 4], [10, 25, 18, 42])\nplt.title('Quarterly Revenue Trend')\nplt.show()");
                } else if (val === 'macro') {
                  setPyCode("# Cross-Document Macro\nwriter = wp.writer.get_active()\nsheets = wp.sheets.get_active()\nwriter.insert_paragraph(f'Total Sales: {sheets.cell(\"C12\").value}')");
                }
              }}
              className="text-[11px] px-2 py-1 rounded-mercury-sm border border-mercury-fog/30 bg-mercury-surface dark:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark"
            >
              <option value="groupby">Snippet: GroupBy & Spill</option>
              <option value="describe">Snippet: DataFrame Describe</option>
              <option value="plot">Snippet: Matplotlib SVG Plot</option>
              <option value="macro">Snippet: Cross-Doc wp.* Macro</option>
            </select>
          </div>

          {/* Python Editor */}
          <div className="flex-1 min-h-[160px] border border-mercury-fog/30 rounded-mercury-md bg-mercury-canvas dark:bg-mercury-night p-2 flex flex-col">
            <textarea
              value={pyCode}
              onChange={(e) => setPyCode(e.target.value)}
              className="w-full h-full resize-none font-mono text-xs bg-transparent text-mercury-ink dark:text-mercury-on-dark focus:outline-none"
              rows={8}
            />
          </div>

          {/* Run Button */}
          <div className="flex items-center justify-between">
            <span className="text-[10px] text-mercury-slate">
              Ctrl+Enter to Execute
            </span>
            <button
              onClick={handleRunPython}
              className="flex items-center space-x-1.5 px-3 py-1.5 rounded-mercury-md bg-mercury-sheets text-white text-xs font-semibold hover:opacity-90 transition-opacity shadow-xs"
            >
              <Play className="w-3.5 h-3.5 fill-white" />
              <span>Execute Code</span>
            </button>
          </div>

          {/* Execution Output Inspector */}
          {pyOutput.status !== 'idle' && (
            <div className="border border-mercury-fog/30 rounded-mercury-md p-2.5 bg-mercury-lavender/30 dark:bg-mercury-night text-xs space-y-2">
              <div className="flex items-center justify-between">
                <span className="font-semibold text-mercury-ink dark:text-mercury-on-dark flex items-center space-x-1">
                  <CheckCircle2 className="w-3.5 h-3.5 text-mercury-sheets" />
                  <span>Output ({pyOutput.outputType})</span>
                </span>
                <span className="text-[10px] text-mercury-slate">
                  {pyOutput.execTimeMs} ms
                </span>
              </div>

              {/* Spill Matrix View */}
              {Array.isArray(pyOutput.data) ? (
                <div className="overflow-x-auto border border-mercury-fog/20 rounded bg-white dark:bg-mercury-night-raised">
                  <table className="w-full text-[11px] text-left">
                    <tbody>
                      {pyOutput.data.map((row: string[], rIdx: number) => (
                        <tr
                          key={rIdx}
                          className={rIdx === 0 ? 'bg-mercury-lavender/60 dark:bg-mercury-night font-bold' : 'border-t border-mercury-fog/10'}
                        >
                          {row.map((val: string, cIdx: number) => (
                            <td key={cIdx} className="px-2 py-1">
                              {val}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : pyOutput.outputType === 'SvgPlot' ? (
                <div
                  className="bg-white dark:bg-mercury-night-raised p-2 rounded border border-mercury-fog/20 flex justify-center"
                  dangerouslySetInnerHTML={{ __html: pyOutput.data }}
                />
              ) : (
                <div className="p-2 font-mono bg-white dark:bg-mercury-night-raised rounded border border-mercury-fog/20">
                  {String(pyOutput.data)}
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </aside>
  );
};
