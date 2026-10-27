import React, { useState } from 'react';
import { 
  Sparkles, 
  Terminal, 
  Settings, 
  X, 
  Send, 
  Play, 
  CheckCircle, 
  Copy, 
  Database,
  Table as TableIcon
} from 'lucide-react';
import { DocumentTab } from '../types';

interface RightInspectorProps {
  isOpen: boolean;
  onClose: () => void;
  tab: DocumentTab;
  initialTab?: 'ai' | 'python' | 'properties';
}

export const RightInspector: React.FC<RightInspectorProps> = ({
  isOpen,
  onClose,
  tab,
  initialTab = 'ai',
}) => {
  const [activePane, setActivePane] = useState<'ai' | 'python' | 'properties'>(initialTab);
  const [aiPrompt, setAiPrompt] = useState('');
  const [aiChat, setAiChat] = useState<Array<{ sender: 'user' | 'ai'; text: string; action?: string }>>([
    {
      sender: 'ai',
      text: `Hello! I am your WPS AI Co-Pilot grounded in the ${tab.type.toUpperCase()} document AST. How can I assist you with this file?`,
    },
  ]);

  const [pyCode, setPyCode] = useState(
    `# OpenWPS Embedded Python Runtime (=PY())\nimport pandas as pd\nimport numpy as np\n\n# Load range from active sheet\ndf = pd.DataFrame({\n    'Region': ['North America', 'EMEA', 'APAC'],\n    'Revenue': [1420500, 985200, 1140000]\n})\ndf['Margin'] = df['Revenue'] * 0.28\nprint(df.describe())`
  );
  const [pyOutput, setPyOutput] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleSendAi = () => {
    if (!aiPrompt.trim()) return;
    const userText = aiPrompt;
    setAiPrompt('');
    setAiChat((prev) => [...prev, { sender: 'user', text: userText }]);

    setTimeout(() => {
      let reply = '';
      let action = undefined;
      if (tab.type === 'sheets') {
        reply = `I analyzed your spreadsheet request. I recommend using dynamic array formula: =XLOOKUP(A2, Products!A:A, Products!C:C, "Not Found"). Would you like to insert it into cell B2?`;
        action = 'Insert Formula in B2';
      } else if (tab.type === 'writer') {
        reply = `I can rewrite the selected paragraph into a concise, professional executive tone. Here is the revised draft.`;
        action = 'Replace Selection';
      } else if (tab.type === 'pdf') {
        reply = `I found 3 relevant citations in this PDF. Click below to highlight and jump to [Page 1, BBox (50, 120, 500, 180)].`;
        action = 'Jump to Citation';
      } else {
        reply = `I generated a 4-slide outline based on your prompt using the active slide master theme.`;
        action = 'Insert Slides';
      }

      setAiChat((prev) => [...prev, { sender: 'ai', text: reply, action }]);
    }, 600);
  };

  const handleRunPython = () => {
    setPyOutput(
      `Executing in OpenWPS embedded Python engine...\nDataFrame Output:\n         Revenue    Margin\ncount       3.00      3.00\nmean  1181900.00 330932.00\nstd    220803.00  61824.84\nmin    985200.00 275856.00\nmax   1420500.00 397740.00\n[Result successfully spilled to range D2:E5]`
    );
  };

  return (
    <aside className="w-80 md:w-96 bg-white dark:bg-mercury-night-raised border-l border-mercury-fog/30 flex flex-col z-30 transition-all select-none">
      {/* Header Tabs */}
      <div className="flex items-center justify-between px-3 h-10 border-b border-mercury-fog/20 text-xs">
        <div className="flex space-x-1">
          <button
            onClick={() => setActivePane('ai')}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm font-medium transition-colors ${
              activePane === 'ai'
                ? 'bg-mercury-ai/10 text-mercury-ai'
                : 'text-mercury-ink-secondary dark:text-mercury-fog hover:text-mercury-ink'
            }`}
          >
            <Sparkles className="w-3.5 h-3.5" />
            <span>WPS AI</span>
          </button>
          <button
            onClick={() => setActivePane('python')}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm font-medium transition-colors ${
              activePane === 'python'
                ? 'bg-mercury-primary/10 text-mercury-primary'
                : 'text-mercury-ink-secondary dark:text-mercury-fog hover:text-mercury-ink'
            }`}
          >
            <Terminal className="w-3.5 h-3.5" />
            <span>Python (=PY)</span>
          </button>
          <button
            onClick={() => setActivePane('properties')}
            className={`flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm font-medium transition-colors ${
              activePane === 'properties'
                ? 'bg-mercury-lavender dark:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark'
                : 'text-mercury-ink-secondary dark:text-mercury-fog hover:text-mercury-ink'
            }`}
          >
            <Settings className="w-3.5 h-3.5" />
            <span>Properties</span>
          </button>
        </div>

        <button
          onClick={onClose}
          className="p-1 rounded-mercury-sm text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night"
        >
          <X className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Pane Body */}
      <div className="flex-1 overflow-y-auto p-3 text-xs">
        {/* ===================== AI PANE ===================== */}
        {activePane === 'ai' && (
          <div className="flex flex-col h-full justify-between">
            <div className="space-y-3 overflow-y-auto pr-1">
              {aiChat.map((msg, i) => (
                <div
                  key={i}
                  className={`p-2.5 rounded-mercury-md ${
                    msg.sender === 'user'
                      ? 'bg-mercury-primary text-white ml-6'
                      : 'bg-mercury-surface-tint dark:bg-mercury-night border border-mercury-fog/30 text-mercury-ink dark:text-mercury-on-dark mr-4'
                  }`}
                >
                  <p className="leading-relaxed">{msg.text}</p>
                  {msg.action && (
                    <button
                      onClick={() => alert(`Applied AI Action: ${msg.action}`)}
                      className="mt-2 w-full flex items-center justify-center space-x-1 py-1 px-2 rounded-mercury-sm bg-mercury-ai text-white hover:bg-mercury-ai/90 font-medium text-[11px] shadow-sm"
                    >
                      <Sparkles className="w-3 h-3" />
                      <span>{msg.action}</span>
                    </button>
                  )}
                </div>
              ))}
            </div>

            {/* AI Input */}
            <div className="mt-3 pt-2 border-t border-mercury-fog/20">
              <div className="relative flex items-center">
                <input
                  type="text"
                  value={aiPrompt}
                  onChange={(e) => setAiPrompt(e.target.value)}
                  onKeyDown={(e) => e.key === 'Enter' && handleSendAi()}
                  placeholder="Ask WPS AI to write, format, or calculate..."
                  className="w-full pl-3 pr-8 py-2 rounded-mercury-pill border border-mercury-fog/40 bg-white dark:bg-mercury-night text-xs text-mercury-ink dark:text-mercury-on-dark outline-none focus:border-mercury-ai"
                />
                <button
                  onClick={handleSendAi}
                  className="absolute right-1.5 p-1 rounded-full bg-mercury-ai text-white hover:bg-mercury-ai/90"
                >
                  <Send className="w-3 h-3" />
                </button>
              </div>
            </div>
          </div>
        )}

        {/* ===================== PYTHON PANE ===================== */}
        {activePane === 'python' && (
          <div className="flex flex-col h-full space-y-3">
            <div className="flex items-center justify-between">
              <span className="font-semibold text-mercury-ink dark:text-mercury-on-dark flex items-center space-x-1">
                <Terminal className="w-3.5 h-3.5 text-mercury-primary" />
                <span>Python Interactive Bridge</span>
              </span>
              <button
                onClick={handleRunPython}
                className="flex items-center space-x-1 px-2.5 py-1 rounded-mercury-sm bg-mercury-sheets text-white hover:bg-mercury-sheets/90 font-medium"
              >
                <Play className="w-3 h-3" />
                <span>Run Code</span>
              </button>
            </div>

            <textarea
              value={pyCode}
              onChange={(e) => setPyCode(e.target.value)}
              className="w-full h-48 p-2 font-mono text-[11px] bg-mercury-night text-mercury-on-dark rounded-mercury-md border border-mercury-fog/30 outline-none resize-none"
              spellCheck={false}
            />

            {pyOutput && (
              <div className="flex-1 p-2 bg-mercury-surface-tint dark:bg-mercury-night rounded-mercury-md border border-mercury-fog/30 font-mono text-[10px] whitespace-pre-wrap overflow-y-auto text-mercury-ink dark:text-mercury-on-dark">
                {pyOutput}
              </div>
            )}
          </div>
        )}

        {/* ===================== PROPERTIES PANE ===================== */}
        {activePane === 'properties' && (
          <div className="space-y-4">
            <div>
              <h4 className="font-semibold text-mercury-ink dark:text-mercury-on-dark mb-1">Document Metrics</h4>
              <div className="space-y-1.5 text-mercury-ink-secondary dark:text-mercury-fog">
                <div className="flex justify-between">
                  <span>Type:</span>
                  <span className="font-medium text-mercury-ink dark:text-mercury-on-dark uppercase">
                    {tab.type}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Status:</span>
                  <span className="font-medium text-mercury-ink dark:text-mercury-on-dark">
                    {tab.isDirty ? 'Unsaved Changes' : 'All Changes Saved'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Zoom Level:</span>
                  <span className="tabular-nums font-mono">{tab.zoom}%</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
