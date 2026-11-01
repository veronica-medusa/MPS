import React, { useState } from 'react';
import {
  Sparkles,
  History,
  X,
  Send,
  RotateCcw,
  CheckCircle,
  Clock,
  ArrowRight,
  FileText,
  Table,
  Presentation,
  FileCheck
} from 'lucide-react';
import { DocumentType, VersionSnapshot, AiChatMessage } from '../types';

interface RightAssistantDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  activeDrawerTab: 'ai' | 'history';
  onTabChange: (tab: 'ai' | 'history') => void;
  docType: DocumentType;
  onApplyAiAction?: (actionTag: string, payload?: any) => void;
  onRestoreSnapshot?: (snapshotId: string) => void;
}

export const RightAssistantDrawer: React.FC<RightAssistantDrawerProps> = ({
  isOpen,
  onClose,
  activeDrawerTab,
  onTabChange,
  docType,
  onApplyAiAction,
  onRestoreSnapshot,
}) => {
  // AI Chat Messages
  const [messages, setMessages] = useState<AiChatMessage[]>([
    {
      id: 'msg-1',
      role: 'assistant',
      content: `Hello! I am your WPS AI Assistant. I can help refine your ${
        docType === 'writer'
          ? 'document text and styling'
          : docType === 'sheets'
          ? 'formulas, calculations, and data models'
          : docType === 'slides'
          ? 'slide presentations and visual themes'
          : 'PDF document analysis, citations, and summaries'
      }. How can I assist you today?`,
      timestamp: 'Just now',
    },
  ]);
  const [inputPrompt, setInputPrompt] = useState<string>('');

  // Version History Stack
  const [snapshots, setSnapshots] = useState<VersionSnapshot[]>([
    {
      id: 'v-3',
      timestamp: 'Today, 2:40 PM',
      author: 'Alex Morgan',
      description: 'Applied Track Changes revisions and formatted headers',
      type: 'manual',
      targetDocTitle: 'Current Document',
    },
    {
      id: 'v-2',
      timestamp: 'Today, 1:15 PM',
      author: 'WPS AI Assistant',
      description: 'Auto-saved before applying executive summary formatting',
      type: 'ai',
      targetDocTitle: 'Current Document',
    },
    {
      id: 'v-1',
      timestamp: 'Today, 11:00 AM',
      author: 'System AutoSave',
      description: 'Initial document creation from template',
      type: 'auto',
      targetDocTitle: 'Current Document',
    },
  ]);

  if (!isOpen) return null;

  // Handle Send AI Prompt
  const handleSendMessage = (text?: string) => {
    const promptToSend = text || inputPrompt;
    if (!promptToSend.trim()) return;

    const userMsg: AiChatMessage = {
      id: `msg-${Date.now()}`,
      role: 'user',
      content: promptToSend,
      timestamp: 'Just now',
    };

    setMessages((prev) => [...prev, userMsg]);
    setInputPrompt('');

    // Generate contextual response
    setTimeout(() => {
      let reply = '';
      let actionTag = '';
      let citationPage = undefined;

      if (docType === 'writer') {
        reply = 'I reviewed the selected paragraph. I recommend restructuring the executive summary to emphasize ROI and cross-device speed.';
        actionTag = 'apply_track_changes';
      } else if (docType === 'sheets') {
        reply = 'Formula suggested: =SUM(B2:D2) to compute the total regional sales for APAC across Q1-Q3.';
        actionTag = 'insert_formula';
      } else if (docType === 'slides') {
        reply = 'I generated a 5-slide pitch outline with Problem, Solution, Market Opportunity, Roadmap, and Financial Projections.';
        actionTag = 'generate_slides';
      } else {
        reply = 'Located termination clause in Section 3 (Page 1). Early buyout fee is $45,000 as agreed upon.';
        citationPage = 1;
      }

      const aiMsg: AiChatMessage = {
        id: `msg-${Date.now() + 1}`,
        role: 'assistant',
        content: reply,
        timestamp: 'Just now',
        actionTag,
        citationPage,
      };

      setMessages((prev) => [...prev, aiMsg]);
    }, 400);
  };

  return (
    <aside className="wps-right-drawer">
      {/* Drawer Header */}
      <div className="wps-drawer-header">
        <div className="flex items-center gap-1 bg-gray-100 dark:bg-mercury-night-raised p-0.5 rounded-full text-xs">
          <button
            onClick={() => onTabChange('ai')}
            className={`px-3 py-1 rounded-full flex items-center gap-1.5 transition-all ${
              activeDrawerTab === 'ai' ? 'bg-white font-semibold text-mercury-primary shadow-sm' : 'text-gray-500'
            }`}
          >
            <Sparkles className="w-3.5 h-3.5 text-amber-500" />
            <span>WPS AI</span>
          </button>
          <button
            onClick={() => onTabChange('history')}
            className={`px-3 py-1 rounded-full flex items-center gap-1.5 transition-all ${
              activeDrawerTab === 'history' ? 'bg-white font-semibold text-mercury-primary shadow-sm' : 'text-gray-500'
            }`}
          >
            <History className="w-3.5 h-3.5" />
            <span>History</span>
          </button>
        </div>

        <button onClick={onClose} className="p-1 rounded hover:bg-gray-200 text-gray-500">
          <X className="w-4 h-4" />
        </button>
      </div>

      {/* ─── TAB 1: WPS AI ASSISTANT ─── */}
      {activeDrawerTab === 'ai' && (
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Quick Contextual Action Chips */}
          <div className="p-3 border-b border-gray-200 bg-mercury-surface-tint flex flex-wrap gap-1.5">
            {docType === 'writer' && (
              <>
                <button
                  onClick={() => handleSendMessage('Improve writing style')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  ✨ Improve Writing
                </button>
                <button
                  onClick={() => handleSendMessage('Fix grammar and spelling')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Fix Grammar
                </button>
                <button
                  onClick={() => handleSendMessage('Summarize document')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Summarize
                </button>
                <button
                  onClick={() => handleSendMessage('Make shorter')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Make Shorter
                </button>
              </>
            )}

            {docType === 'sheets' && (
              <>
                <button
                  onClick={() => handleSendMessage('Calculate total revenue for APAC')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Calculate Total Revenue
                </button>
                <button
                  onClick={() => handleSendMessage('Highlight outliers in sales')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Highlight Outliers
                </button>
                <button
                  onClick={() => handleSendMessage('Explain formula in active cell')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Explain Formula
                </button>
              </>
            )}

            {docType === 'slides' && (
              <>
                <button
                  onClick={() => handleSendMessage('Create a 5-slide pitch deck for a clean energy startup')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Generate 5-Slide Deck
                </button>
                <button
                  onClick={() => handleSendMessage('Improve slide headline and bullet points')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Improve Headline
                </button>
              </>
            )}

            {docType === 'pdf' && (
              <>
                <button
                  onClick={() => handleSendMessage('Summarize key contract terms')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Summarize Contract
                </button>
                <button
                  onClick={() => handleSendMessage('Find early buyout or termination clauses')}
                  className="px-2.5 py-1 rounded-full bg-white border border-mercury-fog/40 text-[11px] hover:border-mercury-primary hover:text-mercury-primary transition-colors"
                >
                  Find Buyout Clause
                </button>
              </>
            )}
          </div>

          {/* Chat Messages Log */}
          <div className="flex-1 p-3 overflow-y-auto flex flex-col gap-3">
            {messages.map((m) => {
              const isAssistant = m.role === 'assistant';

              return (
                <div
                  key={m.id}
                  className={`flex flex-col gap-1 text-xs max-w-[90%] ${
                    isAssistant ? 'self-start' : 'self-end'
                  }`}
                >
                  <div
                    className={`p-3 rounded-mercury-md leading-relaxed ${
                      isAssistant
                        ? 'bg-mercury-surface-tint border border-mercury-fog/40 text-mercury-ink'
                        : 'bg-mercury-primary text-white'
                    }`}
                  >
                    {m.content}

                    {/* Interactive Citation badge */}
                    {m.citationPage && (
                      <div className="mt-2 pt-2 border-t border-gray-200">
                        <button
                          onClick={() => alert(`Navigating to Page ${m.citationPage} and highlighting paragraph.`)}
                          className="px-2 py-0.5 rounded bg-amber-100 text-amber-800 font-semibold text-[10px] hover:bg-amber-200 inline-flex items-center gap-1"
                        >
                          <span>[Page {m.citationPage} Citation]</span>
                          <ArrowRight className="w-2.5 h-2.5" />
                        </button>
                      </div>
                    )}

                    {/* Quick Action Button */}
                    {m.actionTag && (
                      <div className="mt-2 pt-2 border-t border-gray-200">
                        <button
                          onClick={() => {
                            if (onApplyAiAction) onApplyAiAction(m.actionTag!);
                            alert('AI action applied directly to document.');
                          }}
                          className="button-primary h-7 px-3 text-[11px] w-full"
                        >
                          Apply to Document
                        </button>
                      </div>
                    )}
                  </div>
                  <span className="text-[10px] text-mercury-slate px-1">{m.timestamp}</span>
                </div>
              );
            })}
          </div>

          {/* Prompt Input Box */}
          <div className="p-3 border-t border-gray-200 bg-white flex items-center gap-2">
            <input
              type="text"
              placeholder="Ask WPS AI anything..."
              value={inputPrompt}
              onChange={(e) => setInputPrompt(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSendMessage();
              }}
              className="flex-1 h-9 px-3 text-xs border border-gray-300 rounded-full outline-none focus:border-mercury-primary"
            />
            <button
              onClick={() => handleSendMessage()}
              className="w-9 h-9 rounded-full bg-mercury-primary text-white flex items-center justify-center hover:bg-indigo-600 transition-colors flex-shrink-0"
              title="Send Prompt"
            >
              <Send className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      )}

      {/* ─── TAB 2: VERSION HISTORY ─── */}
      {activeDrawerTab === 'history' && (
        <div className="flex-1 p-4 overflow-y-auto flex flex-col gap-4">
          <div className="text-xs text-mercury-slate">
            All document modifications and AI passes are automatically tracked with complete version rollback.
          </div>

          <div className="mercury-timeline">
            <div className="mercury-timeline-stem" />

            {snapshots.map((snap) => (
              <div key={snap.id} className="mercury-timeline-node">
                <div className="mercury-timeline-dot" />

                <div className="flex-1 flex flex-col gap-1 p-2.5 rounded-mercury-sm bg-mercury-surface-tint border border-mercury-fog/30 text-xs">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-mercury-ink">{snap.author}</span>
                    <span className="text-[10px] text-mercury-slate">{snap.timestamp}</span>
                  </div>

                  <p className="text-gray-600 text-[11px] leading-relaxed">{snap.description}</p>

                  <div className="flex items-center justify-between mt-2 pt-1.5 border-t border-gray-200">
                    <span className="text-[10px] uppercase font-semibold text-gray-500">
                      {snap.type}
                    </span>
                    <button
                      onClick={() => {
                        if (onRestoreSnapshot) onRestoreSnapshot(snap.id);
                        alert(`Restored document to version from ${snap.timestamp}`);
                      }}
                      className="px-2 py-0.5 rounded bg-mercury-lavender text-mercury-primary text-[10px] font-semibold hover:bg-indigo-100 flex items-center gap-1"
                    >
                      <RotateCcw className="w-2.5 h-2.5" />
                      <span>Restore</span>
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </aside>
  );
};
