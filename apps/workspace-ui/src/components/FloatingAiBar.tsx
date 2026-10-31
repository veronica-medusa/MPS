import React, { useState, useEffect } from 'react';
import { Sparkles, CornerDownLeft, X } from 'lucide-react';
import { DocumentType } from '../types';
import { MercuryInputCapsule } from './mercury/MercuryInputCapsule';

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

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const getPlaceholder = () => {
    switch (docType) {
      case 'writer':
        return 'Ask WPS AI to rewrite, summarize, or proofread (e.g., /formal)...';
      case 'sheets':
        return 'Generate formula or python expression (e.g., =PY() or /sumifs)...';
      case 'slides':
        return 'Generate slides or design layout (e.g., /timeline)...';
      case 'pdf':
        return 'Ask anything grounded in this PDF with spatial citations...';
    }
  };

  const handleSubmit = (val: string) => {
    if (!val.trim()) return;
    onSubmit(val);
    setPrompt('');
    onClose();
  };

  return (
    <div className="fixed bottom-12 left-1/2 -translate-x-1/2 w-[600px] z-50 animate-in fade-in slide-in-from-bottom-3 duration-200">
      <div className="relative">
        <MercuryInputCapsule
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onSubmit={handleSubmit}
          placeholder={getPlaceholder()}
          leadingIcon={<Sparkles className="w-4 h-4 text-[#5266eb]" />}
          actionLabel="Execute"
          actionIcon={<CornerDownLeft className="w-3.5 h-3.5" />}
          autoFocus
          className="shadow-2xl border-[#5266eb]/30"
        />

        {/* Floating ESC Pill */}
        <button
          onClick={onClose}
          className="absolute -top-3 -right-3 w-6 h-6 rounded-full bg-[#171721] text-[#ededf3] hover:text-white flex items-center justify-center border border-[rgba(175,178,206,0.3)] shadow-md text-[10px] transition"
          title="Dismiss (Esc)"
        >
          <X className="w-3 h-3" />
        </button>
      </div>
    </div>
  );
};
