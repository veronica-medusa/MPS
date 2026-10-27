import React, { useEffect, useState } from 'react';
import { Search, Sparkles, FileText, Table, Presentation, FileCheck, Wand2, Terminal, Hash } from 'lucide-react';
import { CommandItem } from '../types';

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: CommandItem[];
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({ isOpen, onClose, commands }) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        if (isOpen) onClose();
      } else if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const filteredCommands = commands.filter(
    (c) =>
      c.title.toLowerCase().includes(query.toLowerCase()) ||
      c.category.toLowerCase().includes(query.toLowerCase())
  );

  const handleSelect = (index: number) => {
    const cmd = filteredCommands[index];
    if (cmd) {
      cmd.action();
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-start justify-center pt-24 select-none">
      <div
        className="w-full max-w-xl bg-white dark:bg-mercury-night-raised border border-mercury-fog/30 rounded-mercury-lg shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-100"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input */}
        <div className="flex items-center px-4 py-3 border-b border-mercury-fog/20">
          <Search className="w-4 h-4 text-mercury-ink-secondary dark:text-mercury-fog mr-3" />
          <input
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={(e) => {
              if (e.key === 'ArrowDown') {
                e.preventDefault();
                setSelectedIndex((prev) => (prev + 1) % Math.max(1, filteredCommands.length));
              } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                setSelectedIndex((prev) => (prev - 1 + filteredCommands.length) % Math.max(1, filteredCommands.length));
              } else if (e.key === 'Enter') {
                e.preventDefault();
                handleSelect(selectedIndex);
              }
            }}
            placeholder="Search all commands, WPS Smart Tools, formulas, or AI actions... (Esc to close)"
            className="w-full bg-transparent text-sm text-mercury-ink dark:text-mercury-on-dark placeholder-mercury-slate outline-none"
            autoFocus
          />
          <kbd className="text-[10px] px-1.5 py-0.5 rounded bg-mercury-lavender dark:bg-mercury-night text-mercury-ink-secondary dark:text-mercury-fog font-mono border border-mercury-fog/30">
            ESC
          </kbd>
        </div>

        {/* Results List */}
        <div className="max-h-80 overflow-y-auto p-2 space-y-1">
          {filteredCommands.length === 0 ? (
            <div className="py-8 text-center text-xs text-mercury-slate">
              No matching commands or actions found.
            </div>
          ) : (
            filteredCommands.map((cmd, idx) => {
              const isSelected = idx === selectedIndex;
              return (
                <div
                  key={cmd.id}
                  onClick={() => handleSelect(idx)}
                  className={`flex items-center justify-between px-3 py-2 rounded-mercury-md text-xs cursor-pointer transition-colors ${
                    isSelected
                      ? 'bg-mercury-primary text-white font-medium'
                      : 'hover:bg-mercury-lavender/60 dark:hover:bg-mercury-night text-mercury-ink dark:text-mercury-on-dark'
                  }`}
                >
                  <div className="flex items-center space-x-2.5">
                    <Hash className="w-3.5 h-3.5 opacity-60" />
                    <span>{cmd.title}</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span
                      className={`text-[10px] px-2 py-0.5 rounded ${
                        isSelected
                          ? 'bg-white/20 text-white'
                          : 'bg-mercury-lavender dark:bg-mercury-night text-mercury-ink-secondary dark:text-mercury-fog'
                      }`}
                    >
                      {cmd.category}
                    </span>
                    {cmd.shortcut && (
                      <kbd
                        className={`text-[10px] px-1.5 py-0.5 rounded font-mono ${
                          isSelected ? 'bg-white/30 text-white' : 'border border-mercury-fog/40'
                        }`}
                      >
                        {cmd.shortcut}
                      </kbd>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
