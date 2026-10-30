import React, { useState, useEffect, useRef } from 'react';
import { Sparkles, ArrowRight, CornerDownLeft, X } from 'lucide-react';
import { DocumentType } from '../types';

interface FloatingAiBarProps {
  isOpen: boolean;
  docType: DocumentType;
  onClose: () => void;
  onSubmit: (prompt: string) => void;
}

export const FloatingAiBar: React.FC<FloatingAiBarProps> = ({
  isOpen,
  docType,
  onClose,
  onSubmit,
}) => {
  const [prompt, setPrompt] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const getPlaceholder = () => {
    switch (docType) {
      case 'writer': return 'Ask AI to rewrite, summarize, or proofread (e.g., /formal)...';
      case 'sheets': return 'Generate formula or python expression (e.g., /sumifs)...';
      case 'slides': return 'Generate slides or design layout (e.g., /timeline)...';
      case 'pdf': return 'Ask anything grounded in this PDF with spatial citations...';
    }
  };

  const handleSubmit = () => {
    if (!prompt.trim()) return;
    onSubmit(prompt);
    setPrompt('');
    onClose();
  };

  return (
    <div className="fixed bottom-12 left-1/2 -translate-x-1/2 w-[540px] z-50 animate-in fade-in slide-in-from-bottom-3 duration-200">
      <div className="bg-white dark:bg-mercury-night-raised border-2 border-mercury-ai rounded-mercury-xl shadow-2xl p-2.5 flex items-center space-x-2 text-xs">
        <div className="w-7 h-7 rounded-full bg-mercury-ai/10 text-mercury-ai flex items-center justify-center shrink-0">
          <Sparkles className="w-4 h-4 animate-spin-slow" />
        </div>

        <input
          ref={inputRef}
          type="text"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleSubmit();
            if (e.key === 'Escape') onClose();
          }}
          placeholder={getPlaceholder()}
          className="flex-1 bg-transparent text-mercury-ink dark:text-mercury-on-dark focus:outline-none text-xs"
        />

        <div className="flex items-center space-x-1 shrink-0">
          <button
            onClick={handleSubmit}
            className="p-1.5 rounded-mercury-md bg-mercury-ai text-white hover:opacity-90 transition-opacity"
            title="Execute (Enter)"
          >
            <CornerDownLeft className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={onClose}
            className="p-1.5 rounded-mercury-md text-mercury-ink-secondary dark:text-mercury-fog hover:bg-mercury-lavender dark:hover:bg-mercury-night transition-colors"
            title="Dismiss (Esc)"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
};
